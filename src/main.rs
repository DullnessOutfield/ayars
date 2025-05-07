use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use walkdir;

#[derive(Debug, Serialize, Deserialize)]
struct KismetDevice {
    identifier: String,
    first_time: DateTime<Utc>,
    last_time: DateTime<Utc>,
    device_type: String,
    metadata: Value,
}

impl KismetDevice {
    fn from_row(row: &rusqlite::Row) -> Result<Self> {
        let first_time = row.get::<_, i64>(0)?;
        let last_time = row.get::<_, i64>(1)?;
        let mac = row.get::<_, String>(4)?;
        let device_type = row.get::<_, String>(13)?;
        let raw_json: Vec<u8> = row.get(14)?;
        let json_str = String::from_utf8_lossy(&raw_json);

        // Parse the JSON string
        let metadata: Value = serde_json::from_str(&json_str)
            .unwrap_or_else(|_| Value::Object(serde_json::Map::new()));

        Ok(KismetDevice {
            identifier: mac,
            first_time: DateTime::from_timestamp(first_time, 0).unwrap_or_default(),
            last_time: DateTime::from_timestamp(last_time, 0).unwrap_or_default(),
            device_type: device_type.trim().trim_matches('\'').to_string(),
            metadata,
        })
    }
}

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

fn get_access_points(db_path: &str) -> Result<Vec<KismetDevice>> {
    let device_types = Some(vec![
        String::from("Wi-Fi AP"),
        String::from("Wi-Fi Bridged"),
    ]);
    query_sqlite(db_path, device_types)
}

fn get_stas(db_path: &str) -> Result<Vec<KismetDevice>> {
    let device_types = Some(vec![
        String::from("Wi-Fi Client"),
        String::from("Wi-Fi Device"),
    ]);
    query_sqlite(db_path, device_types)
}

fn get_base_path() -> PathBuf {
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

fn main() {
    let basepath = get_base_path();

    match walkdir::WalkDir::new(basepath)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("kismet"))
    {
        iter => {
            for entry in iter {
                let path = entry.path().to_str().unwrap_or_default();
                println!("Processing: {}", path);
                
                match get_access_points(path) {
                    Ok(devices) => {
                        for device in devices {
                            println!("  {}", device.metadata );
                        }
                    }
                    Err(e) => eprintln!("Error processing {}: {}", path, e),
                }
            }
        }
    }
}
