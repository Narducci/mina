use tauri::{Manager, Emitter};
use tauri_plugin_sql::{Migration, MigrationKind};
use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::Mutex;

struct WatcherState(Mutex<Option<notify::RecommendedWatcher>>);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn iniciar_watcher(
    path: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<WatcherState>,
) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = None; // para e descarta o watcher anterior

    if path.is_empty() {
        return Ok(());
    }

    let handle = app_handle.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Create(_)) {
                    for p in &event.paths {
                        let _ = handle.emit(
                            "comprovante:novo",
                            p.to_string_lossy().to_string(),
                        );
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;

    watcher
        .watch(std::path::Path::new(&path), RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    *guard = Some(watcher);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "schema_inicial",
            sql: include_str!("../schema.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "conta_ativa_default_0_lancamento_conta_id",
            sql: include_str!("../schema_v2.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "categoria_tipo_sistema_e_categorias_fixas",
            sql: include_str!("../schema_v3.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "remove_unique_lancamento_periodo_tipo_ordem",
            sql: include_str!("../schema_v4.sql"),
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .manage(WatcherState(Mutex::new(None)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:mina.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![greet, iniciar_watcher])
        .setup(|app| {
            if let Some(monitor) = app.primary_monitor()? {
                let size = monitor.size();
                let w = (size.width as f64 * 0.8) as u32;
                let h = (size.height as f64 * 0.8) as u32;
                let x = ((size.width as f64 - w as f64) / 2.0) as i32;
                let y = ((size.height as f64 - h as f64) / 2.0) as i32;
                if let Some(window) = app.get_webview_window("main") {
                    window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: w,
                        height: h,
                    }))?;
                    window.set_position(tauri::Position::Physical(
                        tauri::PhysicalPosition { x, y },
                    ))?;
                    window.show()?;
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}