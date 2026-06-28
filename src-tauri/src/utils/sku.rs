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

fn trim_float(value: f64) -> String {
    let text = format!("{value:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}
