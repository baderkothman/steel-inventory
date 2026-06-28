use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rusqlite::{params, Connection};

use crate::{
    models::{AdminUser, ChangePasswordPayload, LoginPayload, SetupAdminPayload},
    utils::{
        audit::insert_audit_log,
        dates::now_iso,
        errors::AppError,
        validation::{required},
    },
};

pub fn has_admin(conn: &Connection) -> Result<bool, AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users WHERE is_active = 1", [], |row| {
        row.get(0)
    })?;
    Ok(count > 0)
}

pub fn setup_admin(conn: &Connection, payload: SetupAdminPayload) -> Result<AdminUser, AppError> {
    required(&payload.full_name, "Full name")?;
    required(&payload.email, "Email")?;
    validate_password(&payload.password)?;

    if has_admin(conn)? {
        return Err(AppError::validation("An admin account already exists."));
    }

    let now = now_iso();
    let password_hash = hash_password(&payload.password)?;
    conn.execute(
        "INSERT INTO users (full_name, email, password_hash, role, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'admin', 1, ?4, ?4)",
        params![
            payload.full_name.trim(),
            payload.email.trim().to_lowercase(),
            password_hash,
            now
        ],
    )?;
    let id = conn.last_insert_rowid();
    let user = AdminUser {
        id,
        full_name: payload.full_name.trim().to_string(),
        email: payload.email.trim().to_lowercase(),
        role: "admin".to_string(),
    };
    insert_audit_log(
        conn,
        id,
        "create",
        "users",
        id,
        None,
        Some(serde_json::json!({"id": id, "email": user.email})),
    )?;
    Ok(user)
}

pub fn login_admin(conn: &Connection, payload: LoginPayload) -> Result<AdminUser, AppError> {
    required(&payload.email, "Email")?;
    required(&payload.password, "Password or PIN")?;

    let mut stmt = conn.prepare(
        "SELECT id, full_name, email, password_hash, role
         FROM users
         WHERE lower(email) = lower(?1) AND is_active = 1
         LIMIT 1",
    )?;
    let row = stmt.query_row([payload.email.trim()], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    });

    let (id, full_name, email, password_hash, role) = match row {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(AppError::unauthorized());
        }
        Err(error) => return Err(error.into()),
    };

    verify_password(&payload.password, &password_hash)?;
    insert_audit_log(
        conn,
        id,
        "login",
        "users",
        id,
        None,
        Some(serde_json::json!({"id": id, "email": email})),
    )?;

    Ok(AdminUser {
        id,
        full_name,
        email,
        role,
    })
}

pub fn change_password(
    conn: &Connection,
    user_id: i64,
    payload: ChangePasswordPayload,
) -> Result<(), AppError> {
    required(&payload.current_password, "Current password")?;
    validate_password(&payload.new_password)?;

    let current_hash: String = conn.query_row(
        "SELECT password_hash FROM users WHERE id = ?1 AND is_active = 1",
        [user_id],
        |row| row.get(0),
    )?;
    verify_password(&payload.current_password, &current_hash)?;
    let new_hash = hash_password(&payload.new_password)?;
    conn.execute(
        "UPDATE users SET password_hash = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_hash, now_iso(), user_id],
    )?;
    insert_audit_log(
        conn,
        user_id,
        "update",
        "users",
        user_id,
        None,
        Some(serde_json::json!({"password_changed": true})),
    )?;
    Ok(())
}

fn validate_password(value: &str) -> Result<(), AppError> {
    required(value, "Password or PIN")?;
    if value.chars().count() < 4 {
        return Err(AppError::validation(
            "Password or PIN must contain at least 4 characters.",
        ));
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AppError::database("Could not hash password."))
}

fn verify_password(password: &str, encoded_hash: &str) -> Result<(), AppError> {
    let parsed_hash = PasswordHash::new(encoded_hash)
        .map_err(|_| AppError::database("Stored password hash is invalid."))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::unauthorized())
}
