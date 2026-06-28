use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::UserDirs;
use rusqlite::{params, Connection};

use crate::{
    db::connection::open_database,
    models::BackupRow,
    utils::{
        audit::insert_audit_log,
        dates::{filename_timestamp, now_iso, today_date},
        errors::AppError,
    },
};

pub fn create_manual_backup(db_path: &Path, user_id: i64) -> Result<BackupRow, AppError> {
    create_backup(db_path, Some(user_id), "manual")
}

pub fn create_automatic_backup_if_due(db_path: &Path) -> Result<(), AppError> {
    let conn = open_database(db_path)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM backups
         WHERE backup_type = 'automatic'
           AND status = 'success'
           AND date(created_at) = date(?1)",
        [today_date()],
        |row| row.get(0),
    )?;
    drop(conn);
    if count == 0 {
        let _ = create_backup(db_path, None, "automatic");
    }
    Ok(())
}

pub fn restore_backup(db_path: &Path, user_id: i64, source_path: String) -> Result<(), AppError> {
    let source = PathBuf::from(source_path);
    if !source.exists() {
        return Err(AppError::restore_failed("Backup file does not exist."));
    }
    create_backup(db_path, Some(user_id), "emergency")?;
    fs::copy(&source, db_path)
        .map_err(|error| AppError::restore_failed(format!("Could not restore backup: {error}")))?;
    let conn = open_database(db_path)?;
    insert_backup_row(
        &conn,
        source.to_string_lossy().to_string(),
        "manual",
        "success",
        Some("Database restored from backup. Restart the app to reload all state.".to_string()),
    )?;
    insert_audit_log(
        &conn,
        user_id,
        "restore",
        "backups",
        0,
        None,
        Some(serde_json::json!({"source_path": source.to_string_lossy()})),
    )?;
    Ok(())
}

pub fn list_backups(conn: &Connection) -> Result<Vec<BackupRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, backup_path, backup_type, status, notes, created_at
         FROM backups
         ORDER BY created_at DESC, id DESC",
    )?;
    let rows = stmt
        .query_map([], map_backup)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn create_backup(db_path: &Path, user_id: Option<i64>, backup_type: &str) -> Result<BackupRow, AppError> {
    let conn = open_database(db_path)?;
    let backup_dir = backup_dir(&conn)?;
    fs::create_dir_all(&backup_dir)
        .map_err(|error| AppError::backup_failed(format!("Could not create backup folder: {error}")))?;
    let backup_path = backup_dir.join(format!(
        "steel_inventory_backup_{}.db",
        filename_timestamp()
    ));
    let backup_path_text = backup_path.to_string_lossy().to_string();
    let result = conn.execute("VACUUM INTO ?1", params![backup_path_text]);
    match result {
        Ok(_) => {
            let row = insert_backup_row(
                &conn,
                backup_path.to_string_lossy().to_string(),
                backup_type,
                "success",
                None,
            )?;
            if let Some(user_id) = user_id {
                insert_audit_log(
                    &conn,
                    user_id,
                    "create",
                    "backups",
                    row.id,
                    None,
                    Some(serde_json::to_value(&row).unwrap_or_default()),
                )?;
            }
            Ok(row)
        }
        Err(error) => {
            let _ = insert_backup_row(
                &conn,
                backup_path.to_string_lossy().to_string(),
                backup_type,
                "failed",
                Some(error.to_string()),
            );
            Err(AppError::backup_failed(error.to_string()))
        }
    }
}

fn backup_dir(conn: &Connection) -> Result<PathBuf, AppError> {
    let configured: Option<String> = conn.query_row(
        "SELECT backup_path FROM company_settings WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    if let Some(path) = configured.filter(|value| !value.trim().is_empty()) {
        return Ok(PathBuf::from(path));
    }
    let user_dirs = UserDirs::new()
        .ok_or_else(|| AppError::backup_failed("Could not resolve user directories."))?;
    let documents = user_dirs
        .document_dir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| user_dirs.home_dir().join("Documents"));
    Ok(documents.join("SteelInventoryBackups"))
}

fn insert_backup_row(
    conn: &Connection,
    backup_path: String,
    backup_type: &str,
    status: &str,
    notes: Option<String>,
) -> Result<BackupRow, AppError> {
    conn.execute(
        "INSERT INTO backups (backup_path, backup_type, status, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![backup_path, backup_type, status, notes, now_iso()],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, backup_path, backup_type, status, notes, created_at
         FROM backups WHERE id = ?1",
        [id],
        map_backup,
    )
    .map_err(Into::into)
}

fn map_backup(row: &rusqlite::Row<'_>) -> rusqlite::Result<BackupRow> {
    Ok(BackupRow {
        id: row.get(0)?,
        backup_path: row.get(1)?,
        backup_type: row.get(2)?,
        status: row.get(3)?,
        notes: row.get(4)?,
        created_at: row.get(5)?,
    })
}
