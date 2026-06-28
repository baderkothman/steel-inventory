pub fn payment_status(total_cents: i64, paid_cents: i64) -> String {
    if paid_cents <= 0 {
        "unpaid".to_string()
    } else if paid_cents >= total_cents {
        "paid".to_string()
    } else {
        "partial".to_string()
    }
}

pub fn checked_total(
    subtotal_cents: i64,
    discount_cents: i64,
    tax_cents: i64,
    extra_cents: i64,
) -> Result<i64, crate::utils::errors::AppError> {
    if discount_cents < 0 || tax_cents < 0 || extra_cents < 0 {
        return Err(crate::utils::errors::AppError::validation(
            "Discount, tax, shipping, and delivery values must be zero or greater.",
        ));
    }
    let total = subtotal_cents - discount_cents + tax_cents + extra_cents;
    if total < 0 {
        return Err(crate::utils::errors::AppError::validation(
            "Invoice total cannot be negative.",
        ));
    }
    Ok(total)
}
