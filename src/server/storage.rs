//! Storage (§24.1, M11.1): the `Storage` trait and the `local` driver. Keys are validated
//! (no traversal, no absolute paths) before they touch the file system. R2 / Google Drive
//! drivers arrive with M12 behind the same trait.
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

pub trait Storage: Send + Sync {
    /// Move a finished local file into storage under `key`.
    fn put_file(
        &self,
        key: &str,
        from: &Path,
    ) -> impl std::future::Future<Output = anyhow::Result<u64>> + Send;
    /// Local path for streaming a stored file (local driver) — remote drivers download first.
    fn path(&self, key: &str) -> anyhow::Result<PathBuf>;
    fn delete(&self, key: &str) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

/// `segment/segment.ext` with `[A-Za-z0-9._-]` only, no `..`, not absolute.
pub fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 255
        && !key.starts_with('/')
        && key.split('/').all(|seg| {
            !seg.is_empty()
                && seg != "."
                && seg != ".."
                && seg
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
        })
}

pub struct Local {
    pub root: PathBuf,
}

impl Local {
    fn resolve(&self, key: &str) -> anyhow::Result<PathBuf> {
        if !valid_key(key) {
            bail!("invalid storage key");
        }
        Ok(self.root.join(key))
    }
}

impl Storage for Local {
    async fn put_file(&self, key: &str, from: &Path) -> anyhow::Result<u64> {
        let to = self.resolve(key)?;
        if let Some(dir) = to.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .context("create storage dir")?;
        }
        if tokio::fs::rename(from, &to).await.is_err() {
            tokio::fs::copy(from, &to)
                .await
                .context("copy into storage")?;
            let _ = tokio::fs::remove_file(from).await;
        }
        restrict(&to).await?;
        Ok(tokio::fs::metadata(&to).await?.len())
    }

    fn path(&self, key: &str) -> anyhow::Result<PathBuf> {
        self.resolve(key)
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        match tokio::fs::remove_file(self.resolve(key)?).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Owner read/write only (M11.9): dumps contain every row of the database.
pub async fn restrict(path: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_cannot_escape_the_root() {
        assert!(valid_key("backups/2026-11-17_020000.dump"));
        for bad in [
            "",
            "/etc/passwd",
            "../x",
            "a/../../b",
            "a//b",
            "a/./b",
            "a b",
            "a\\b",
        ] {
            assert!(!valid_key(bad), "{bad}");
        }
    }

    #[tokio::test]
    async fn local_put_and_delete() {
        let dir = std::env::temp_dir().join(format!("starter-storage-{}", uuid::Uuid::new_v4()));
        let s = Local { root: dir.clone() };
        let src = dir.join("tmp.bin");
        tokio::fs::create_dir_all(&dir).await.unwrap();
        tokio::fs::write(&src, b"hello").await.unwrap();
        assert_eq!(s.put_file("backups/a.dump", &src).await.unwrap(), 5);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(s.path("backups/a.dump").unwrap())
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        s.delete("backups/a.dump").await.unwrap();
        s.delete("backups/a.dump").await.unwrap(); // idempotent
        assert!(s.put_file("../evil", &src).await.is_err());
        let _ = tokio::fs::remove_dir_all(dir).await;
    }
}
