use std::fs;
use std::path::{Path, PathBuf};

use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, Position, Runtime, Size, State,
    WebviewWindow,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::database;
use crate::database::AppState;
use crate::models::{
    AppEnvironment, Prompt, PromptInput, PromptUpdateInput, SavedFile, Setting, Tag,
};

#[cfg(target_os = "macos")]
const APP_IDENTIFIER: &str = "com.promptpop.desktop";
const DEFAULT_LAUNCHER_SHORTCUT: &str = "Alt+Space";
const LAUNCHER_WIDTH: f64 = 430.0;
const LAUNCHER_HEIGHT: f64 = 540.0;
const PEEK_WIDTH: f64 = 760.0;
const WORKSPACE_WIDTH: f64 = 1120.0;
const WORKSPACE_HEIGHT: f64 = 720.0;
const WORKSPACE_MIN_WIDTH: f64 = 720.0;
const WORKSPACE_MIN_HEIGHT: f64 = 560.0;

#[tauri::command]
pub fn list_prompts(state: State<'_, AppState>) -> Result<Vec<Prompt>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::list_prompts(&conn)
}

#[tauri::command]
pub fn restore_starter_snippets(
    state: State<'_, AppState>,
    locale: Option<String>,
) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    database::restore_starter_snippets(&conn, locale.as_deref())
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
        launch_at_login: launch_at_login_enabled(),
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
    set_launch_at_login_platform(enabled)?;
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
pub fn configure_window_mode(app: AppHandle, mode: String) -> Result<(), String> {
    configure_window_mode_inner(&app, &mode)
}

#[tauri::command]
pub fn open_settings_target(app: AppHandle, target: String) -> Result<(), String> {
    match target.as_str() {
        "accessibility" => open_accessibility_settings(),
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
    let _ = hide_launcher_window(&app);
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
    let _ = hide_launcher_window(&app);
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
    remember_paste_target();
    if let Some(window) = app.get_webview_window("main") {
        let _ = configure_window_mode_inner(app, "launcher");
        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit("promptpop:launcher-shortcut", ());
}

pub(crate) fn show_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = configure_window_mode_inner(app, "workspace");
        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit("promptpop:open-settings", ());
}

pub(crate) fn toggle_launcher(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return;
        }

        remember_paste_target();
        let _ = configure_window_mode_inner(app, "launcher");
        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit("promptpop:launcher-shortcut", ());
}

fn configure_window_mode_inner(app: &AppHandle, mode: &str) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    let (width, height, min_width, min_height, resizable) = match mode {
        "launcher" => (
            LAUNCHER_WIDTH,
            LAUNCHER_HEIGHT,
            LAUNCHER_WIDTH,
            LAUNCHER_HEIGHT,
            false,
        ),
        "peek" => (
            PEEK_WIDTH,
            LAUNCHER_HEIGHT,
            PEEK_WIDTH,
            LAUNCHER_HEIGHT,
            false,
        ),
        "workspace" => (
            WORKSPACE_WIDTH,
            WORKSPACE_HEIGHT,
            WORKSPACE_MIN_WIDTH,
            WORKSPACE_MIN_HEIGHT,
            true,
        ),
        _ => return Err(format!("Unknown window mode: {mode}")),
    };

    window
        .set_min_size(Some(Size::Logical(LogicalSize::new(min_width, min_height))))
        .map_err(|error| error.to_string())?;
    window
        .set_resizable(resizable)
        .map_err(|error| error.to_string())?;
    window
        .set_size(Size::Logical(LogicalSize::new(width, height)))
        .map_err(|error| error.to_string())?;
    set_center_position_for_inner_size(&window, width, height)?;
    Ok(())
}

fn set_center_position_for_inner_size<R: Runtime>(
    window: &WebviewWindow<R>,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let monitor = window
        .current_monitor()
        .map_err(|error| error.to_string())?
        .or(window
            .primary_monitor()
            .map_err(|error| error.to_string())?);

    let Some(monitor) = monitor else {
        return window.center().map_err(|error| error.to_string());
    };

    let scale_factor = monitor.scale_factor();
    let target_inner_width = (width * scale_factor).round() as i32;
    let target_inner_height = (height * scale_factor).round() as i32;
    let chrome_size = window
        .outer_size()
        .ok()
        .zip(window.inner_size().ok())
        .map(|(outer, inner)| {
            (
                outer.width.saturating_sub(inner.width) as i32,
                outer.height.saturating_sub(inner.height) as i32,
            )
        })
        .unwrap_or((0, 0));
    let target_outer_width = target_inner_width + chrome_size.0;
    let target_outer_height = target_inner_height + chrome_size.1;
    let work_area = monitor.work_area();
    let x = work_area.position.x + (work_area.size.width as i32 - target_outer_width) / 2;
    let y = work_area.position.y + (work_area.size.height as i32 - target_outer_height) / 2;

    window
        .set_position(Position::Physical(PhysicalPosition::new(x, y)))
        .map_err(|error| error.to_string())
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

fn hide_launcher_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return open_external(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        );
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Accessibility permission settings are only available on macOS".to_string())
    }
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

#[cfg(target_os = "macos")]
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

fn launch_at_login_enabled() -> bool {
    #[cfg(target_os = "macos")]
    {
        return launch_agent_path().exists();
    }

    #[cfg(target_os = "windows")]
    {
        return windows_launch_at_login::enabled();
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        false
    }
}

fn set_launch_at_login_platform(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return set_launch_agent(enabled);
    }

    #[cfg(target_os = "windows")]
    {
        return windows_launch_at_login::set_enabled(enabled, &app_launch_argument());
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        if enabled {
            Err("Launch at login is not supported on this platform".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "windows")]
mod windows_launch_at_login {
    use std::ffi::c_void;

    type Dword = u32;
    type Hkey = isize;
    type Long = i32;

    const ERROR_FILE_NOT_FOUND: Long = 2;
    const ERROR_SUCCESS: Long = 0;
    const HKEY_CURRENT_USER: Hkey = 0x80000001u32 as i32 as Hkey;
    const KEY_QUERY_VALUE: Dword = 0x0001;
    const KEY_SET_VALUE: Dword = 0x0002;
    const REG_OPTION_NON_VOLATILE: Dword = 0;
    const REG_SZ: Dword = 1;
    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "PromptPop";

    #[link(name = "Advapi32")]
    unsafe extern "system" {
        fn RegCloseKey(h_key: Hkey) -> Long;
        fn RegCreateKeyExW(
            h_key: Hkey,
            sub_key: *const u16,
            reserved: Dword,
            class: *mut u16,
            options: Dword,
            sam_desired: Dword,
            security_attributes: *mut c_void,
            result: *mut Hkey,
            disposition: *mut Dword,
        ) -> Long;
        fn RegDeleteValueW(h_key: Hkey, value_name: *const u16) -> Long;
        fn RegOpenKeyExW(
            h_key: Hkey,
            sub_key: *const u16,
            options: Dword,
            sam_desired: Dword,
            result: *mut Hkey,
        ) -> Long;
        fn RegQueryValueExW(
            h_key: Hkey,
            value_name: *const u16,
            reserved: *mut Dword,
            value_type: *mut Dword,
            data: *mut u8,
            data_size: *mut Dword,
        ) -> Long;
        fn RegSetValueExW(
            h_key: Hkey,
            value_name: *const u16,
            reserved: Dword,
            value_type: Dword,
            data: *const u8,
            data_size: Dword,
        ) -> Long;
    }

    struct RegistryKey(Hkey);

    impl Drop for RegistryKey {
        fn drop(&mut self) {
            let _ = unsafe { RegCloseKey(self.0) };
        }
    }

    pub fn enabled() -> bool {
        query_run_value().is_some_and(|value| !value.trim().is_empty())
    }

    pub fn set_enabled(enabled: bool, launch_argument: &str) -> Result<(), String> {
        if enabled {
            set_run_value(launch_argument)
        } else {
            delete_run_value()
        }
    }

    fn set_run_value(launch_argument: &str) -> Result<(), String> {
        let key = create_run_key()?;
        let value_name = wide_null(VALUE_NAME);
        let value = wide_null(&quote_launch_argument(launch_argument));
        let status = unsafe {
            RegSetValueExW(
                key.0,
                value_name.as_ptr(),
                0,
                REG_SZ,
                value.as_ptr().cast(),
                (value.len() * 2) as Dword,
            )
        };

        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("Unable to set Windows launch at login: {status}"))
        }
    }

    fn delete_run_value() -> Result<(), String> {
        let key = match open_run_key(KEY_SET_VALUE) {
            Ok(key) => key,
            Err(_) => return Ok(()),
        };
        let value_name = wide_null(VALUE_NAME);
        let status = unsafe { RegDeleteValueW(key.0, value_name.as_ptr()) };

        if matches!(status, ERROR_SUCCESS | ERROR_FILE_NOT_FOUND) {
            Ok(())
        } else {
            Err(format!(
                "Unable to remove Windows launch at login: {status}"
            ))
        }
    }

    fn query_run_value() -> Option<String> {
        let key = open_run_key(KEY_QUERY_VALUE).ok()?;
        let value_name = wide_null(VALUE_NAME);
        let mut value_type = 0;
        let mut byte_len = 0;
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                value_name.as_ptr(),
                std::ptr::null_mut(),
                &mut value_type,
                std::ptr::null_mut(),
                &mut byte_len,
            )
        };
        if status != ERROR_SUCCESS || value_type != REG_SZ || byte_len < 2 {
            return None;
        }

        let mut buffer = vec![0u16; ((byte_len as usize) + 1) / 2];
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                value_name.as_ptr(),
                std::ptr::null_mut(),
                &mut value_type,
                buffer.as_mut_ptr().cast(),
                &mut byte_len,
            )
        };
        if status != ERROR_SUCCESS || value_type != REG_SZ {
            return None;
        }

        let len = buffer
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(buffer.len());
        Some(String::from_utf16_lossy(&buffer[..len]))
    }

    fn create_run_key() -> Result<RegistryKey, String> {
        let sub_key = wide_null(RUN_KEY);
        let mut key = 0;
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                sub_key.as_ptr(),
                0,
                std::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                std::ptr::null_mut(),
                &mut key,
                std::ptr::null_mut(),
            )
        };

        if status == ERROR_SUCCESS {
            Ok(RegistryKey(key))
        } else {
            Err(format!("Unable to open Windows Run registry key: {status}"))
        }
    }

    fn open_run_key(access: Dword) -> Result<RegistryKey, String> {
        let sub_key = wide_null(RUN_KEY);
        let mut key = 0;
        let status =
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, sub_key.as_ptr(), 0, access, &mut key) };

        if status == ERROR_SUCCESS {
            Ok(RegistryKey(key))
        } else {
            Err(format!("Unable to open Windows Run registry key: {status}"))
        }
    }

    fn quote_launch_argument(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            trimmed.to_string()
        } else {
            format!("\"{}\"", trimmed.replace('"', "\\\""))
        }
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(target_os = "macos")]
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

    #[cfg(target_os = "windows")]
    {
        windows_paste::trigger_paste();
    }
}

fn remember_paste_target() {
    #[cfg(target_os = "windows")]
    {
        windows_paste::remember_foreground_window();
    }
}

#[cfg(target_os = "windows")]
mod windows_paste {
    use std::mem;
    use std::sync::{Mutex, OnceLock};
    use std::thread;
    use std::time::Duration;

    type Hwnd = isize;
    type Dword = u32;
    type Long = i32;
    type Word = u16;

    const INPUT_KEYBOARD: Dword = 1;
    const KEYEVENTF_KEYUP: Dword = 0x0002;
    const VK_CONTROL: Word = 0x11;
    const VK_MENU: Word = 0x12;
    const VK_V: Word = 0x56;
    const SW_RESTORE: i32 = 9;

    static PREVIOUS_FOREGROUND_WINDOW: OnceLock<Mutex<Option<Hwnd>>> = OnceLock::new();

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct Input {
        r#type: Dword,
        u: InputUnion,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    union InputUnion {
        mi: MouseInput,
        ki: KeybdInput,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct MouseInput {
        dx: Long,
        dy: Long,
        mouse_data: Dword,
        dw_flags: Dword,
        time: Dword,
        dw_extra_info: usize,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct KeybdInput {
        w_vk: Word,
        w_scan: Word,
        dw_flags: Dword,
        time: Dword,
        dw_extra_info: usize,
    }

    unsafe extern "system" {
        fn GetClassNameW(hwnd: Hwnd, class_name: *mut u16, max_count: i32) -> i32;
        fn GetCurrentProcessId() -> Dword;
        fn GetForegroundWindow() -> Hwnd;
        fn GetCurrentThreadId() -> Dword;
        fn GetWindowThreadProcessId(hwnd: Hwnd, process_id: *mut Dword) -> Dword;
        fn IsWindow(hwnd: Hwnd) -> i32;
        fn IsIconic(hwnd: Hwnd) -> i32;
        fn AttachThreadInput(id_attach: Dword, id_attach_to: Dword, attach: i32) -> i32;
        fn BringWindowToTop(hwnd: Hwnd) -> i32;
        fn SetFocus(hwnd: Hwnd) -> Hwnd;
        fn SetForegroundWindow(hwnd: Hwnd) -> i32;
        fn ShowWindow(hwnd: Hwnd, cmd_show: i32) -> i32;
        fn SendInput(c_inputs: u32, p_inputs: *const Input, cb_size: i32) -> u32;
    }

    pub fn remember_foreground_window() {
        let hwnd = unsafe { GetForegroundWindow() };
        let target = PREVIOUS_FOREGROUND_WINDOW.get_or_init(|| Mutex::new(None));
        if let Ok(mut saved) = target.lock() {
            *saved = if is_valid_paste_target(hwnd) {
                Some(hwnd)
            } else {
                None
            };
        }
    }

    pub fn trigger_paste() {
        if let Some(hwnd) = saved_foreground_window() {
            if unsafe { IsWindow(hwnd) } != 0 {
                restore_foreground_window(hwnd);
            }
        }

        thread::sleep(Duration::from_millis(180));
        send_key_chord(&[VK_CONTROL], VK_V);
    }

    fn is_valid_paste_target(hwnd: Hwnd) -> bool {
        if hwnd == 0 || unsafe { IsWindow(hwnd) } == 0 {
            return false;
        }

        let mut process_id = 0;
        let _ = unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };
        if process_id == unsafe { GetCurrentProcessId() } {
            return false;
        }

        !matches!(
            window_class_name(hwnd).as_deref(),
            Some("Shell_TrayWnd" | "Shell_SecondaryTrayWnd" | "NotifyIconOverflowWindow")
        )
    }

    fn window_class_name(hwnd: Hwnd) -> Option<String> {
        let mut buffer = [0u16; 256];
        let length = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
        if length <= 0 {
            return None;
        }

        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    }

    fn restore_foreground_window(hwnd: Hwnd) {
        if unsafe { IsIconic(hwnd) } != 0 {
            let _ = unsafe { ShowWindow(hwnd, SW_RESTORE) };
        }

        // A short Alt press lets this process legally hand foreground back to
        // the target under Windows foreground-lock rules.
        tap_key(VK_MENU);

        let current_thread = unsafe { GetCurrentThreadId() };
        let target_thread = unsafe { GetWindowThreadProcessId(hwnd, std::ptr::null_mut()) };
        let attached = target_thread != 0
            && target_thread != current_thread
            && unsafe { AttachThreadInput(current_thread, target_thread, 1) } != 0;

        let _ = unsafe { BringWindowToTop(hwnd) };
        let _ = unsafe { SetForegroundWindow(hwnd) };
        let _ = unsafe { SetFocus(hwnd) };

        if attached {
            let _ = unsafe { AttachThreadInput(current_thread, target_thread, 0) };
        }

        thread::sleep(Duration::from_millis(180));
    }

    fn send_key_chord(modifiers: &[Word], key: Word) {
        let mut inputs = Vec::with_capacity((modifiers.len() * 2) + 2);
        for modifier in modifiers {
            inputs.push(key_input(*modifier, 0));
        }
        inputs.push(key_input(key, 0));
        inputs.push(key_input(key, KEYEVENTF_KEYUP));
        for modifier in modifiers.iter().rev() {
            inputs.push(key_input(*modifier, KEYEVENTF_KEYUP));
        }

        let _ = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                mem::size_of::<Input>() as i32,
            )
        };
    }

    fn tap_key(key: Word) {
        let inputs = [key_input(key, 0), key_input(key, KEYEVENTF_KEYUP)];
        let _ = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                mem::size_of::<Input>() as i32,
            )
        };
    }

    fn saved_foreground_window() -> Option<Hwnd> {
        PREVIOUS_FOREGROUND_WINDOW
            .get()
            .and_then(|target| target.lock().ok().and_then(|saved| *saved))
    }

    fn key_input(vk: Word, flags: Dword) -> Input {
        Input {
            r#type: INPUT_KEYBOARD,
            u: InputUnion {
                ki: KeybdInput {
                    w_vk: vk,
                    w_scan: 0,
                    dw_flags: flags,
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        }
    }
}
