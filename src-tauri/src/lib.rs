use tauri::{Manager, Emitter};
use tauri_plugin_sql::{Migration, MigrationKind};
use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::Mutex;
use lopdf::{Document, Object, ObjectId, Dictionary};
use sha2::{Sha256, Digest};

struct WatcherState(Mutex<Option<notify::RecommendedWatcher>>);

// ── Helpers ───────────────────────────────────────────────────────────
fn hash_arquivo(caminho: &str) -> Result<String, String> {
    let bytes = std::fs::read(caminho).map_err(|e| e.to_string())?;
    let result = Sha256::digest(&bytes);
    Ok(result.iter().map(|b| format!("{:02x}", b)).collect())
}

// ── Struct de retorno do mesclar ──────────────────────────────────────
#[derive(serde::Serialize)]
struct MesclarResult {
    destino: String,
    hash: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn calcular_hash(caminho: String) -> Result<String, String> {
    hash_arquivo(&caminho)
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

/// Mescla fisicamente os PDFs em `caminhos`, salva em `destino` e
/// retorna o caminho e o SHA-256 do arquivo gerado.
#[tauri::command]
fn mesclar_pdfs(caminhos: Vec<String>, destino: String) -> Result<MesclarResult, String> {
    if caminhos.len() < 2 {
        return Err("Selecione ao menos dois arquivos para mesclar.".into());
    }

    let mut documents: Vec<Document> = caminhos
        .iter()
        .map(|p| -> Result<Document, String> {
            let mut doc = Document::load(p).map_err(|e| format!("Erro ao abrir '{}': {}", p, e))?;
            doc.decompress();
            Ok(doc)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut next_id: u32 = 1;
    for doc in &mut documents {
        doc.renumber_objects_with(next_id);
        next_id = doc.max_id + 1;
    }

    let mut merged = Document::with_version("1.5");
    let mut all_page_ids: Vec<ObjectId> = vec![];

    for doc in documents {
        let mut sorted_pages: Vec<(u32, ObjectId)> = doc.get_pages().into_iter().collect();
        sorted_pages.sort_by_key(|(n, _)| *n);
        for (_, pid) in sorted_pages {
            all_page_ids.push(pid);
        }
        merged.objects.extend(doc.objects);
    }

    let pages_id: ObjectId = (next_id, 0);
    next_id += 1;

    for &pid in &all_page_ids {
        if let Some(Object::Dictionary(ref mut dict)) = merged.objects.get_mut(&pid) {
            dict.set("Parent", Object::Reference(pages_id));
        }
    }

    let kids: Vec<Object> = all_page_ids
        .iter()
        .map(|&id| Object::Reference(id))
        .collect();

    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set("Kids", Object::Array(kids));
    pages_dict.set("Count", Object::Integer(all_page_ids.len() as i64));
    merged.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id: ObjectId = (next_id, 0);
    let mut catalog_dict = Dictionary::new();
    catalog_dict.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog_dict.set("Pages", Object::Reference(pages_id));
    merged.objects.insert(catalog_id, Object::Dictionary(catalog_dict));
    merged.max_id = next_id;

    merged.trailer.set("Root", Object::Reference(catalog_id));

    merged
        .save(&destino)
        .map_err(|e| format!("Erro ao salvar '{}': {}", destino, e))?;

    let hash = hash_arquivo(&destino)?;

    Ok(MesclarResult { destino, hash })
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
            description: "add_nome_curto_and_seq_documento",
            sql: "ALTER TABLE comprovante ADD COLUMN nome_curto TEXT; \
                CREATE TABLE IF NOT EXISTS seq_documento ( \
                    id INTEGER PRIMARY KEY CHECK (id = 1), \
                    proximo INTEGER NOT NULL DEFAULT 1 \
                ); \
                INSERT OR IGNORE INTO seq_documento (id, proximo) VALUES (1, 1);",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "mesclado_em_e_mesclado_em_id_em_comprovante",
            sql: "ALTER TABLE comprovante ADD COLUMN mesclado_em TEXT; \
                  ALTER TABLE comprovante ADD COLUMN mesclado_em_id INTEGER REFERENCES comprovante(id);",
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
        .invoke_handler(tauri::generate_handler![
            greet,
            iniciar_watcher,
            escanear_pasta,
            mesclar_pdfs,
            calcular_hash
        ])
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
