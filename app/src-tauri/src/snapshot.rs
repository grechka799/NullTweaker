
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::registry::{self, Addr, Hive, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPoint {
    pub hive: Hive,
    pub key: String,
    #[serde(default)]
    pub name: String,
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_unix: u64,
    pub label: String,
    pub points: Vec<SavedPoint>,
}

pub fn snapshot_dir(home: &PathBuf) -> PathBuf {
    home.join("snapshots")
}

fn file_name(id: &str) -> String {
    format!("{id}.json")
}

fn save(dir: &PathBuf, snap: &Snapshot) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("create snapshot dir: {e}"))?;
    let path = dir.join(file_name(&snap.id));
    let json = serde_json::to_string_pretty(snap).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| format!("write snapshot: {e}"))
}

pub fn load(dir: &PathBuf, id: &str) -> Result<Snapshot, String> {
    let path = dir.join(file_name(id));
    let text = fs::read_to_string(path).map_err(|e| format!("read snapshot: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("parse snapshot: {e}"))
}

pub fn list(dir: &PathBuf) -> Result<Vec<(String, u64, String, usize)>, String> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    let entries = fs::read_dir(dir).map_err(|e| format!("list snapshots: {e}"))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if !name.ends_with(".json") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).map_err(|e| format!("read {}: {e}", name))?;
        if let Ok(snap) = serde_json::from_str::<Snapshot>(&text) {
            out.push((snap.id, snap.created_unix, snap.label, snap.points.len()));
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(out)
}

pub fn take(id: String, label: String, addrs: &[Addr], dir: &PathBuf) -> Result<String, String> {
    let mut points = Vec::new();
    for addr in addrs {
        let value = registry::read_state(addr)?;
        points.push(SavedPoint {
            hive: addr.hive,
            key: addr.key.clone(),
            name: addr.name.clone(),
            value,
        });
    }
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let snap = Snapshot { id: id.clone(), created_unix: created, label, points };
    save(dir, &snap)?;
    Ok(id)
}

pub fn restore(dir: &PathBuf, id: Option<String>) -> Result<usize, String> {
    let id = match id {
        Some(id) => id,
        None => {
            let all = list(dir)?;
            let first = all.first().ok_or("no snapshots")?;
            first.0.clone()
        }
    };
    let snap = load(dir, &id)?;
    let mut restored = 0;
    for point in &snap.points {
        let addr = Addr {
            hive: point.hive,
            key: point.key.clone(),
            name: point.name.clone(),
        };
        let change = match &point.value {
            Some(value) => crate::registry::Change {
                addr,
                action: crate::registry::Action::Set(value.clone()),
            },
            None => crate::registry::Change {
                addr,
                action: crate::registry::Action::Delete,
            },
        };
        registry::apply(&change)?;
        restored += 1;
    }
    Ok(restored)
}
