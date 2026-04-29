use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::database;
use crate::database::AppState;
use crate::models::{
    AppEnvironment, Prompt, PromptInput, PromptUpdateInput, SavedFile, Setting, Tag,
};

const APP_IDENTIFIER: &str = "com.promptpop.desktop";
const DEFAULT_LAUNCHER_SHORTCUT: &str = "Alt+Space";

#[tauri::command]
pub fn list_prompts(state: State<'_, AppState>) -> Result<Vec<Prompt>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::list_prompts(&conn)
}

#[tauri::command]
pub fn create_prompt(state: State<'_, AppState>, input: PromptInput) -> Result<Prompt, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::create_prompt(&conn, input)
}

#[tauri::command]
pub fn update_prompt(
    state: State<'_, AppState>,
    input: PromptUpdateInput,
) -> Result<Prompt, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::update_prompt(&conn, input)
}

#[tauri::command]
pub fn delete_prompt(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::delete_prompt(&conn, &id)
}

#[tauri::command]
pub fn use_prompt(state: State<'_, AppState>, id: String) -> Result<Prompt, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::mark_used(&conn, &id)
}

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::list_tags(&conn)
}

#[tauri::command]
pub fn upsert_tag(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::upsert_tag(&conn, name, color)
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<Setting>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::get_setting(&conn, &key)
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<Setting, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::set_setting(&conn, key, value)
}

#[tauri::command]
pub fn app_environment(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppEnvironment, String> {
    let data_dir = state.data_dir.clone();
    let logs_dir = logs_dir(&app)?;
    let exports_dir = data_dir.join("exports");
    let backups_dir = data_dir.join("backups");

    ensure_dir(&data_dir)?;
    ensure_dir(&logs_dir)?;
    ensure_dir(&exports_dir)?;
    ensure_dir(&backups_dir)?;

    Ok(AppEnvironment {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        tauri_version: "2".to_string(),
        database_path: display_path(&state.db_path),
        data_dir: display_path(&data_dir),
        logs_dir: display_path(&logs_dir),
        exports_dir: display_path(&exports_dir),
        backups_dir: display_path(&backups_dir),
        launch_at_login: launch_agent_path().exists(),
        accessibility_trusted: accessibility_trusted(),
    })
}

#[tauri::command]
pub fn register_launcher_shortcut(
    app: AppHandle,
    state: State<'_, AppState>,
    shortcut: String,
) -> Result<String, String> {
    let shortcut = clean_shortcut(&shortcut);
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let previous_shortcut = database::get_setting(&conn, "shortcuts.globalLauncher")?
        .map(|setting| setting.value)
        .unwrap_or_else(|| DEFAULT_LAUNCHER_SHORTCUT.to_string());
    drop(conn);

    if let Err(error) = register_launcher_shortcut_inner(&app, &shortcut) {
        let _ = register_launcher_shortcut_inner(&app, &previous_shortcut);
        return Err(error);
    }

    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::set_setting(
        &conn,
        "shortcuts.globalLauncher".to_string(),
        shortcut.clone(),
    )?;
    Ok(shortcut)
}

#[tauri::command]
pub fn set_launch_at_login(state: State<'_, AppState>, enabled: bool) -> Result<bool, String> {
    set_launch_agent(enabled)?;
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::set_setting(&conn, "launchAtLogin".to_string(), enabled.to_string())?;
    Ok(enabled)
}

#[tauri::command]
pub fn hide_launcher(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_settings_target(app: AppHandle, target: String) -> Result<(), String> {
    match target.as_str() {
        "accessibility" => open_external(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        ),
        "data" => open_path(
            app.path()
                .app_data_dir()
                .map_err(|error| error.to_string())?,
        ),
        "logs" => open_path(logs_dir(&app)?),
        "exports" => open_path(
            app.path()
                .app_data_dir()
                .map_err(|error| error.to_string())?
                .join("exports"),
        ),
        "backups" => open_path(
            app.path()
                .app_data_dir()
                .map_err(|error| error.to_string())?
                .join("backups"),
        ),
        _ => Err("Unknown settings target".to_string()),
    }
}

#[tauri::command]
pub fn test_paste(app: AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(format!("PromptPop paste test {}", database::now()))
        .map_err(|error| error.to_string())?;
    trigger_paste();
    Ok(())
}

#[tauri::command]
pub fn save_export(app: AppHandle, format: String, contents: String) -> Result<SavedFile, String> {
    let extension = match format.as_str() {
        "json" => "json",
        "markdown" => "md",
        _ => return Err("Unsupported export format".to_string()),
    };

    let exports_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("exports");
    ensure_dir(&exports_dir)?;

    let filename = format!(
        "promptpop-export-{}.{}",
        database::now().replace([':', '.'], "-"),
        extension
    );
    let path = exports_dir.join(filename);
    fs::write(&path, contents).map_err(|error| error.to_string())?;
    saved_file(path)
}

#[tauri::command]
pub fn backup_database(state: State<'_, AppState>) -> Result<SavedFile, String> {
    let backup_dir = state.data_dir.join("backups");
    ensure_dir(&backup_dir)?;

    let backup_path = backup_dir.join(format!(
        "promptpop-backup-{}.sqlite3",
        database::now().replace([':', '.'], "-")
    ));
    fs::copy(&state.db_path, &backup_path).map_err(|error| error.to_string())?;
    saved_file(backup_path)
}

#[tauri::command]
pub fn copy_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Prompt, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let prompt = database::mark_used(&conn, &id)?;
    app.clipboard()
        .write_text(prompt.body.clone())
        .map_err(|error| error.to_string())?;
    Ok(prompt)
}

#[tauri::command]
pub fn paste_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Prompt, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let prompt = database::mark_used(&conn, &id)?;
    app.clipboard()
        .write_text(prompt.body.clone())
        .map_err(|error| error.to_string())?;
    trigger_paste();
    Ok(prompt)
}

pub(crate) fn register_launcher_shortcut_inner(
    app: &AppHandle,
    shortcut: &str,
) -> Result<(), String> {
    let shortcut = clean_shortcut(shortcut);
    app.global_shortcut()
        .unregister_all()
        .map_err(|error| error.to_string())?;
    app.global_shortcut()
        .on_shortcut(shortcut.as_str(), |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle_launcher(app);
            }
        })
        .map_err(|error| error.to_string())
}

pub(crate) fn register_saved_launcher_shortcut(
    app: &AppHandle,
    state: &AppState,
) -> Result<String, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let shortcut = database::get_setting(&conn, "shortcuts.globalLauncher")?
        .map(|setting| setting.value)
        .unwrap_or_else(|| DEFAULT_LAUNCHER_SHORTCUT.to_string());
    drop(conn);

    match register_launcher_shortcut_inner(app, &shortcut) {
        Ok(()) => Ok(shortcut),
        Err(_) => {
            register_launcher_shortcut_inner(app, DEFAULT_LAUNCHER_SHORTCUT)?;
            Ok(DEFAULT_LAUNCHER_SHORTCUT.to_string())
        }
    }
}

pub(crate) fn show_launcher(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit("promptpop:launcher-shortcut", ());
}

pub(crate) fn toggle_launcher(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return;
        }

        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit("promptpop:launcher-shortcut", ());
}

fn clean_shortcut(shortcut: &str) -> String {
    let trimmed = shortcut.trim();
    if trimmed.is_empty() {
        DEFAULT_LAUNCHER_SHORTCUT.to_string()
    } else {
        trimmed.to_string()
    }
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| error.to_string())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn saved_file(path: PathBuf) -> Result<SavedFile, String> {
    let bytes = fs::metadata(&path)
        .map_err(|error| error.to_string())?
        .len();
    Ok(SavedFile {
        path: display_path(&path),
        bytes,
    })
}

fn logs_dir(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home).join("Library/Logs/PromptPop"));
        }
    }

    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("logs"))
}

fn open_path(path: PathBuf) -> Result<(), String> {
    ensure_dir(&path)?;
    open_external(&display_path(&path))
}

fn open_external(target: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = std::process::Command::new("open");
        command.arg(target);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = std::process::Command::new("cmd");
        command.args(["/C", "start", "", target]);
        command
    };

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut command = {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(target);
        command
    };

    command.status().map_err(|error| error.to_string())?;
    Ok(())
}

fn launch_agent_path() -> PathBuf {
    let home = std::env::var_os("HOME").unwrap_or_default();
    PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{APP_IDENTIFIER}.plist"))
}

fn app_launch_argument() -> String {
    if let Ok(exe) = std::env::current_exe() {
        let mut current = exe.as_path();
        while let Some(parent) = current.parent() {
            if parent
                .extension()
                .is_some_and(|extension| extension == "app")
            {
                return parent.to_string_lossy().into_owned();
            }
            current = parent;
        }
        return exe.to_string_lossy().into_owned();
    }

    "/Applications/PromptPop.app".to_string()
}

fn set_launch_agent(enabled: bool) -> Result<(), String> {
    let path = launch_agent_path();
    if !enabled {
        if path.exists() {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let launch_argument = app_launch_argument();
    let plist = if launch_argument.ends_with(".app") {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{APP_IDENTIFIER}</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/bin/open</string>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
"#,
            escape_plist(&launch_argument)
        )
    } else {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{APP_IDENTIFIER}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
"#,
            escape_plist(&launch_argument)
        )
    };

    fs::write(path, plist).map_err(|error| error.to_string())
}

fn escape_plist(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn accessibility_trusted() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get UI elements enabled")
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
        return Some(text.contains("true"));
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn trigger_paste() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to keystroke \"v\" using command down")
            .status();
    }
}
