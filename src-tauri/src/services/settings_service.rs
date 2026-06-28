use rusqlite::{params, Connection};

use crate::{
    models::{CompanySettings, CompanySettingsPayload},
    utils::{
        audit::insert_audit_log,
        dates::now_iso,
        errors::AppError,
        validation::{required},
    },
};

pub fn get_company_settings(conn: &Connection) -> Result<CompanySettings, AppError> {
    conn.query_row(
        "SELECT id, company_name, phone, email, address, tax_number, default_currency,
                invoice_prefix_sales, invoice_prefix_purchase, allow_negative_stock,
                backup_path, default_tax_rate, default_profit_method, created_at, updated_at
         FROM company_settings
         WHERE id = 1",
        [],
        map_settings,
    )
    .map_err(Into::into)
}

pub fn update_company_settings(
    conn: &Connection,
    user_id: i64,
    payload: CompanySettingsPayload,
) -> Result<CompanySettings, AppError> {
    required(&payload.company_name, "Company name")?;
    required(&payload.default_currency, "Default currency")?;
    required(&payload.invoice_prefix_sales, "Sales invoice prefix")?;
    required(&payload.invoice_prefix_purchase, "Purchase invoice prefix")?;
    if payload.default_tax_rate < 0.0 {
        return Err(AppError::validation("Default tax value must be zero or greater."));
    }

    let now = now_iso();
    conn.execute(
        "UPDATE company_settings
         SET company_name = ?1, phone = ?2, email = ?3, address = ?4, tax_number = ?5,
             default_currency = ?6, invoice_prefix_sales = ?7, invoice_prefix_purchase = ?8,
             allow_negative_stock = ?9, backup_path = ?10, default_tax_rate = ?11,
             default_profit_method = ?12, updated_at = ?13
         WHERE id = 1",
        params![
            payload.company_name.trim(),
            payload.phone,
            payload.email,
            payload.address,
            payload.tax_number,
            payload.default_currency.trim().to_uppercase(),
            payload.invoice_prefix_sales.trim().to_uppercase(),
            payload.invoice_prefix_purchase.trim().to_uppercase(),
            if payload.allow_negative_stock { 1 } else { 0 },
            payload.backup_path,
            payload.default_tax_rate,
            payload.default_profit_method,
            now
        ],
    )?;
    let settings = get_company_settings(conn)?;
    insert_audit_log(
        conn,
        user_id,
        "update",
        "company_settings",
        1,
        None,
        Some(serde_json::to_value(&settings).unwrap_or_default()),
    )?;
    Ok(settings)
}

fn map_settings(row: &rusqlite::Row<'_>) -> rusqlite::Result<CompanySettings> {
    Ok(CompanySettings {
        id: row.get(0)?,
        company_name: row.get(1)?,
        phone: row.get(2)?,
        email: row.get(3)?,
        address: row.get(4)?,
        tax_number: row.get(5)?,
        default_currency: row.get(6)?,
        invoice_prefix_sales: row.get(7)?,
        invoice_prefix_purchase: row.get(8)?,
        allow_negative_stock: row.get::<_, i64>(9)? == 1,
        backup_path: row.get(10)?,
        default_tax_rate: row.get(11)?,
        default_profit_method: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}
