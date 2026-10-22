
use serde::{Deserialize, Serialize};
use winreg::enums::{self, RegType as WRegType};
use winreg::RegKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Hive {
    Hklm,
    Hkcu,
}

impl Hive {
    fn predef(self) -> RegKey {
        RegKey::predef(match self {
            Hive::Hklm => enums::HKEY_LOCAL_MACHINE,
            Hive::Hkcu => enums::HKEY_CURRENT_USER,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Value {
    Dword(u32),
    String(String),
    Qword(u64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "value")]
pub enum Action {
    Set(Value),
    Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Addr {
    pub hive: Hive,
    pub key: String,
    #[serde(default)]
    pub name: String,
}

impl Addr {
    pub fn display(&self) -> String {
        let root = match self.hive {
            Hive::Hklm => "HKLM",
            Hive::Hkcu => "HKCU",
        };
        if self.name.is_empty() {
            format!("{root}\\{}", self.key)
        } else {
            format!("{root}\\{} : {}", self.key, self.name)
        }
    }
}

pub struct Change {
    pub addr: Addr,
    pub action: Action,
}

fn read_point(addr: &Addr) -> Result<Option<Value>, String> {
    let access = readable_access(addr.hive);
    let key = match addr.hive.predef().open_subkey_with_flags(&addr.key, access) {
        Ok(key) => key,
        Err(e) if is_absent_error(&e) => return Ok(None),
        Err(e) => return Err(format!("open {}: {e}", addr.display())),
    };

    let raw = key.get_raw_value(&addr.name).ok();
    match raw {
        Some(raw) => Some(decode_value(raw.vtype, &raw.bytes))
            .transpose()
            .map(|v| v.flatten()),
        None if addr.name.is_empty() => Ok(None),
        None => Ok(None),
    }
    .map_err(|e: String| format!("decode {}: {e}", addr.display()))
}

fn is_absent_error(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::NotFound
        || matches!(e.raw_os_error(), Some(2) | Some(3))
}

fn readable_access(hive: Hive) -> u32 {
    let base = enums::KEY_READ;
    match hive {
        Hive::Hklm => base | enums::KEY_WOW64_64KEY,
        Hive::Hkcu => base,
    }
}

fn decode_value(vtype: WRegType, bytes: &[u8]) -> Result<Option<Value>, String> {
    match vtype {
        WRegType::REG_DWORD if bytes.len() == 4 => {
            Ok(Some(Value::Dword(u32::from_le_bytes(bytes.try_into().unwrap()))))
        }
        WRegType::REG_QWORD if bytes.len() == 8 => {
            Ok(Some(Value::Qword(u64::from_le_bytes(bytes.try_into().unwrap()))))
        }
        WRegType::REG_SZ | WRegType::REG_EXPAND_SZ => {
            let units: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|&u| u != 0)
                .collect();
            Ok(Some(Value::String(String::from_utf16_lossy(&units))))
        }
        _ => Ok(None),
    }
}

fn encode_value(value: &Value) -> Result<winreg::RegValue, String> {
    use winreg::RegValue;
    Ok(match value {
        Value::Dword(v) => RegValue { vtype: WRegType::REG_DWORD, bytes: v.to_le_bytes().to_vec() },
        Value::Qword(v) => RegValue { vtype: WRegType::REG_QWORD, bytes: v.to_le_bytes().to_vec() },
        Value::String(s) => {
            let mut units: Vec<u16> = s.encode_utf16().collect();
            units.push(0);
            let bytes: Vec<u8> = units.iter().flat_map(|u| u.to_le_bytes()).collect();
            RegValue { vtype: WRegType::REG_SZ, bytes }
        }
    })
}

pub fn read_state(addr: &Addr) -> Result<Option<Value>, String> {
    read_point(addr)
}

pub fn matches(action: &Action, addr: &Addr) -> Result<bool, String> {
    let current = read_state(addr)?;
    Ok(match (action, current) {
        (Action::Set(target), Some(cur)) => *target == cur,
        (Action::Delete, None) => true,
        _ => false,
    })
}

pub fn apply(change: &Change) -> Result<(), String> {
    let Change { addr, action } = change;
    let hive = RegKey::predef(match addr.hive {
        Hive::Hklm => enums::HKEY_LOCAL_MACHINE,
        Hive::Hkcu => enums::HKEY_CURRENT_USER,
    });
    let (key, _disposition) = hive
        .create_subkey(&addr.key)
        .map_err(|e| format!("create {}: {e}", addr.display()))?;

    match action {
        Action::Set(value) => {
            let raw = encode_value(value)?;
            key.set_raw_value(&addr.name, &raw)
                .map_err(|e| format!("set {}: {e}", addr.display()))
        }
        Action::Delete => match key.delete_value(&addr.name) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("delete {}: {e}", addr.display())),
        },
    }
}
