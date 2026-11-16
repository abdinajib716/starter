//! CSV export (§25.2): streamed response with a UTF-8 BOM (Excel), a fixed column list,
//! CSV-injection protection and the `export.max_rows` cap (`X-Export-Truncated: true`).
use dioxus::server::axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::sync::mpsc;

use super::error::AppError;

/// One CSV cell: quoted when needed; a leading `= + - @` (or tab/CR) gets a `'` so spreadsheet
/// apps don't run it as a formula.
pub fn cell(s: &str) -> String {
    let s = if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s.to_string()
    };
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

pub fn line(cells: &[&str]) -> String {
    let mut out = cells.iter().map(|c| cell(c)).collect::<Vec<_>>().join(",");
    out.push_str("\r\n");
    out
}

/// Stream `rx` (one chunk per message) as a CSV download. The producer task fills it.
pub fn csv_response(filename: &str, truncated: bool, rx: mpsc::Receiver<String>) -> Response {
    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        rx.recv()
            .await
            .map(|chunk| (Ok::<_, std::io::Error>(chunk), rx))
    });
    let mut res = Response::new(Body::from_stream(stream));
    let h = res.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        "x-export-truncated",
        HeaderValue::from_static(if truncated { "true" } else { "false" }),
    );
    res
}

/// Error → plain response for the export routes (no HTML error page, just the code).
pub fn error_response(e: AppError) -> Response {
    let status = match e {
        AppError::Unauthenticated => StatusCode::UNAUTHORIZED,
        AppError::Forbidden => StatusCode::FORBIDDEN,
        AppError::NotFound | AppError::Disabled(_) => StatusCode::NOT_FOUND,
        AppError::Validation(_) => StatusCode::BAD_REQUEST,
        AppError::Conflict(_) => StatusCode::CONFLICT,
        AppError::Internal(ref err) => {
            tracing::error!(error = ?err, "export failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (status, e.code()).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_are_quoted_and_formulas_neutralised() {
        assert_eq!(cell("plain"), "plain");
        assert_eq!(cell("a,b"), "\"a,b\"");
        assert_eq!(cell("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(cell("=HYPERLINK(\"x\")"), "\"'=HYPERLINK(\"\"x\"\")\"");
        assert_eq!(cell("+1"), "'+1");
        assert_eq!(cell("@SUM(A1)"), "'@SUM(A1)");
        assert_eq!(line(&["a", "b"]), "a,b\r\n");
    }
}
