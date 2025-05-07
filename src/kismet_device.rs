use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use rusqlite::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct KismetDevice {
    identifier: String,
    first_time: DateTime<Utc>,
    last_time: DateTime<Utc>,
    device_type: String,
    metadata: Value,
}

impl KismetDevice {
    pub fn from_row(row: &rusqlite::Row) -> Result<Self> {
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

    pub fn probed_ssids(&self) -> Vec<String> {

        if let Value::Object(probe_map) = &self.metadata["dot11.device"]["dot11.device.probed_ssid_map"] {
            probe_map
                .values()
                .filter_map(|entry| {
                    entry["dot11.probedssid.ssid"]
                        .as_str()
                        .filter(|ssid| !ssid.is_empty())
                        .map(String::from)
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}