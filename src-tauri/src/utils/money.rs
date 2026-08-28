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
    let total = subtotal_cents
        .checked_sub(discount_cents)
        .and_then(|value| value.checked_add(tax_cents))
        .and_then(|value| value.checked_add(extra_cents))
        .ok_or_else(|| {
            crate::utils::errors::AppError::validation("Document total is too large.")
        })?;
    if total < 0 {
        return Err(crate::utils::errors::AppError::validation(
            "Invoice total cannot be negative.",
        ));
    }
    Ok(total)
}
