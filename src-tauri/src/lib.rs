mod commands;
mod database;
mod models;

use database::{open_database, AppState};
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{image::Image, Manager, WindowEvent};

const TRAY_ICON_WIDTH: u32 = 124;
const TRAY_ICON_HEIGHT: u32 = 96;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                app.set_dock_visibility(false);
            }

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

            let data_dir = app.path().app_data_dir()?;
            let db_path = data_dir.join("promptpop.sqlite3");
            let conn = open_database(&db_path)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
            app.manage(AppState {
                conn: std::sync::Mutex::new(conn),
                db_path,
                data_dir,
            });

            #[cfg(desktop)]
            {
                let state = app.state::<AppState>();
                let _ = commands::register_saved_launcher_shortcut(app.handle(), state.inner());

                let tray_menu = MenuBuilder::new(app)
                    .text("promptpop-show", "显示 PromptPop")
                    .text("promptpop-settings", "打开设置")
                    .separator()
                    .text("promptpop-quit", "退出 PromptPop")
                    .build()?;
                let tray_icon = Image::new(
                    include_bytes!("../icons/tray-template.rgba"),
                    TRAY_ICON_WIDTH,
                    TRAY_ICON_HEIGHT,
                );
                let tray_builder = TrayIconBuilder::with_id("promptpop-tray")
                    .tooltip("PromptPop")
                    .menu(&tray_menu)
                    .icon(tray_icon)
                    .show_menu_on_left_click(false)
                    .icon_as_template(true)
                    .on_menu_event(|app, event| {
                        if event.id() == "promptpop-show" {
                            commands::show_launcher(app);
                        } else if event.id() == "promptpop-settings" {
                            commands::show_settings(app);
                        } else if event.id() == "promptpop-quit" {
                            app.exit(0);
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            commands::show_launcher(tray.app_handle());
                        }
                    });
                let tray = tray_builder.build(app)?;
                tray.set_visible(true)?;

                if let Some(window) = app.get_webview_window("main") {
                    let handle = app.handle().clone();
                    window.on_window_event(move |event| {
                        if let WindowEvent::CloseRequested { api, .. } = event {
                            api.prevent_close();
                            if let Some(window) = handle.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                    });
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_prompts,
            commands::restore_starter_snippets,
            commands::create_prompt,
            commands::update_prompt,
            commands::delete_prompt,
            commands::use_prompt,
            commands::list_tags,
            commands::upsert_tag,
            commands::get_setting,
            commands::set_setting,
            commands::app_environment,
            commands::register_launcher_shortcut,
            commands::set_launch_at_login,
            commands::hide_launcher,
            commands::open_settings_target,
            commands::configure_window_mode,
            commands::test_paste,
            commands::save_export,
            commands::backup_database,
            commands::copy_prompt,
            commands::paste_prompt
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
