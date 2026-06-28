use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn unauthorized() -> Self {
        Self::new("UNAUTHORIZED", "Please log in to continue.")
    }

    pub fn duplicate_sku() -> Self {
        Self::new("DUPLICATE_SKU", "A product with this SKU already exists.")
    }

    pub fn duplicate_invoice_number() -> Self {
        Self::new(
            "DUPLICATE_INVOICE_NUMBER",
            "An invoice with this number already exists.",
        )
    }

    pub fn insufficient_stock(message: impl Into<String>) -> Self {
        Self::new("INSUFFICIENT_STOCK", message)
    }

    pub fn backup_failed(message: impl Into<String>) -> Self {
        Self::new("BACKUP_FAILED", message)
    }

    pub fn restore_failed(message: impl Into<String>) -> Self {
        Self::new("RESTORE_FAILED", message)
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::new("DATABASE_ERROR", message)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        match value {
            rusqlite::Error::SqliteFailure(err, _) if err.code == rusqlite::ErrorCode::ConstraintViolation => {
                AppError::validation("A database constraint was violated.")
            }
            other => AppError::database(other.to_string()),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::database(value.to_string())
    }
}
