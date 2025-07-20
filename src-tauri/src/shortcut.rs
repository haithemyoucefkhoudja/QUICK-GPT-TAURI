use anyhow::Result;
use tauri::App;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;
use tauri::Runtime;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_global_shortcut::Shortcut;
use tauri_plugin_global_shortcut::ShortcutState;
use tauri_plugin_store::JsonValue;
use tauri_plugin_store::StoreExt;

/// Name of the Tauri storage
pub const COCO_TAURI_STORE: &str = "coco_tauri_store";

/// Key for storing global shortcuts
pub const COCO_GLOBAL_SHORTCUT: &str = "coco_global_shortcut";
pub const COCO_SCREENSHOT_SHORTCUT: &str = "coco_screenshot_shortcut";

/// Default shortcut for macOS
#[cfg(target_os = "macos")]
const DEFAULT_SHORTCUT: &str = "command+shift+Y";
#[cfg(target_os = "macos")]
const DEFAULT_SCREENSHOT_SHORTCUT: &str = "command+shift+S";

/// Default shortcut for Windows and Linux
#[cfg(any(target_os = "windows", target_os = "linux"))]
const DEFAULT_SHORTCUT: &str = "ctrl+shift+Y";
#[cfg(any(target_os = "windows", target_os = "linux"))]
const DEFAULT_SCREENSHOT_SHORTCUT: &str = "ctrl+shift+S";

/// Set shortcut during application startup
pub fn enable_shortcut(app: &App) {
    let store = app
        .store(COCO_TAURI_STORE)
        .expect("Creating the store should not fail");

    // Main shortcut
    let main_shortcut = if let Some(stored_shortcut) = store.get(COCO_GLOBAL_SHORTCUT) {
        let stored_shortcut_str = match stored_shortcut {
            JsonValue::String(str) => str,
            unexpected_type => panic!(
                "COCO shortcuts should be stored as strings, found type: {} ",
                unexpected_type
            ),
        };
        stored_shortcut_str
            .parse::<Shortcut>()
            .expect("Stored shortcut string should be valid")
    } else {
        // This is the original, correct logic.
        // The `set` method might return a Result<(), Error> which can be ignored
        // with a semicolon if we don't need to handle the failure case.
        let _ = store.set(
            COCO_GLOBAL_SHORTCUT,
            JsonValue::String(DEFAULT_SHORTCUT.to_string()),
        );
        DEFAULT_SHORTCUT
            .parse::<Shortcut>()
            .expect("Default shortcut should be valid")
    };

    // Screenshot shortcut
    let screenshot_shortcut = if let Some(stored_shortcut) = store.get(COCO_SCREENSHOT_SHORTCUT) {
        let stored_shortcut_str = match stored_shortcut {
            JsonValue::String(str) => str,
            unexpected_type => panic!(
                "COCO shortcuts should be stored as strings, found type: {} ",
                unexpected_type
            ),
        };
        stored_shortcut_str
            .parse::<Shortcut>()
            .expect("Stored shortcut string should be valid")
    } else {
        // This is the original, correct logic.
        let _ = store.set(
            COCO_SCREENSHOT_SHORTCUT,
            JsonValue::String(DEFAULT_SCREENSHOT_SHORTCUT.to_string()),
        );
        DEFAULT_SCREENSHOT_SHORTCUT
            .parse::<Shortcut>()
            .expect("Default shortcut should be valid")
    };

    // Now, register both shortcuts with a single plugin instance
    let app_handle = app.handle();
    let main_shortcut_clone = main_shortcut.clone();
    let screenshot_shortcut_clone = screenshot_shortcut.clone();

    // The only change is here, to handle the potential registration error without crashing.
    let result = app_handle.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_shortcuts([main_shortcut, screenshot_shortcut])
            .expect("Failed to add shortcuts to builder")
            .with_handler(move |app, scut, event| {
                println!(
                    "[Handler] Shortcut event for {:?}, state: {:?}",
                    scut,
                    event.state()
                );
                if let ShortcutState::Pressed = event.state() {
                    if scut == &main_shortcut_clone {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap() {
                                // window.set_always_on_top(false).unwrap();
                                // window.hide().unwrap();
                            } else {
                                if let Ok(position) = app.cursor_position() {
                                    let shift_x = -100.0;
                                    let shift_y = -100.0;
                                    window
                                        .set_position(tauri::PhysicalPosition::new(
                                            position.x + shift_x,
                                            position.y + shift_y,
                                        ))
                                        .unwrap();
                                }
                                window.show().unwrap();
                                window.set_focus().unwrap();
                                window.set_always_on_top(true).unwrap();
                            }
                        } else {
                            eprintln!("main window not found when shortcut was pressed!");
                        }
                    } else if scut == &screenshot_shortcut_clone {
                        app.emit("internal_take_screenshot", ()).unwrap();
                    }
                }
            })
            .build(),
    );

    if let Err(e) = result {
        // This will print the "HotKey already registered" error to the console
        // but will NOT crash the application.
        eprintln!("Failed to register global shortcut plugin: {}", e);
    }
}

/// Get the current stored shortcut as a string
#[tauri::command]
pub fn get_current_shortcut<R: Runtime>(
    app: AppHandle<R>,
    window_label: String,
) -> Result<String, String> {
    let shortcut = _get_shortcut(&app, &window_label);
    Ok(shortcut)
}

/// Unregister the current shortcut in Tauri
#[tauri::command]
pub fn unregister_shortcut<R: Runtime>(app: AppHandle<R>, window_label: String) {
    let shortcut_str = _get_shortcut(&app, &window_label);

    if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
        if let Err(e) = app.global_shortcut().unregister(shortcut) {
            eprintln!(
                "Warning: could not unregister shortcut '{}'. This is often normal: {}",
                shortcut_str, e
            );
        }
    } else {
        if !shortcut_str.is_empty() {
            eprintln!(
                "Warning: could not parse stored shortcut string to unregister: '{}'",
                shortcut_str
            );
        }
    }
}

/// Change the global shortcut
#[tauri::command]
pub fn change_shortcut<R: Runtime>(
    app: AppHandle<R>,
    _window: tauri::Window<R>,
    key: String,
    window_label: String,
) -> Result<(), String> {
    println!("[change_shortcut] Key: {}, Window: {}", key, window_label);
    let shortcut = match key.parse::<Shortcut>() {
        Ok(shortcut) => shortcut,
        Err(_) => return Err(format!("Invalid shortcut {}", key)),
    };

    let store = app
        .get_store(COCO_TAURI_STORE)
        .expect("Store should already be loaded or created");

    let shortcut_key = if window_label == "main" {
        COCO_GLOBAL_SHORTCUT
    } else {
        COCO_SCREENSHOT_SHORTCUT
    };

    store.set(shortcut_key, JsonValue::String(key));

    _register_shortcut(&app, shortcut, &window_label);

    Ok(())
}

/// Helper function to register a shortcut, primarily for updating shortcuts
fn _register_shortcut<R: Runtime>(
    app: &AppHandle<R>,
    shortcut: Shortcut,
    window_label: &str,
) -> Result<()> {
    let window_label = window_label.to_string();
    app.global_shortcut()
        .on_shortcut(shortcut, move |app_handle, _scut, event| {
            if let ShortcutState::Pressed = event.state() {
                if window_label == "screenshot" {
                    app_handle.emit("internal_take_screenshot", ()).unwrap();
                } else if let Some(window) = app_handle.get_webview_window(&window_label) {
                    if window.is_visible().unwrap_or(false) {
                        // window
                        //     .set_always_on_top(false)
                        //     .expect("Failed to set always on top");
                        // window.hide().expect("Failed to hide window");
                    } else {
                        if window_label == "main" {
                            if let Ok(position) = app_handle.cursor_position() {
                                let shift_x = -100.0;
                                let shift_y = -100.0;
                                window
                                    .set_position(tauri::PhysicalPosition::new(
                                        position.x + shift_x,
                                        position.y + shift_y,
                                    ))
                                    .unwrap();
                            }
                        }
                        window.show().expect("Failed to show window");
                        window.set_focus().expect("Failed to focus window");
                        window.set_always_on_top(true).unwrap();
                    }
                } else {
                    eprintln!(
                        "{} window not found when shortcut was pressed!",
                        window_label
                    );
                }
            }
        })?;

    Ok(())
}

/// Retrieve the stored global shortcut as a string
pub fn _get_shortcut<R: Runtime>(app: &AppHandle<R>, window_label: &str) -> String {
    let store = app
        .get_store(COCO_TAURI_STORE)
        .expect("Store should already be loaded or created");

    let shortcut_key = if window_label == "main" {
        COCO_GLOBAL_SHORTCUT
    } else {
        COCO_SCREENSHOT_SHORTCUT
    };

    match store
        .get(shortcut_key)
        .expect("Shortcut should already be stored")
    {
        JsonValue::String(str) => str,
        unexpected_type => panic!(
            "COCO shortcuts should be stored as strings, found type: {} ",
            unexpected_type
        ),
    }
}
