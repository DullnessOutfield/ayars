pub mod kismet_device;
use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use kismet_device::KismetDevice;

fn query_sqlite(db_path: &str, device_types: Option<Vec<String>>) -> Result<Vec<KismetDevice>> {
    let conn = Connection::open(db_path)?;
    let query = match device_types {
        Some(types) => {
            let types_str: Vec<String> = types.iter().map(|t| format!("'{}'", t)).collect();
            format!(
                "SELECT * FROM devices WHERE type IN ({})",
                types_str.join(", ")
            )
        }
        None => String::from("SELECT * FROM devices"),
    };
    let mut stmt = conn.prepare(&query)?;
    let devices = stmt.query_map([], |row| KismetDevice::from_row(row))?;
    devices.collect()
}

pub fn get_access_points(db_path: &str) -> Result<Vec<KismetDevice>> {
    let device_types = Some(vec![
        String::from("Wi-Fi AP"),
        String::from("Wi-Fi Bridged"),
    ]);
    query_sqlite(db_path, device_types)
}

pub fn get_stas(db_path: &str) -> Result<Vec<KismetDevice>> {
    let device_types = Some(vec![
        String::from("Wi-Fi Client"),
        String::from("Wi-Fi Device"),
    ]);
    query_sqlite(db_path, device_types)
}

pub fn get_base_path() -> PathBuf {
    // Try to read from pathconfig.txt in current directory
    if let Ok(content) = fs::read_to_string("./pathconfig.txt") {
        if let Some(configured_path) = content.lines().next() {
            let path = Path::new(configured_path);
            if path.exists() {
                return path.to_path_buf();
            }
        }
    }

    // Fall back to ~/Data/
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE")).unwrap_or_default();
    PathBuf::from(home).join("Data")
}