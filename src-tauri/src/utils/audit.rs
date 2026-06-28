use rusqlite::{params, Connection};
use serde_json::Value;

use crate::utils::{dates::now_iso, errors::AppError};

pub fn insert_audit_log(
    conn: &Connection,
    user_id: i64,
    action: &str,
    table_name: &str,
    record_id: i64,
    old_value: Option<Value>,
    new_value: Option<Value>,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO audit_logs
         (user_id, action, table_name, record_id, old_value_json, new_value_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            user_id,
            action,
            table_name,
            record_id,
            old_value.map(|v| v.to_string()),
            new_value.map(|v| v.to_string()),
            now_iso()
        ],
    )?;
    Ok(())
}
