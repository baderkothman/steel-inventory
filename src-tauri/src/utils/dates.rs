use chrono::{Local, NaiveDate};

pub fn now_iso() -> String {
    Local::now().to_rfc3339()
}

pub fn today_date() -> String {
    Local::now().date_naive().to_string()
}

pub fn filename_timestamp() -> String {
    Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
}

pub fn validate_date(value: &str, field: &str) -> Result<(), crate::utils::errors::AppError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| crate::utils::errors::AppError::validation(format!("{field} must be a valid YYYY-MM-DD date.")))
}
