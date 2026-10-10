#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use gamespec_core::*;
use std::{path::PathBuf, sync::Mutex};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

struct ProfileStore {
    directory: PathBuf,
    lock: Mutex<()>,
}
#[tauri::command]
async fn scan_system() -> Result<SystemSnapshot, String> {
    tauri::async_runtime::spawn_blocking(scan)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn list_profiles(store: tauri::State<'_, ProfileStore>) -> Result<Vec<GameProfile>, String> {
    let _guard = store.lock.lock().map_err(|e| e.to_string())?;
    profiles::load(&store.directory)
}
#[tauri::command]
fn save_profile(
    json: String,
    store: tauri::State<'_, ProfileStore>,
) -> Result<GameProfile, String> {
    let profile = profiles::parse(&json)?;
    let _guard = store.lock.lock().map_err(|e| e.to_string())?;
    profiles::save(&store.directory, &profile)?;
    Ok(profile)
}
#[tauri::command]
fn remove_profile(id: String, store: tauri::State<'_, ProfileStore>) -> Result<(), String> {
    let mut test = profiles::sample();
    test.id = id.clone();
    profiles::validate(&test)?;
    if id == "icarus" {
        return Err("Icarus is the bundled sample. Edit or duplicate it instead.".into());
    }
    let _guard = store.lock.lock().map_err(|e| e.to_string())?;
    std::fs::remove_file(store.directory.join(format!("{id}.json"))).map_err(|e| e.to_string())
}
#[tauri::command]
async fn check_profile(
    json: String,
    gpu_id: Option<String>,
    destination: String,
) -> Result<Assessment, String> {
    let profile = profiles::parse(&json)?;
    tauri::async_runtime::spawn_blocking(move || {
        let system = scan();
        if gpu_id
            .as_ref()
            .is_some_and(|id| !system.gpus.iter().any(|g| &g.id == id))
        {
            return Err(
                "Selected GPU is no longer detected. Rescan and select another adapter.".into(),
            );
        }
        let volume = resolve_location(&destination, &system.volumes)?;
        Ok(assess(
            &profile,
            &system,
            gpu_id.as_deref(),
            &destination,
            Some(volume),
        ))
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
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err("Invalid display dimensions.".into());
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
    let name = if kind == "profile" {
        "game-profile.json"
    } else {
        "compatibility-report.json"
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
                std::fs::write(path, json).map_err(|e| e.to_string())?;
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
            let directory = app.path().app_data_dir()?.join("profiles");
            app.manage(ProfileStore {
                directory,
                lock: Mutex::new(()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_system,
            list_profiles,
            save_profile,
            remove_profile,
            check_profile,
            graphics_settings,
            export_document
        ])
        .run(tauri::generate_context!())
        .expect("failed to run GameSpec Checker");
}
