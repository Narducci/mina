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
    *guard = None;

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

#[tauri::command]
fn escanear_pasta(path: String) -> Result<Vec<String>, String> {
    if path.is_empty() {
        return Ok(vec![]);
    }
    let dir = std::path::Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("Caminho não é um diretório: {}", path));
    }
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    let mut arquivos = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "pdf" | "png" | "jpg" | "jpeg") {
                    arquivos.push(p.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(arquivos)
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
        Migration {
            version: 5,
            description: "caminho_arquivo_em_comprovante",
            sql: "ALTER TABLE comprovante ADD COLUMN caminho_arquivo TEXT;",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "add nome_curto and seq_documento",
            sql: "ALTER TABLE comprovante ADD COLUMN nome_curto TEXT; \
                CREATE TABLE IF NOT EXISTS seq_documento ( \
                    id INTEGER PRIMARY KEY CHECK (id = 1), \
                    proximo INTEGER NOT NULL DEFAULT 1 \
                ); \
                INSERT OR IGNORE INTO seq_documento (id, proximo) VALUES (1, 1);",
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
        .invoke_handler(tauri::generate_handler![greet, iniciar_watcher, escanear_pasta])
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
