//! Google Drive driver (M17.9) with a service account: `GDRIVE_SERVICE_ACCOUNT_JSON` (the key
//! file's JSON, or a path to it) and `GDRIVE_FOLDER_ID` (a folder shared with the service
//! account, or one in a shared drive). Keys map to file names inside that folder.
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, anyhow, bail};
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

const API: &str = "https://www.googleapis.com/drive/v3";

pub struct Drive {
    email: String,
    key_pem: String,
    token_uri: String,
    folder: String,
}

static TOKEN: Mutex<Option<(String, u64)>> = Mutex::const_new(None);

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn http() -> &'static reqwest::Client {
    static C: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(3600))
            .build()
            .expect("HTTP client")
    })
}

impl Drive {
    pub fn from_env() -> anyhow::Result<Self> {
        let raw = std::env::var("GDRIVE_SERVICE_ACCOUNT_JSON")
            .context("GDRIVE_SERVICE_ACCOUNT_JSON is not set")?;
        let text = if raw.trim_start().starts_with('{') {
            raw
        } else {
            std::fs::read_to_string(raw.trim())?
        };
        let v: Value = serde_json::from_str(&text).context("service account JSON")?;
        Ok(Self {
            email: v["client_email"].as_str().context("client_email")?.into(),
            key_pem: v["private_key"].as_str().context("private_key")?.into(),
            token_uri: v["token_uri"]
                .as_str()
                .unwrap_or("https://oauth2.googleapis.com/token")
                .into(),
            folder: std::env::var("GDRIVE_FOLDER_ID").context("GDRIVE_FOLDER_ID is not set")?,
        })
    }

    /// OAuth access token from a signed JWT (cached until 1 minute before expiry).
    async fn token(&self) -> anyhow::Result<String> {
        let mut t = TOKEN.lock().await;
        if let Some((tok, exp)) = t.as_ref()
            && *exp > now() + 60
        {
            return Ok(tok.clone());
        }
        let iat = now();
        let claims = json!({
            "iss": self.email, "scope": "https://www.googleapis.com/auth/drive",
            "aud": self.token_uri, "iat": iat, "exp": iat + 3600,
        });
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(self.key_pem.as_bytes())?;
        let jwt = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &key,
        )?;
        let res: Value = http()
            .post(&self.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", jwt.as_str()),
            ])
            .send()
            .await?
            .json()
            .await?;
        let tok = res["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("Google token: {res}"))?
            .to_string();
        let exp = iat + res["expires_in"].as_u64().unwrap_or(3600);
        *t = Some((tok.clone(), exp));
        Ok(tok)
    }

    fn name(key: &str) -> String {
        key.replace('/', "__")
    }

    /// The Drive file id of `key` in the folder, if any.
    async fn find(&self, key: &str) -> anyhow::Result<Option<(String, u64)>> {
        let q = format!(
            "name = '{}' and '{}' in parents and trashed = false",
            Self::name(key).replace('\'', "\\'"),
            self.folder
        );
        let res: Value = http()
            .get(format!("{API}/files"))
            .bearer_auth(self.token().await?)
            .query(&[
                ("q", q.as_str()),
                ("fields", "files(id,size)"),
                ("supportsAllDrives", "true"),
                ("includeItemsFromAllDrives", "true"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(res["files"].as_array().and_then(|a| a.first()).map(|f| {
            (
                f["id"].as_str().unwrap_or_default().to_string(),
                f["size"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0),
            )
        }))
    }

    pub async fn upload(&self, key: &str, file: &Path) -> anyhow::Result<u64> {
        let size = tokio::fs::metadata(file).await?.len();
        // Resumable session, then one streamed PUT of the whole file.
        let res = http()
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable&supportsAllDrives=true")
            .bearer_auth(self.token().await?)
            .header("X-Upload-Content-Length", size)
            .json(&json!({ "name": Self::name(key), "parents": [self.folder] }))
            .send()
            .await?
            .error_for_status()?;
        let location = res
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .context("no upload location")?
            .to_string();
        let f = tokio::fs::File::open(file).await?;
        let body = reqwest::Body::wrap_stream(tokio_util_stream(f));
        http()
            .put(location)
            .header("Content-Length", size)
            .body(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(size)
    }

    pub async fn download(&self, key: &str, to: &Path) -> anyhow::Result<u64> {
        use futures_util::StreamExt;
        let (id, _) = self.find(key).await?.context("file not found on Drive")?;
        let res = http()
            .get(format!("{API}/files/{id}?alt=media&supportsAllDrives=true"))
            .bearer_auth(self.token().await?)
            .send()
            .await?
            .error_for_status()?;
        let mut out = tokio::fs::File::create(to).await?;
        let mut n = 0;
        let mut s = res.bytes_stream();
        while let Some(c) = s.next().await {
            let c = c?;
            n += c.len() as u64;
            out.write_all(&c).await?;
        }
        out.flush().await?;
        Ok(n)
    }

    pub async fn size(&self, key: &str) -> anyhow::Result<Option<u64>> {
        Ok(self.find(key).await?.map(|(_, s)| s))
    }

    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let Some((id, _)) = self.find(key).await? else {
            return Ok(());
        };
        let r = http()
            .delete(format!("{API}/files/{id}?supportsAllDrives=true"))
            .bearer_auth(self.token().await?)
            .send()
            .await?;
        if !r.status().is_success() && r.status().as_u16() != 404 {
            bail!("Drive delete: {}", r.status());
        }
        Ok(())
    }
}

/// File → byte stream for a streaming request body.
fn tokio_util_stream(
    f: tokio::fs::File,
) -> impl futures_util::Stream<Item = std::io::Result<bytes::Bytes>> + Send + 'static {
    futures_util::stream::unfold(f, |mut f| async move {
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0u8; 256 * 1024];
        match f.read(&mut buf).await {
            Ok(0) => None,
            Ok(n) => {
                buf.truncate(n);
                Some((Ok(bytes::Bytes::from(buf)), f))
            }
            Err(e) => Some((Err(e), f)),
        }
    })
}
