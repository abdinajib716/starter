//! HTML → PDF with headless Chrome over the DevTools protocol (CDP).
//!
//! Speed: one WebSocket to the browser is opened on first use and kept; each PDF is a new tab
//! (`Target.createTarget`), the HTML is injected directly (`Page.setDocumentContent`, no URL
//! fetch, no file on disk), printed (`Page.printToPDF`) and the tab closed. A render takes
//! ~100–300 ms. At most `MAX_PARALLEL` renders run at once; a dropped connection is reopened
//! on the next call.
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{Context, anyhow, bail};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, Semaphore, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;

const MAX_PARALLEL: usize = 4;
const CALL_TIMEOUT: Duration = Duration::from_secs(20);

struct Cdp {
    out: mpsc::UnboundedSender<String>,
    pending: Arc<std::sync::Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    next: AtomicU64,
    alive: Arc<AtomicBool>,
}

static CLIENT: Mutex<Option<Arc<Cdp>>> = Mutex::const_new(None);
static LIMIT: Semaphore = Semaphore::const_new(MAX_PARALLEL);

/// `http://host:port` → the browser WebSocket URL from `/json/version`.
async fn browser_ws_url(base: &str) -> anyhow::Result<String> {
    let hostport = base.trim_start_matches("http://").trim_end_matches('/');
    let mut s = tokio::net::TcpStream::connect(hostport)
        .await
        .with_context(|| format!("PDF service (Chrome) not reachable at {base}"))?;
    s.write_all(
        format!("GET /json/version HTTP/1.1\r\nHost: {hostport}\r\nConnection: close\r\n\r\n")
            .as_bytes(),
    )
    .await?;
    // Read headers, then exactly Content-Length bytes (the proxy may keep the socket open).
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let body = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let n = s.read(&mut chunk).await?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            let text = String::from_utf8_lossy(&buf).to_string();
            if let Some((head, body)) = text.split_once("\r\n\r\n") {
                let len = head
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().to_string())
                    })
                    .and_then(|v| v.parse::<usize>().ok());
                if len.is_none_or(|l| body.len() >= l) {
                    return Ok::<_, std::io::Error>(body.to_string());
                }
            }
        }
        Ok(String::from_utf8_lossy(&buf)
            .split_once("\r\n\r\n")
            .map(|x| x.1.to_string())
            .unwrap_or_default())
    })
    .await
    .map_err(|_| anyhow!("Chrome /json/version timed out"))??;
    let v: Value = serde_json::from_str(&body).context("bad /json/version")?;
    v["webSocketDebuggerUrl"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("no webSocketDebuggerUrl"))
}

async fn connect(base: &str) -> anyhow::Result<Arc<Cdp>> {
    let url = browser_ws_url(base).await?;
    let (ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .context("CDP connect")?;
    let (mut sink, mut stream) = ws.split();
    let (out, mut rx) = mpsc::unbounded_channel::<String>();
    let pending: Arc<std::sync::Mutex<HashMap<u64, oneshot::Sender<Value>>>> = Arc::default();
    let alive = Arc::new(AtomicBool::new(true));
    let a1 = alive.clone();
    tokio::spawn(async move {
        while let Some(m) = rx.recv().await {
            if sink.send(Message::Text(m.into())).await.is_err() {
                break;
            }
        }
        a1.store(false, Ordering::SeqCst);
    });
    let (p2, a2) = (pending.clone(), alive.clone());
    tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            let Message::Text(t) = msg else { continue };
            let Ok(v) = serde_json::from_str::<Value>(&t) else {
                continue;
            };
            if let Some(id) = v.get("id").and_then(Value::as_u64)
                && let Some(tx) = p2.lock().ok().and_then(|mut p| p.remove(&id))
            {
                let _ = tx.send(v);
            }
        }
        a2.store(false, Ordering::SeqCst);
        tracing::warn!("PDF: Chrome connection closed; it reopens on the next export");
    });
    Ok(Arc::new(Cdp {
        out,
        pending,
        next: AtomicU64::new(1),
        alive,
    }))
}

impl Cdp {
    async fn call(
        &self,
        method: &str,
        params: Value,
        session: Option<&str>,
    ) -> anyhow::Result<Value> {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        let mut msg = json!({ "id": id, "method": method, "params": params });
        if let Some(s) = session {
            msg["sessionId"] = json!(s);
        }
        let (tx, rx) = oneshot::channel();
        self.pending
            .lock()
            .map_err(|_| anyhow!("lock"))?
            .insert(id, tx);
        self.out
            .send(msg.to_string())
            .map_err(|_| anyhow!("CDP connection closed"))?;
        let v = tokio::time::timeout(CALL_TIMEOUT, rx)
            .await
            .map_err(|_| anyhow!("{method} timed out"))?
            .map_err(|_| anyhow!("CDP connection closed"))?;
        if let Some(e) = v.get("error") {
            bail!("{method}: {e}");
        }
        Ok(v["result"].clone())
    }
}

async fn client(base: &str) -> anyhow::Result<Arc<Cdp>> {
    let mut c = CLIENT.lock().await;
    if let Some(cdp) = c.as_ref()
        && cdp.alive.load(Ordering::SeqCst)
    {
        return Ok(cdp.clone());
    }
    let cdp = connect(base).await?;
    *c = Some(cdp.clone());
    Ok(cdp)
}

/// Page setup for `Page.printToPDF`.
pub struct PrintOptions {
    pub landscape: bool,
    /// HTML for the running header/footer (Chrome fills `.pageNumber`, `.totalPages`).
    pub header: String,
    pub footer: String,
}

/// Render `html` to PDF bytes.
pub async fn render(base: &str, html: &str, opt: &PrintOptions) -> anyhow::Result<Vec<u8>> {
    let _permit = LIMIT.acquire().await?;
    let cdp = client(base).await?;
    let target = cdp
        .call("Target.createTarget", json!({ "url": "about:blank" }), None)
        .await?;
    let target_id = target["targetId"]
        .as_str()
        .ok_or_else(|| anyhow!("no targetId"))?
        .to_string();
    let result = async {
        let att = cdp
            .call(
                "Target.attachToTarget",
                json!({ "targetId": target_id, "flatten": true }),
                None,
            )
            .await?;
        let session = att["sessionId"]
            .as_str()
            .ok_or_else(|| anyhow!("no sessionId"))?
            .to_string();
        let s = Some(session.as_str());
        let tree = cdp.call("Page.getFrameTree", json!({}), s).await?;
        let frame = tree["frameTree"]["frame"]["id"]
            .as_str()
            .ok_or_else(|| anyhow!("no frame"))?
            .to_string();
        cdp.call(
            "Page.setDocumentContent",
            json!({ "frameId": frame, "html": html }),
            s,
        )
        .await?;
        // Wait for web fonts / images referenced by the document.
        cdp.call(
            "Runtime.evaluate",
            json!({ "expression": "document.fonts.ready.then(() => true)", "awaitPromise": true }),
            s,
        )
        .await?;
        let pdf = cdp
            .call(
                "Page.printToPDF",
                json!({
                    "landscape": opt.landscape,
                    "printBackground": true,
                    "preferCSSPageSize": true,
                    "displayHeaderFooter": true,
                    "headerTemplate": opt.header,
                    "footerTemplate": opt.footer,
                    "marginTop": 0.6, "marginBottom": 0.6, "marginLeft": 0.4, "marginRight": 0.4,
                }),
                s,
            )
            .await?;
        let data = pdf["data"].as_str().ok_or_else(|| anyhow!("no PDF data"))?;
        Ok::<_, anyhow::Error>(base64::engine::general_purpose::STANDARD.decode(data)?)
    }
    .await;
    // Close the tab whatever happened (not awaited by the caller).
    let cdp2 = cdp.clone();
    tokio::spawn(async move {
        let _ = cdp2
            .call("Target.closeTarget", json!({ "targetId": target_id }), None)
            .await;
    });
    result
}
