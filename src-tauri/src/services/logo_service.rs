use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::{models::LogoUploadPayload, utils::errors::AppError};

const MAX_LOGO_BYTES: usize = 2 * 1024 * 1024;

pub fn save_logo(
    db_path: &Path,
    folder: &str,
    stem: &str,
    payload: &LogoUploadPayload,
) -> Result<String, AppError> {
    let (extension, expected_mime) = accepted_type(&payload.mime_type)?;
    let encoded = payload
        .base64_data
        .split_once(',')
        .map(|(_, data)| data)
        .unwrap_or(payload.base64_data.as_str());
    if encoded.len() > MAX_LOGO_BYTES.div_ceil(3) * 4 + 4 {
        return Err(AppError::validation(
            "Logo files must be valid images no larger than 2 MB.",
        ));
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| AppError::validation("The selected logo file could not be decoded."))?;
    if bytes.is_empty() || bytes.len() > MAX_LOGO_BYTES {
        return Err(AppError::validation(
            "Logo files must be valid images no larger than 2 MB.",
        ));
    }
    validate_magic(&bytes, expected_mime)?;

    let target_dir = branding_root(db_path).join(folder);
    fs::create_dir_all(&target_dir)?;
    for old_extension in ["png", "jpg", "webp"] {
        let old = target_dir.join(format!("{stem}.{old_extension}"));
        if old.exists() {
            let _ = fs::remove_file(old);
        }
    }
    fs::write(target_dir.join(format!("{stem}.{extension}")), bytes)?;
    Ok(format!("branding/{folder}/{stem}.{extension}"))
}

pub fn remove_logo(db_path: &Path, relative_path: Option<&str>) -> Result<(), AppError> {
    if let Some(path) = relative_path.and_then(safe_logo_path) {
        let target = db_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path);
        if target.exists() {
            fs::remove_file(target)?;
        }
    }
    Ok(())
}

pub fn logo_data_uri(db_path: &Path, relative_path: Option<&str>) -> Option<String> {
    let relative = relative_path.and_then(safe_logo_path)?;
    let path = db_path.parent()?.join(relative);
    let bytes = fs::read(path).ok()?;
    if bytes.is_empty() || bytes.len() > MAX_LOGO_BYTES {
        return None;
    }
    let mime = match relative.extension().and_then(|value| value.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => return None,
    };
    if validate_magic(&bytes, mime).is_err() {
        return None;
    }
    Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

fn branding_root(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("branding")
}

fn safe_logo_path(value: &str) -> Option<&Path> {
    let path = Path::new(value);
    if !value.starts_with("branding/")
        || path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return None;
    }
    Some(path)
}

fn accepted_type(mime: &str) -> Result<(&'static str, &'static str), AppError> {
    match mime.trim().to_ascii_lowercase().as_str() {
        "image/png" => Ok(("png", "image/png")),
        "image/jpeg" | "image/jpg" => Ok(("jpg", "image/jpeg")),
        "image/webp" => Ok(("webp", "image/webp")),
        _ => Err(AppError::validation(
            "Logo files must be PNG, JPEG, or WebP images.",
        )),
    }
}

fn validate_magic(bytes: &[u8], mime: &str) -> Result<(), AppError> {
    let valid = match mime {
        "image/png" => bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
        "image/jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        "image/webp" => bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(AppError::validation(
            "The selected file content does not match its image type.",
        ))
    }
}
