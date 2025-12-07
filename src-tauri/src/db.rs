use rusqlite::{Connection, Result};
use std::path::PathBuf;
use tauri::AppHandle;

pub fn get_db_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".claudius").join("claudius.db")
}

pub fn init_database(_app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)?;

    // Create tables
    conn.execute_batch(include_str!("schema.sql"))?;

    Ok(())
}

pub fn get_connection() -> Result<Connection> {
    Connection::open(get_db_path())
}
