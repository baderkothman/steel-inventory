use crate::models::ProductPayload;

pub fn generate_sku_from_product(payload: &ProductPayload) -> String {
    let finish = abbreviation(&payload.finish, 2);
    let material = abbreviation(&payload.material, 1);
    let product_type = abbreviation(&payload.product_type, 1);
    let shape = shape_code(&payload.shape);
    let mut parts = vec![format!("{finish}{material}{product_type}"), shape];

    if !payload.size_label.trim().is_empty() {
        parts.push(payload.size_label.trim().replace(' ', "").to_uppercase());
    }

    if let Some(thickness) = payload.thickness_mm {
        parts.push(trim_float(thickness));
    }

    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Builds a supplier-independent specification key used to group the same
/// physical product type across suppliers (e.g. "round pipe 2 inch 2 mm").
/// Must stay in sync with the spec_key backfill in migration 003.
pub fn spec_key_from_product(payload: &ProductPayload) -> String {
    // Format matches SQLite CAST(thickness_mm AS TEXT) used by migration 003's backfill,
    // so legacy and newly-created rows for the same spec produce an identical key.
    let thickness = payload
        .thickness_mm
        .map(sqlite_real_text)
        .unwrap_or_default();
    format!(
        "{}|{}|{}|{}|{}|{}",
        payload.product_type.trim(),
        payload.material.trim(),
        payload.shape.trim(),
        payload.finish.trim(),
        payload.size_label.trim(),
        thickness
    )
    .to_uppercase()
}

fn abbreviation(value: &str, max: usize) -> String {
    value
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(max)
        .collect::<String>()
        .to_uppercase()
}

fn shape_code(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "square" => "SQ".to_string(),
        "rectangular" | "rectangle" => "REC".to_string(),
        "round" => "RD".to_string(),
        "flat" => "FL".to_string(),
        "angle" => "ANG".to_string(),
        "channel" => "CH".to_string(),
        "beam" => "BM".to_string(),
        other => abbreviation(other, 2),
    }
}

/// Mirrors SQLite's textual rendering of a REAL value (e.g. 2.0 -> "2.0", 2.5 -> "2.5").
fn sqlite_real_text(value: f64) -> String {
    if value == value.trunc() {
        format!("{value:.1}")
    } else {
        let text = format!("{value}");
        text
    }
}

fn trim_float(value: f64) -> String {
    let text = format!("{value:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}
