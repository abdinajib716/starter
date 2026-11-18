//! Backup encryption (§24.2, M12.1) with `age`: to a public key (`BACKUP_PUBLIC_KEY`,
//! `age1…`; the private key never lives on the server) or, as the fallback, to a passphrase
//! (`BACKUP_ENCRYPTION_KEY`, scrypt). Streams file to file on a blocking thread.
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use age::secrecy::SecretString;
use anyhow::{Context, anyhow, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Off,
    PublicKey,
    Passphrase,
}

impl Mode {
    /// From `backup.encrypt_mode` (`none` | `age` | `symmetric`).
    pub fn parse(s: &str) -> Self {
        match s {
            "age" => Self::PublicKey,
            "symmetric" => Self::Passphrase,
            _ => Self::Off,
        }
    }
    pub fn scheme(self) -> Option<&'static str> {
        match self {
            Self::Off => None,
            Self::PublicKey => Some("age"),
            Self::Passphrase => Some("passphrase"),
        }
    }
}

fn env(name: &str) -> anyhow::Result<String> {
    std::env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| anyhow!("{name} is not set"))
}

/// Encrypt `from` into `to` (`.age`).
pub async fn encrypt(mode: Mode, from: PathBuf, to: PathBuf) -> anyhow::Result<()> {
    let key = match mode {
        Mode::Off => bail!("encryption is off"),
        Mode::PublicKey => env("BACKUP_PUBLIC_KEY")?,
        Mode::Passphrase => env("BACKUP_ENCRYPTION_KEY")?,
    };
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let encryptor = match mode {
            Mode::PublicKey => {
                let r = age::x25519::Recipient::from_str(key.trim())
                    .map_err(|e| anyhow!("BACKUP_PUBLIC_KEY is not an age public key: {e}"))?;
                age::Encryptor::with_recipients(std::iter::once(&r as &dyn age::Recipient))?
            }
            _ => age::Encryptor::with_user_passphrase(SecretString::from(key)),
        };
        let mut input = BufReader::new(std::fs::File::open(&from)?);
        let out = std::fs::File::create(&to)?;
        let mut w = encryptor.wrap_output(std::io::BufWriter::new(out))?;
        std::io::copy(&mut input, &mut w)?;
        w.finish()?.flush()?;
        Ok(())
    })
    .await?
}

/// Decrypt `from` into `to`. `secret` = the private key (`AGE-SECRET-KEY-…`, given at restore
/// time and kept in memory only) or the passphrase; `None` falls back to the environment.
pub async fn decrypt(
    scheme: &str,
    secret: Option<String>,
    from: PathBuf,
    to: PathBuf,
) -> anyhow::Result<()> {
    let scheme = scheme.to_string();
    let secret = match (scheme.as_str(), secret.filter(|s| !s.trim().is_empty())) {
        (_, Some(s)) => s,
        ("passphrase", None) => env("BACKUP_ENCRYPTION_KEY")?,
        (_, None) => {
            env("BACKUP_PRIVATE_KEY").context("the backup's private key is needed to restore it")?
        }
    };
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let input = BufReader::new(std::fs::File::open(&from)?);
        let d = age::Decryptor::new(input).map_err(|e| anyhow!("not an age file: {e}"))?;
        let mut reader: Box<dyn Read> = if scheme == "passphrase" {
            let id = age::scrypt::Identity::new(SecretString::from(secret));
            Box::new(
                d.decrypt(std::iter::once(&id as &dyn age::Identity))
                    .map_err(|e| anyhow!("wrong passphrase: {e}"))?,
            )
        } else {
            let id = age::x25519::Identity::from_str(secret.trim())
                .map_err(|e| anyhow!("not an age private key: {e}"))?;
            Box::new(
                d.decrypt(std::iter::once(&id as &dyn age::Identity))
                    .map_err(|e| anyhow!("wrong private key: {e}"))?,
            )
        };
        let mut out = std::io::BufWriter::new(std::fs::File::create(&to)?);
        std::io::copy(&mut reader, &mut out)?;
        out.flush()?;
        Ok(())
    })
    .await?
}

/// SHA-256 (hex) of a file.
pub async fn sha256_file(path: &Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncReadExt;
    let mut f = tokio::fs::File::open(path).await?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn round_trip_public_key_and_passphrase() {
        let dir = std::env::temp_dir().join(format!("bk-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let plain = dir.join("a.dump");
        std::fs::write(&plain, b"PGDMP fake dump body").unwrap();
        let id = age::x25519::Identity::generate();
        let pk = id.to_public().to_string();
        let sk = age::secrecy::ExposeSecret::expose_secret(&id.to_string()).to_string();
        // SAFETY: tests in this module run sequentially on these variables only.
        unsafe { std::env::set_var("BACKUP_PUBLIC_KEY", &pk) };
        encrypt(Mode::PublicKey, plain.clone(), dir.join("a.age"))
            .await
            .unwrap();
        decrypt("age", Some(sk), dir.join("a.age"), dir.join("b.dump"))
            .await
            .unwrap();
        assert_eq!(
            std::fs::read(dir.join("b.dump")).unwrap(),
            b"PGDMP fake dump body"
        );
        assert!(
            decrypt(
                "age",
                Some(
                    age::secrecy::ExposeSecret::expose_secret(
                        &age::x25519::Identity::generate().to_string()
                    )
                    .to_string()
                ),
                dir.join("a.age"),
                dir.join("c.dump")
            )
            .await
            .is_err()
        );

        unsafe { std::env::set_var("BACKUP_ENCRYPTION_KEY", "correct horse battery staple") };
        encrypt(Mode::Passphrase, plain, dir.join("p.age"))
            .await
            .unwrap();
        decrypt("passphrase", None, dir.join("p.age"), dir.join("p.dump"))
            .await
            .unwrap();
        assert_eq!(
            std::fs::read(dir.join("p.dump")).unwrap(),
            b"PGDMP fake dump body"
        );
        std::fs::remove_dir_all(dir).ok();
    }
}
