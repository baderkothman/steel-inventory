use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use directories::ProjectDirs;
use rusqlite::Connection;

use crate::{
    db::{connection::open_database, migrations::run_migrations},
    models::AdminUser,
    utils::errors::AppError,
};

#[derive(Debug, Clone)]
pub struct Session {
    pub user: AdminUser,
}

pub struct AppState {
    pub db_path: PathBuf,
    session: Mutex<Option<Session>>,
}

impl AppState {
    pub fn initialize() -> Result<Self, AppError> {
        let db_path = database_path()?;
        let mut conn = open_database(&db_path)?;
        run_migrations(&mut conn)?;
        Ok(Self {
            db_path,
            session: Mutex::new(None),
        })
    }

    pub fn open_conn(&self) -> Result<Connection, AppError> {
        open_database(&self.db_path)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn set_session(&self, user: AdminUser) -> Result<(), AppError> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| AppError::database("Could not lock session state."))?;
        *session = Some(Session { user });
        Ok(())
    }

    pub fn clear_session(&self) -> Result<(), AppError> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| AppError::database("Could not lock session state."))?;
        *session = None;
        Ok(())
    }

    pub fn current_user(&self) -> Result<Option<AdminUser>, AppError> {
        let session = self
            .session
            .lock()
            .map_err(|_| AppError::database("Could not lock session state."))?;
        Ok(session.as_ref().map(|s| s.user.clone()))
    }

    pub fn require_user(&self) -> Result<AdminUser, AppError> {
        self.current_user()?.ok_or_else(AppError::unauthorized)
    }

    pub fn require_user_id(&self) -> Result<i64, AppError> {
        Ok(self.require_user()?.id)
    }
}

fn database_path() -> Result<PathBuf, AppError> {
    let project_dirs = ProjectDirs::from("com", "local", "SteelInventory")
        .ok_or_else(|| AppError::database("Could not resolve the application data directory."))?;
    Ok(project_dirs.data_local_dir().join("steel_inventory.db"))
}
