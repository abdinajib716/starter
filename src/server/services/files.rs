//! Uploaded files (§24.1, M11.2–11.3): avatars. Content is sniffed from its magic bytes (the
//! browser's name and type are ignored), decoded, cropped square, resized and re-encoded as
//! WebP, so what is stored is always an image we produced. Served only to signed-in users of
//! the same organization.
use dioxus::server::axum::{
    Extension,
    body::Body,
    extract::Path,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::user_entry;
use crate::server::audit::{self, Entry};
use crate::server::ctx::Ctx;
use crate::server::error::AppError;
use crate::server::storage::{Local, Storage};

pub const AVATAR_MAX_BYTES: usize = 1024 * 1024;
const AVATAR_PX: u32 = 256;

fn driver() -> String {
    crate::server::settings::cache()
        .map(|c| c.get("storage.primary_driver"))
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "local".into())
}

fn local() -> anyhow::Result<Local> {
    let st = crate::server::state::get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(Local {
        root: st.config.files_dir.clone(),
    })
}

/// Magic bytes → format. Only these four are accepted.
pub fn sniff(bytes: &[u8]) -> Option<image::ImageFormat> {
    match bytes {
        [0x89, b'P', b'N', b'G', ..] => Some(image::ImageFormat::Png),
        [0xFF, 0xD8, 0xFF, ..] => Some(image::ImageFormat::Jpeg),
        [b'G', b'I', b'F', b'8', ..] => Some(image::ImageFormat::Gif),
        [
            b'R',
            b'I',
            b'F',
            b'F',
            _,
            _,
            _,
            _,
            b'W',
            b'E',
            b'B',
            b'P',
            ..,
        ] => Some(image::ImageFormat::WebP),
        _ => None,
    }
}

/// Decode, centre-crop to a square, resize to 256 px, encode WebP.
pub fn to_avatar_webp(bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    let fmt = sniff(bytes).ok_or(AppError::Validation("files.not_an_image"))?;
    let mut reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), fmt);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8000);
    limits.max_image_height = Some(8000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let img = reader
        .decode()
        .map_err(|_| AppError::Validation("files.not_an_image"))?;
    let side = img.width().min(img.height());
    let img = img
        .crop_imm(
            (img.width() - side) / 2,
            (img.height() - side) / 2,
            side,
            side,
        )
        .resize_exact(AVATAR_PX, AVATAR_PX, image::imageops::FilterType::Lanczos3);
    let mut out = Vec::new();
    img.to_rgba8()
        .write_to(
            &mut std::io::Cursor::new(&mut out),
            image::ImageFormat::WebP,
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("webp encode: {e}")))?;
    Ok(out)
}

async fn store(driver: &str, key: &str, bytes: &[u8]) -> anyhow::Result<()> {
    let tmp = std::env::temp_dir().join(format!("up-{}", Uuid::new_v4().simple()));
    tokio::fs::write(&tmp, bytes).await?;
    let r = match driver {
        "local" => local()?.put_file(key, &tmp).await.map(|_| ()),
        other => crate::server::remote::for_target(other)?
            .upload(&format!("files/{key}"), &tmp)
            .await
            .map(|_| ()),
    };
    let _ = tokio::fs::remove_file(&tmp).await;
    r
}

async fn remove(driver: &str, key: &str) -> anyhow::Result<()> {
    match driver {
        "local" => local()?.delete(key).await,
        other => {
            crate::server::remote::for_target(other)?
                .delete(&format!("files/{key}"))
                .await
        }
    }
}

/// Own avatar; replaces (and deletes) the previous one.
pub async fn set_avatar(pool: &PgPool, ctx: &Ctx, bytes: Vec<u8>) -> Result<Uuid, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    if bytes.len() > AVATAR_MAX_BYTES {
        return Err(AppError::Validation("files.too_large"));
    }
    let webp = tokio::task::spawn_blocking(move || to_avatar_webp(&bytes))
        .await
        .map_err(|e| AppError::Internal(e.into()))??;
    let sha = {
        use sha2::{Digest, Sha256};
        Sha256::digest(&webp)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    let id = Uuid::new_v4();
    let key = format!("avatars/{}.webp", id.simple());
    let drv = driver();
    store(&drv, &key, &webp).await?;
    let mut tx = pool.begin().await?;
    let old = sqlx::query!(
        "SELECT f.id, f.driver, f.storage_key FROM users u JOIN files f ON f.id::text = u.avatar_key WHERE u.id = $1",
        user.id
    )
    .fetch_optional(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO files (id, org_id, owner_id, kind, driver, storage_key, mime, size_bytes, sha256)
         VALUES ($1, $2, $3, 'avatar', $4, $5, 'image/webp', $6, $7)",
        id,
        ctx.org_id,
        user.id,
        drv,
        key,
        webp.len() as i64,
        sha
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE users SET avatar_key = $2, updated_at = now() WHERE id = $1",
        user.id,
        id.to_string()
    )
    .execute(&mut *tx)
    .await?;
    if let Some(o) = &old {
        sqlx::query!("DELETE FROM files WHERE id = $1", o.id)
            .execute(&mut *tx)
            .await?;
    }
    audit::record(
        &mut tx,
        Entry {
            after: Some(json!({ "file": id, "bytes": webp.len() })),
            ..user_entry(ctx, user, "profile.avatar")
        },
    )
    .await?;
    tx.commit().await?;
    if let Some(o) = old
        && let Err(e) = remove(&o.driver, &o.storage_key).await
    {
        tracing::warn!("old avatar not deleted: {e:#}");
    }
    Ok(id)
}

pub async fn remove_avatar(pool: &PgPool, ctx: &Ctx) -> Result<(), AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    let mut tx = pool.begin().await?;
    let old = sqlx::query!(
        "SELECT f.id, f.driver, f.storage_key FROM users u JOIN files f ON f.id::text = u.avatar_key WHERE u.id = $1",
        user.id
    )
    .fetch_optional(&mut *tx)
    .await?;
    sqlx::query!("UPDATE users SET avatar_key = NULL WHERE id = $1", user.id)
        .execute(&mut *tx)
        .await?;
    if let Some(o) = &old {
        sqlx::query!("DELETE FROM files WHERE id = $1", o.id)
            .execute(&mut *tx)
            .await?;
    }
    audit::record(&mut tx, user_entry(ctx, user, "profile.avatar_removed")).await?;
    tx.commit().await?;
    if let Some(o) = old {
        let _ = remove(&o.driver, &o.storage_key).await;
    }
    Ok(())
}

/// `GET /files/avatars/{id}`: signed in, same organization; cached privately for a day
/// (a new avatar gets a new id, so the URL changes).
pub async fn avatar_route(Extension(ctx): Extension<Ctx>, Path(id): Path<Uuid>) -> Response {
    let Some(user) = ctx.user.as_ref() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let r: anyhow::Result<Option<Vec<u8>>> = async {
        let st = crate::server::state::get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let Some(f) = sqlx::query!(
            "SELECT driver, storage_key FROM files WHERE id = $1 AND org_id = $2 AND kind = 'avatar'",
            id,
            user.org_id
        )
        .fetch_optional(&st.pool)
        .await?
        else {
            return Ok(None);
        };
        Ok(Some(match f.driver.as_str() {
            "local" => tokio::fs::read(local()?.path(&f.storage_key)?).await?,
            other => {
                let tmp = std::env::temp_dir().join(format!("dl-{}", Uuid::new_v4().simple()));
                crate::server::remote::for_target(other)?.download(&format!("files/{}", f.storage_key), &tmp).await?;
                let b = tokio::fs::read(&tmp).await?;
                let _ = tokio::fs::remove_file(&tmp).await;
                b
            }
        }))
    }
    .await;
    match r {
        Ok(Some(bytes)) => {
            let mut res = Response::new(Body::from(bytes));
            let h = res.headers_mut();
            h.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/webp"));
            h.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, max-age=86400, immutable"),
            );
            h.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );
            res
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("avatar: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_and_reencodes() {
        let mut png = Vec::new();
        image::RgbImage::from_pixel(300, 200, image::Rgb([200, 30, 30]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        assert_eq!(sniff(&png), Some(image::ImageFormat::Png));
        let webp = to_avatar_webp(&png).unwrap();
        assert_eq!(sniff(&webp), Some(image::ImageFormat::WebP));
        let back = image::load_from_memory(&webp).unwrap();
        assert_eq!((back.width(), back.height()), (256, 256));
        // A script renamed .png is refused.
        assert!(to_avatar_webp(b"<svg onload=alert(1)>").is_err());
        assert!(sniff(b"GIF89a").is_some());
    }
}
