//! URL query state (`?filter=x&type=connected`), the Rust side of the reference's
//! `route.useSearch()` / `navigate({ search })` and `use-table-url-state.ts`. Used with
//! `#[route("/path?:..query")]`; the router needs `From<&str>` + `Display`.
use std::fmt;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query(Vec<(String, String)>);

impl Query {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Set or (with `None` / empty) remove a key; keeps the other keys' order.
    pub fn with(mut self, key: &str, value: Option<impl Into<String>>) -> Self {
        let value = value.map(Into::into).filter(|v| !v.is_empty());
        match (self.0.iter().position(|(k, _)| k == key), value) {
            (Some(i), Some(v)) => self.0[i].1 = v,
            (Some(i), None) => {
                self.0.remove(i);
            }
            (None, Some(v)) => self.0.push((key.to_string(), v)),
            (None, None) => {}
        }
        self
    }
}

fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                // Bytes, not `&s[..]`: slicing a str inside a multi-byte character would panic.
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(b) => {
                        out.push(b);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

impl From<&str> for Query {
    fn from(q: &str) -> Self {
        Query(
            q.trim_start_matches('?')
                .split('&')
                .filter(|p| !p.is_empty())
                .map(|p| {
                    let (k, v) = p.split_once('=').unwrap_or((p, ""));
                    (decode(k), decode(v))
                })
                .collect(),
        )
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: Vec<String> = self
            .0
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
            .collect();
        f.write_str(&s.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

    #[test]
    fn round_trip() {
        let q = Query::from("filter=no%20te&type=connected&x=a+b");
        assert_eq!(q.get("filter"), Some("no te"));
        assert_eq!(q.get("x"), Some("a b"));
        let q = q.with("type", None::<String>).with("sort", Some("desc"));
        assert_eq!(q.to_string(), "filter=no%20te&x=a%20b&sort=desc");
        assert_eq!(Query::from(""), Query::default());
    }
}
