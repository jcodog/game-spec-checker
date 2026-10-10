#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use gamespec_core::*;
use std::{path::PathBuf, sync::Mutex};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
struct ProfileStore {
    directory: PathBuf,
    lock: Mutex<()>,
}
struct CatalogStore {
    path: PathBuf,
    catalog: Mutex<catalog::Catalog>,
    load_error: Mutex<Option<String>>,
}
#[tauri::command]
async fn scan_system() -> Result<SystemSnapshot, String> {
    tauri::async_runtime::spawn_blocking(scan)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn list_profiles(
    store: tauri::State<'_, ProfileStore>,
) -> Result<profiles::ProfileLibrary, String> {
    let _guard = store.lock.lock().map_err(|e| e.to_string())?;
    profiles::load(&store.directory)
}
#[tauri::command]
fn parse_profile(json: String) -> Result<GameProfile, String> {
    profiles::parse(&json)
}
#[tauri::command]
fn blank_profile(id: String) -> Result<GameProfile, String> {
    if !profiles::valid_id(&id) {
        return Err("Invalid profile ID.".into());
    }
    Ok(profiles::new_profile(id))
}
#[tauri::command]
fn save_profile(
    json: String,
    store: tauri::State<'_, ProfileStore>,
) -> Result<GameProfile, String> {
    let p = profiles::parse(&json)?;
    let _guard = store.lock.lock().map_err(|e| e.to_string())?;
    profiles::save(&store.directory, &p)?;
    Ok(p)
}
#[tauri::command]
fn remove_profile(id: String, store: tauri::State<'_, ProfileStore>) -> Result<(), String> {
    if !profiles::valid_id(&id) {
        return Err("Invalid profile ID.".into());
    }
    let _guard = store.lock.lock().map_err(|e| e.to_string())?;
    std::fs::remove_file(store.directory.join(format!("{id}.json"))).map_err(|e| e.to_string())
}
#[tauri::command]
fn catalog_info(store: tauri::State<'_, CatalogStore>) -> Result<catalog::CatalogSummary, String> {
    let mut summary = store.catalog.lock().map_err(|e| e.to_string())?.summary();
    if let Some(error) = &*store.load_error.lock().map_err(|e| e.to_string())? {
        summary.diagnostics.push(error.clone());
    }
    Ok(summary)
}
#[tauri::command]
fn preview_catalog(json: String) -> Result<catalog::CatalogSummary, String> {
    Ok(catalog::parse(&json)?.summary())
}
#[tauri::command]
fn install_catalog(
    json: String,
    store: tauri::State<'_, CatalogStore>,
) -> Result<catalog::CatalogSummary, String> {
    let c = catalog::parse(&json)?;
    let mut current = store.catalog.lock().map_err(|e| e.to_string())?;
    profiles::atomic_write(
        &store.path,
        &serde_json::to_vec_pretty(&c).map_err(|e| e.to_string())?,
    )?;
    *current = c;
    *store.load_error.lock().map_err(|e| e.to_string())? = None;
    Ok(current.summary())
}
#[tauri::command]
fn reset_catalog(store: tauri::State<'_, CatalogStore>) -> Result<catalog::CatalogSummary, String> {
    let mut current = store.catalog.lock().map_err(|e| e.to_string())?;
    if store.path.exists() {
        std::fs::remove_file(&store.path).map_err(|e| e.to_string())?;
    }
    *current = catalog::catalog();
    *store.load_error.lock().map_err(|e| e.to_string())? = None;
    Ok(current.summary())
}
#[tauri::command]
async fn check_profile(
    json: String,
    gpu_id: Option<String>,
    destination: String,
    store: tauri::State<'_, CatalogStore>,
) -> Result<Assessment, String> {
    let profile = profiles::parse(&json)?;
    let catalog = store.catalog.lock().map_err(|e| e.to_string())?.clone();
    let catalog_error = store.load_error.lock().map_err(|e| e.to_string())?.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let system = scan();
        if gpu_id
            .as_ref()
            .is_some_and(|id| !system.gpus.iter().any(|g| &g.id == id))
        {
            return Err("Selected GPU disappeared. Rescan and select another adapter.".into());
        }
        let volume = resolve_location(&destination, &system.volumes)?;
        let mut report = assess_with_catalog(
            &profile,
            &system,
            gpu_id.as_deref(),
            &destination,
            Some(volume),
            &catalog,
        );
        if let Some(error) = catalog_error {
            report.warnings.push(error);
        }
        Ok(report)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn graphics_settings(
    report: Assessment,
    width: u32,
    height: u32,
    preset: Option<String>,
    observed: Option<f64>,
    target: Option<f64>,
) -> Result<Vec<SettingRecommendation>, String> {
    profiles::validate(&report.profile)?;
    if width == 0
        || height == 0
        || width > 16384
        || height > 16384
        || [observed, target]
            .into_iter()
            .flatten()
            .any(|v| !v.is_finite() || v <= 0. || v > 10000.)
        || preset
            .as_deref()
            .is_some_and(|p| !["Very Low", "Low", "Medium", "High", "Ultra"].contains(&p))
    {
        return Err("Invalid resolution, preset or measured FPS.".into());
    }
    Ok(recommend(
        &report,
        width,
        height,
        preset.as_deref(),
        observed,
        target,
    ))
}
#[tauri::command]
async fn export_document(
    app: tauri::AppHandle,
    json: String,
    kind: String,
) -> Result<bool, String> {
    if json.len() > 10_000_000 {
        return Err("Export exceeds 10 MB.".into());
    }
    let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    let name = match kind.as_str() {
        "profile" => "game-profile.json",
        "diagnostics" => "gamespec-diagnostics.json",
        "report" => "compatibility-report.json",
        _ => return Err("Unsupported export kind.".into()),
    };
    tauri::async_runtime::spawn_blocking(move || {
        match app
            .dialog()
            .file()
            .add_filter("JSON", &["json"])
            .set_file_name(name)
            .blocking_save_file()
        {
            Some(path) => {
                let path = path.into_path().map_err(|e| e.to_string())?;
                profiles::atomic_write(&path, json.as_bytes())?;
                Ok(true)
            }
            None => Ok(false),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let directory = match std::env::var_os("GAMESPEC_DATA_DIR") {
                Some(p) => {
                    let p = PathBuf::from(p);
                    if !p.is_absolute() {
                        return Err("GAMESPEC_DATA_DIR must be absolute".into());
                    }
                    p
                }
                None => app.path().app_data_dir()?,
            };
            let path = directory.join("catalog.json");
            let (catalog, load_error) = if path.exists() {
                match catalog::load(Some(&path)) {
                    Ok(catalog) => (catalog, None),
                    Err(error) => (catalog::catalog(), Some(format!("Saved catalog was rejected: {error}. Bundled baseline is active. Original file was preserved; review Settings to replace or reset it."))),
                }
            } else {
                (catalog::catalog(), None)
            };
            app.manage(ProfileStore {
                directory: directory.join("profiles"),
                lock: Mutex::new(()),
            });
            app.manage(CatalogStore {
                path,
                catalog: Mutex::new(catalog),
                load_error: Mutex::new(load_error),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_system,
            list_profiles,
            parse_profile,
            blank_profile,
            save_profile,
            remove_profile,
            catalog_info,
            preview_catalog,
            install_catalog,
            reset_catalog,
            check_profile,
            graphics_settings,
            export_document
        ])
        .run(tauri::generate_context!())
        .expect("failed to run GameSpec Checker");
}
