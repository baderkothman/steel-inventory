use crate::utils::errors::AppError;

pub fn required(value: &str, field: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        Err(AppError::validation(format!("{field} is required.")))
    } else {
        Ok(())
    }
}

pub fn non_negative_i64(value: i64, field: &str) -> Result<(), AppError> {
    if value < 0 {
        Err(AppError::validation(format!("{field} must be zero or greater.")))
    } else {
        Ok(())
    }
}

pub fn positive_i64(value: i64, field: &str) -> Result<(), AppError> {
    if value <= 0 {
        Err(AppError::validation(format!("{field} must be greater than zero.")))
    } else {
        Ok(())
    }
}

pub fn positive_f64(value: f64, field: &str) -> Result<(), AppError> {
    if value <= 0.0 {
        Err(AppError::validation(format!("{field} must be greater than zero.")))
    } else {
        Ok(())
    }
}

pub fn optional_positive(value: Option<f64>, field: &str) -> Result<(), AppError> {
    if let Some(number) = value {
        if number <= 0.0 {
            return Err(AppError::validation(format!("{field} must be positive when provided.")));
        }
    }
    Ok(())
}
