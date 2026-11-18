//! TOTP second factor (§21, M9.7): RFC 6238 codes (SHA-1, 6 digits, 30 s), the secret kept
//! AES-256-GCM encrypted under a key derived (HKDF-SHA256) from `APP_SECRET`, recovery codes.
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{Context, anyhow};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm, Secret, TOTP};

const STEP: u64 = 30;

fn cipher(app_secret: &str) -> Aes256Gcm {
    let hk = hkdf::Hkdf::<Sha256>::new(Some(b"starter-totp-v1"), app_secret.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(b"totp secret encryption", &mut key)
        .expect("32 bytes is a valid HKDF length");
    Aes256Gcm::new(&key.into())
}

/// `nonce || ciphertext`.
pub fn encrypt(app_secret: &str, plain: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut nonce = [0u8; 12];
    getrandom::fill(&mut nonce).map_err(|e| anyhow!("rng: {e}"))?;
    let ct = cipher(app_secret)
        .encrypt(Nonce::from_slice(&nonce), plain)
        .map_err(|_| anyhow!("encrypt"))?;
    Ok([nonce.as_slice(), &ct].concat())
}

pub fn decrypt(app_secret: &str, blob: &[u8]) -> anyhow::Result<Vec<u8>> {
    let (nonce, ct) = blob.split_at_checked(12).context("ciphertext too short")?;
    cipher(app_secret)
        .decrypt(Nonce::from_slice(nonce), ct)
        .map_err(|_| anyhow!("TOTP secret cannot be decrypted (APP_SECRET changed?)"))
}

/// 20 random bytes (160 bits, the RFC 4226 recommendation).
pub fn new_secret() -> Vec<u8> {
    let mut s = vec![0u8; 20];
    getrandom::fill(&mut s).expect("OS RNG available");
    s
}

pub fn base32(secret: &[u8]) -> String {
    Secret::Raw(secret.to_vec()).to_encoded().to_string()
}

fn totp(secret: &[u8]) -> anyhow::Result<TOTP> {
    TOTP::new(Algorithm::SHA1, 6, 1, STEP, secret.to_vec()).map_err(|e| anyhow!("{e}"))
}

/// The time step a valid `code` belongs to (now ±1 step), or `None`. Steps `<= last_step`
/// are refused so a code can't be replayed.
pub fn verify(secret: &[u8], code: &str, last_step: i64, now: u64) -> Option<i64> {
    let code: String = code.chars().filter(char::is_ascii_digit).collect();
    if code.len() != 6 {
        return None;
    }
    let t = totp(secret).ok()?;
    let current = now / STEP;
    [current.saturating_sub(1), current, current + 1]
        .into_iter()
        .filter(|s| i64::try_from(*s).is_ok_and(|s| s > last_step))
        .find(|s| constant_eq(&t.generate(s * STEP), &code))
        .and_then(|s| i64::try_from(s).ok())
}

fn constant_eq(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `otpauth://totp/Issuer:account?secret=…&issuer=…` for authenticator apps.
pub fn otpauth_url(issuer: &str, account: &str, secret: &[u8]) -> String {
    let enc = |s: &str| {
        s.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'@' => {
                    (b as char).to_string()
                }
                _ => format!("%{b:02X}"),
            })
            .collect::<String>()
    };
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        enc(issuer),
        enc(account),
        base32(secret),
        enc(issuer)
    )
}

/// The otpauth URL as an inline SVG QR code.
pub fn qr_svg(url: &str) -> anyhow::Result<String> {
    let code = qrcode::QrCode::new(url.as_bytes())?;
    Ok(code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .quiet_zone(true)
        .build())
}

/// Ten one-time recovery codes `xxxxx-xxxxx` (lowercase base32, ~50 bits each).
pub fn recovery_codes() -> Vec<String> {
    const ALPHA: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    (0..10)
        .map(|_| {
            let mut b = [0u8; 10];
            getrandom::fill(&mut b).expect("OS RNG available");
            let s: String = b
                .iter()
                .map(|x| ALPHA[*x as usize % ALPHA.len()] as char)
                .collect();
            format!("{}-{}", &s[..5], &s[5..])
        })
        .collect()
}

/// Hash of a recovery code, ignoring case, spaces and dashes.
pub fn recovery_hash(code: &str) -> Vec<u8> {
    let norm: String = code
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    Sha256::digest(norm.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_round_trip_and_wrong_key_fails() {
        let s = new_secret();
        let blob = encrypt("k".repeat(32).as_str(), &s).unwrap();
        assert_eq!(decrypt(&"k".repeat(32), &blob).unwrap(), s);
        assert!(decrypt(&"x".repeat(32), &blob).is_err());
    }

    #[test]
    fn verify_window_and_replay() {
        let s = new_secret();
        let now = 1_700_000_000;
        let code = totp(&s).unwrap().generate(now);
        let step = verify(&s, &code, 0, now).expect("valid");
        assert_eq!(step as u64, now / STEP);
        // Same step again: replay refused.
        assert_eq!(verify(&s, &code, step, now), None);
        // Previous step still accepted (clock drift), two steps back not.
        let prev = totp(&s).unwrap().generate(now - STEP);
        assert!(verify(&s, &prev, 0, now).is_some());
        let old = totp(&s).unwrap().generate(now - 2 * STEP);
        assert!(old == code || verify(&s, &old, 0, now).is_none());
        assert_eq!(verify(&s, "12345", 0, now), None);
    }

    #[test]
    fn recovery_codes_normalise() {
        let c = recovery_codes();
        assert_eq!(c.len(), 10);
        assert_eq!(
            recovery_hash(&c[0]),
            recovery_hash(&c[0].to_uppercase().replace('-', " "))
        );
        assert!(
            otpauth_url("My App", "a@b.c", &new_secret())
                .starts_with("otpauth://totp/My%20App:a@b.c?secret=")
        );
        assert!(qr_svg("otpauth://totp/x").unwrap().contains("<svg"));
    }
}
