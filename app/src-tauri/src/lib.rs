
mod registry;
mod snapshot;
mod tweaks;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::Manager;

use registry::{Action, Value};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const POWERSHELL: &str = "powershell";

#[derive(Serialize)]
pub struct SystemInfo {
    pub is_admin: bool,
    pub windows_product: String,
    pub display_version: String,
    pub build_number: String,
    pub arch: String,
}

#[tauri::command]
fn get_system_info() -> SystemInfo {
    let (windows_product, display_version, build_number) = read_os_version();
    SystemInfo {
        is_admin: is_admin(),
        windows_product,
        display_version,
        build_number,
        arch: std::env::consts::ARCH.to_string(),
    }
}

fn is_admin() -> bool {
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0
    }
    #[cfg(not(windows))]
    false
}

fn read_os_version() -> (String, String, String) {
    winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            winreg::enums::KEY_READ,
        )
        .map(|k| {
            (
                k.get_value("ProductName").unwrap_or_default(),
                k.get_value("DisplayVersion").unwrap_or_default(),
                k.get_value("CurrentBuildNumber").unwrap_or_default(),
            )
        })
        .unwrap_or_default()
}

#[derive(Serialize)]
pub struct TweakDto {
    pub id: &'static str,
    pub group: String,
    pub title: tweaks::Loc,
    pub description: tweaks::Loc,
    pub details: Vec<String>,
    pub requires_admin: bool,
    pub win11_only: bool,
}

#[tauri::command]
fn list_tweaks() -> Vec<TweakDto> {
    tweaks::catalog()
        .into_iter()
        .map(|t| TweakDto {
            id: t.id,
            group: t.group.as_str().to_string(),
            title: t.title,
            description: t.description,
            details: t.details,
            requires_admin: t.requires_admin,
            win11_only: t.win11_only,
        })
        .collect()
}

#[tauri::command]
fn get_tweaks_state(ids: Vec<String>) -> Result<HashMap<String, bool>, String> {
    let mut state = HashMap::new();
    for tweak in tweaks::catalog() {
        if !ids.iter().any(|id| id == tweak.id) {
            continue;
        }
        let mut applied = true;
        for c in &tweak.changes {
            if !registry::matches(&c.action, &c.addr)? {
                applied = false;
                break;
            }
        }
        state.insert(tweak.id.to_string(), applied);
    }
    Ok(state)
}

#[derive(Serialize)]
pub struct TweakResult {
    pub id: String,
    pub ok: bool,
    pub error: Option<String>,
}

fn find_tweaks(ids: &[String]) -> Vec<tweaks::Tweak> {
    tweaks::catalog()
        .into_iter()
        .filter(|t| ids.iter().any(|id| id == t.id))
        .collect()
}

fn snapshot_id() -> String {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("snap-{ms}")
}

#[tauri::command]
fn apply_tweaks(app: tauri::AppHandle, ids: Vec<String>) -> Result<Vec<TweakResult>, String> {
    let dir = snapshot::snapshot_dir(&app.path().app_data_dir().map_err(|e| e.to_string())?);

    let addrs: Vec<_> = find_tweaks(&ids)
        .iter()
        .flat_map(|t| t.changes.iter().map(|c| c.addr.clone()))
        .collect();
    let label = format!("apply: {}", ids.join(", "));
    snapshot::take(snapshot_id(), label, &addrs, &dir)?;

    let mut results = Vec::new();
    for tweak in find_tweaks(&ids) {
        let mut error = None;
        for change in &tweak.changes {
            if let Err(e) = registry::apply(change) {
                error = Some(e);
                break;
            }
        }
        results.push(TweakResult {
            id: tweak.id.to_string(),
            ok: error.is_none(),
            error,
        });
    }
    Ok(results)
}

#[tauri::command]
fn revert_tweaks(ids: Vec<String>) -> Result<Vec<TweakResult>, String> {
    let mut results = Vec::new();
    for tweak in find_tweaks(&ids) {
        let mut error = None;
        for change in &tweak.changes {
            let undo = match &change.action {
                Action::Set(_) => registry::Change {
                    addr: change.addr.clone(),
                    action: Action::Delete,
                },
                Action::Delete => registry::Change {
                    addr: change.addr.clone(),
                    action: Action::Set(Value::Dword(1)),
                },
            };
            if let Err(e) = registry::apply(&undo) {
                error = Some(e);
                break;
            }
        }
        results.push(TweakResult {
            id: tweak.id.to_string(),
            ok: error.is_none(),
            error,
        });
    }
    Ok(results)
}

#[tauri::command]
fn list_snapshots(app: tauri::AppHandle) -> Result<Vec<(String, u64, String, usize)>, String> {
    let dir = snapshot::snapshot_dir(&app.path().app_data_dir().map_err(|e| e.to_string())?);
    snapshot::list(&dir)
}

#[tauri::command]
fn restore_snapshot(app: tauri::AppHandle, id: Option<String>) -> Result<usize, String> {
    let dir = snapshot::snapshot_dir(&app.path().app_data_dir().map_err(|e| e.to_string())?);
    snapshot::restore(&dir, id)
}

fn spawn_quiet(cmd: &str, args: &[&str]) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new(cmd)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("expected an http(s) url".into());
    }
    spawn_quiet("cmd", &["/C", "start", "", &url]).map_err(|e| e.to_string())
}

#[tauri::command]
fn restart_explorer() -> Result<(), String> {
    let _ = spawn_quiet("taskkill", &["/F", "/IM", "explorer.exe"]);
    spawn_quiet("cmd", &["/C", "start", "explorer.exe"])
        .map_err(|e| format!("start explorer: {e}"))
}

async fn run_blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    match tauri::async_runtime::spawn_blocking(f).await {
        Ok(res) => res,
        Err(e) => Err(format!("join: {e}")),
    }
}

fn run_quiet(cmd: &str, args: &[&str]) -> Result<i32, String> {
    use std::os::windows::process::CommandExt;
    let out = std::process::Command::new(cmd)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    Ok(out.status.code().unwrap_or(-1))
}

#[tauri::command]
async fn create_restore_point() -> Result<String, String> {
    run_blocking(move || {
        let code = run_quiet(
            POWERSHELL,
            &[
                "-NoProfile",
                "-Command",
                "Enable-ComputerRestore -Drive 'C:'; Checkpoint-Computer -Description 'NullTweaker' -RestorePointType 'MODIFY_SETTINGS'",
            ],
        )?;
        if code == 0 {
            Ok("created".into())
        } else {
            Err(format!("Checkpoint-Computer: exit {code}"))
        }
    })
    .await
}

#[tauri::command]
async fn empty_recycle_bin() -> Result<String, String> {
    run_blocking(move || {
        let code = run_quiet(
            POWERSHELL,
            &["-NoProfile", "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"],
        )?;
        if code == 0 {
            Ok("cleared".into())
        } else {
            Err(format!("Clear-RecycleBin: exit {code}"))
        }
    })
    .await
}

#[tauri::command]
async fn clear_temp() -> Result<u64, String> {
    run_blocking(move || {
        let mut dirs = Vec::<PathBuf>::new();
        if let Ok(t) = std::env::var("TEMP") {
            dirs.push(PathBuf::from(t));
        }
        if let Ok(windir) = std::env::var("WINDIR") {
            let sys = PathBuf::from(windir).join("Temp");
            if sys.exists() {
                dirs.push(sys);
            }
        }

        let mut freed = 0u64;
        for dir in dirs {
            if dir.exists() {
                freed += wipe_dir(&dir);
            }
        }
        Ok(freed)
    })
    .await
}

fn wipe_dir(dir: &Path) -> u64 {
    let mut freed = 0u64;
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            freed += wipe_dir(&path);
            let _ = std::fs::remove_dir(&path);
        } else if std::fs::remove_file(&path).is_ok() {
            freed += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    freed
}

#[tauri::command]
async fn run_sfc() -> Result<i32, String> {
    run_blocking(move || run_quiet("cmd", &["/C", "sfc", "/scannow"])).await
}

#[tauri::command]
async fn run_dism() -> Result<i32, String> {
    run_blocking(move || run_quiet("DISM", &["/Online", "/Cleanup-Image", "/ScanHealth"])).await
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UwpApp {
    pub name: String,
    pub full_name: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct UwpResult {
    pub full_name: String,
    pub ok: bool,
    pub error: Option<String>,
}

fn ps(script: &str) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    let out = std::process::Command::new(POWERSHELL)
        .args(["-NoProfile", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("powershell: {e}"))?;

    if out.status.code() != Some(0) {
        return Err(format!(
            "powershell exit {}: {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    if text.starts_with('\u{feff}') {
        text = text[1..].to_string();
    }
    Ok(text)
}

const JUNK: &[&str] = &[
    "microsoft.549981c3f5f10",
    "copilot",
    "msteams",
    "microsoftteams",
    "microsoft.bing",
    "microsoft.news",
    "microsoft.gamingapp",
    "microsoft.gethelp",
    "microsoft.getstarted",
    "microsoft.tips",
    "microsoft.todos",
    "microsoft.microsoftofficehub",
    "microsoft.microsoftstickynotes",
    "microsoft.whiteboard",
    "microsoft.outlookforwindows",
    "microsoft.microsoft3dviewer",
    "microsoft.print3d",
    "microsoft.mixedreality",
    "microsoft.skypeapp",
    "microsoft.yourphone",
    "microsoft.people",
    "microsoft.linkedin",
    "microsoft.windowscommunicationsapps",
    "microsoft.zunemusic",
    "microsoft.zunevideo",
    "microsoft.clipchamp",
    "microsoft.windowsmaps",
    "microsoft.powerautomatedesktop",
    "microsoft.family",
    "microsoftwindows.client.webexperience",
    "king.com.",
];

fn is_junk(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    JUNK.iter().any(|p| n.contains(p))
}

fn parse_app(v: serde_json::Value) -> Option<UwpApp> {
    Some(UwpApp {
        name: v.get("Name")?.as_str()?.to_string(),
        full_name: v.get("PackageFullName")?.as_str()?.to_string(),
        version: v.get("Version").and_then(|x| x.as_str()).unwrap_or("").to_string(),
    })
}

#[tauri::command]
async fn list_uwp() -> Result<Vec<UwpApp>, String> {
    run_blocking(move || {
        let text = ps(
            r#"[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; Get-AppxPackage | Where-Object { -not $_.IsFramework -and $_.NonRemovable -ne $true } | Where-Object { $_.PackageFullName } | Select-Object Name, PackageFullName, @{n='Version';e={$_.Version.ToString()}} | ConvertTo-Json -Compress -Depth 2"#,
        )?;
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("json: {e}"))?;
        let mut apps = match value {
            serde_json::Value::Array(items) => items.into_iter().filter_map(parse_app).collect(),
            other => parse_app(other).into_iter().collect::<Vec<_>>(),
        };
        apps.retain(|a| is_junk(&a.name));
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(apps)
    })
    .await
}

#[tauri::command]
async fn remove_uwp(full_names: Vec<String>) -> Result<Vec<UwpResult>, String> {
    if full_names.is_empty() {
        return Ok(Vec::new());
    }
    run_blocking(move || {
        let mut results = Vec::new();
        for full in &full_names {
            let script = format!(
                "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; Remove-AppxPackage -Package '{}'",
                full.replace('\'', "''")
            );
            results.push(match ps(&script) {
                Ok(_) => UwpResult { full_name: full.clone(), ok: true, error: None },
                Err(e) => UwpResult { full_name: full.clone(), ok: false, error: Some(e) },
            });
        }
        Ok(results)
    })
    .await
}

const ULTIMATE_GUID: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";
const BALANCED_GUID: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
const PLAN_NAME: &str = "NullTweaker Ultimate";

fn guid_tokens(text: &str) -> Vec<String> {
    fn looks_guid(t: &str) -> bool {
        let b = t.as_bytes();
        b.len() == 36
            && [8, 13, 18, 23].iter().all(|&i| b[i] == b'-')
            && b.iter().all(|c| c.is_ascii_hexdigit() || *c == b'-')
    }

    let mut out = Vec::new();
    let mut run = String::new();
    for ch in text.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_hexdigit() || ch == '-' {
            run.push(ch);
        } else {
            let tok = std::mem::take(&mut run);
            if looks_guid(&tok) {
                out.push(tok.to_ascii_lowercase());
            }
        }
    }
    out
}

fn powercfg(args: &[&str]) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    let out = std::process::Command::new("powercfg")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("powercfg: {e}"))?;
    if out.status.code() != Some(0) {
        return Err(format!(
            "powercfg {}: exit {}",
            args.join(" "),
            out.status.code().unwrap_or(-1)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn our_plan_guid() -> Option<String> {
    let list = powercfg(&["/list"]).ok()?;
    list.lines()
        .find(|line| line.contains(PLAN_NAME))
        .and_then(|line| guid_tokens(line).into_iter().next())
}

#[tauri::command]
fn get_max_power_state() -> bool {
    let Some(plan) = our_plan_guid() else { return false };
    let Some(active) = powercfg(&["/getactivescheme"]).ok().and_then(|s| guid_tokens(&s).into_iter().next())
    else {
        return false;
    };
    active == plan
}

#[tauri::command]
async fn set_max_power(enabled: bool) -> Result<(), String> {
    run_blocking(move || {
        if !enabled {
            let _ = powercfg(&["/setactive", BALANCED_GUID]);
            return Ok(());
        }

        let plan = match our_plan_guid() {
            Some(guid) => guid,
            None => {
                let out = powercfg(&["/duplicatescheme", ULTIMATE_GUID])?;
                let guid = guid_tokens(&out)
                    .into_iter()
                    .next()
                    .ok_or("powercfg: no guid in output")?;
                let _ = powercfg(&["/changesetname", &guid, PLAN_NAME]);
                guid
            }
        };
        powercfg(&["/setactive", &plan]).map(|_| ())
    })
    .await
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            list_tweaks,
            get_tweaks_state,
            apply_tweaks,
            revert_tweaks,
            list_snapshots,
            restore_snapshot,
            restart_explorer,
            create_restore_point,
            empty_recycle_bin,
            clear_temp,
            run_sfc,
            run_dism,
            list_uwp,
            remove_uwp,
            get_max_power_state,
            set_max_power,
            open_url,
        ])
        .run(tauri::generate_context!())
        .expect("NullTweaker failed to start");
}
