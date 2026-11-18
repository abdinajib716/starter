//! Off-site object storage (§24.2, M12.3, M17.9): S3-compatible (Cloudflare R2, AWS S3,
//! MinIO) through `object_store`, and Google Drive through a service account. Both stream
//! files, so size doesn't matter.
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, anyhow, bail};
use object_store::aws::AmazonS3Builder;
use object_store::{ObjectStore, path::Path as ObjPath};
use tokio::io::AsyncWriteExt;

fn env(name: &str) -> anyhow::Result<String> {
    std::env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| anyhow!("{name} is not set"))
}

pub enum Remote {
    S3(Arc<dyn ObjectStore>),
    Drive(super::gdrive::Drive),
}

/// `r2` | `gdrive` (the `backup.offsite_driver` / `storage.primary_driver` values).
pub fn for_target(target: &str) -> anyhow::Result<Remote> {
    match target {
        "r2" => Ok(Remote::S3(r2()?)),
        "gdrive" => Ok(Remote::Drive(super::gdrive::Drive::from_env()?)),
        other => bail!("unknown storage driver {other}"),
    }
}

/// From `R2_ACCOUNT_ID` (or `R2_ENDPOINT`), `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`.
fn r2() -> anyhow::Result<Arc<dyn ObjectStore>> {
    let endpoint = match env("R2_ENDPOINT") {
        Ok(e) => e,
        Err(_) => format!("https://{}.r2.cloudflarestorage.com", env("R2_ACCOUNT_ID")?),
    };
    let s = AmazonS3Builder::new()
        .with_endpoint(endpoint.clone())
        .with_allow_http(endpoint.starts_with("http://"))
        .with_region(std::env::var("R2_REGION").unwrap_or_else(|_| "auto".into()))
        .with_bucket_name(env("R2_BUCKET")?)
        .with_access_key_id(env("R2_ACCESS_KEY_ID")?)
        .with_secret_access_key(env("R2_SECRET_ACCESS_KEY")?)
        .with_virtual_hosted_style_request(false)
        .build()
        .context("R2 configuration")?;
    Ok(Arc::new(s))
}

impl Remote {
    pub async fn upload(&self, key: &str, file: &Path) -> anyhow::Result<u64> {
        match self {
            Self::S3(store) => {
                let mut w =
                    object_store::buffered::BufWriter::new(store.clone(), ObjPath::from(key));
                let mut f = tokio::fs::File::open(file).await?;
                let n = tokio::io::copy(&mut f, &mut w).await?;
                w.shutdown().await?;
                Ok(n)
            }
            Self::Drive(d) => d.upload(key, file).await,
        }
    }

    pub async fn download(&self, key: &str, to: &Path) -> anyhow::Result<u64> {
        match self {
            Self::S3(store) => {
                use futures_util::StreamExt;
                let mut stream = store.get(&ObjPath::from(key)).await?.into_stream();
                let mut out = tokio::fs::File::create(to).await?;
                let mut n = 0u64;
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    n += chunk.len() as u64;
                    out.write_all(&chunk).await?;
                }
                out.flush().await?;
                Ok(n)
            }
            Self::Drive(d) => d.download(key, to).await,
        }
    }

    /// Size of a remote object, or `None` when it doesn't exist.
    pub async fn size(&self, key: &str) -> anyhow::Result<Option<u64>> {
        match self {
            Self::S3(store) => match store.head(&ObjPath::from(key)).await {
                Ok(m) => Ok(Some(m.size)),
                Err(object_store::Error::NotFound { .. }) => Ok(None),
                Err(e) => Err(e.into()),
            },
            Self::Drive(d) => d.size(key).await,
        }
    }

    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        match self {
            Self::S3(store) => match store.delete(&ObjPath::from(key)).await {
                Ok(()) | Err(object_store::Error::NotFound { .. }) => Ok(()),
                Err(e) => Err(e.into()),
            },
            Self::Drive(d) => d.delete(key).await,
        }
    }

    /// "Test off-site connection": write, read back and delete a small file.
    pub async fn test(&self) -> anyhow::Result<()> {
        let key = format!(
            "starter-connection-test-{}.txt",
            uuid::Uuid::new_v4().simple()
        );
        let dir = std::env::temp_dir();
        let (src, back) = (dir.join(&key), dir.join(format!("{key}.back")));
        tokio::fs::write(&src, b"ok").await?;
        let r = async {
            self.upload(&key, &src).await.context("write")?;
            self.download(&key, &back).await.context("read")?;
            self.delete(&key).await.context("delete")?;
            if tokio::fs::read(&back).await? != b"ok" {
                bail!("read back different content");
            }
            Ok(())
        }
        .await;
        let _ = tokio::fs::remove_file(&src).await;
        let _ = tokio::fs::remove_file(&back).await;
        r
    }
}

#[cfg(test)]
mod tests {
    /// Runs against a real S3-compatible endpoint when `R2_ENDPOINT` is set
    /// (e.g. `docker run -p 8333:8333 chrislusf/seaweedfs server -s3`); skipped otherwise.
    #[tokio::test]
    async fn s3_round_trip_when_configured() {
        if std::env::var("R2_ENDPOINT").is_err() {
            return;
        }
        let r = super::for_target("r2").unwrap();
        r.test().await.unwrap();
        let dir = std::env::temp_dir();
        let src = dir.join(format!("s3t-{}", uuid::Uuid::new_v4()));
        let data = vec![7u8; 12 * 1024 * 1024]; // > one multipart part
        tokio::fs::write(&src, &data).await.unwrap();
        assert_eq!(
            r.upload("t/big.bin", &src).await.unwrap(),
            data.len() as u64
        );
        assert_eq!(r.size("t/big.bin").await.unwrap(), Some(data.len() as u64));
        let back = dir.join(format!("s3b-{}", uuid::Uuid::new_v4()));
        r.download("t/big.bin", &back).await.unwrap();
        assert_eq!(tokio::fs::read(&back).await.unwrap(), data);
        r.delete("t/big.bin").await.unwrap();
        assert_eq!(r.size("t/big.bin").await.unwrap(), None);
        let _ = tokio::fs::remove_file(src).await;
        let _ = tokio::fs::remove_file(back).await;
    }
}
