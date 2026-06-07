use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_NAME: &str = "settings.json";
const CONFIG_KEY: &str = "app-config";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub pet_name: String,
    pub personality: String,
    pub opacity: f64,
    pub position: Option<WindowPosition>,
    pub has_seen_welcome: bool,
    pub active_skin: String,
    pub active_theme: String,
    pub active_sound_pack: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            pet_name: "Uwu-chan".to_string(),
            personality: "Cheerful".to_string(),
            opacity: 0.9,
            position: None,
            has_seen_welcome: false,
            active_skin: "default".to_string(),
            active_theme: "elegance".to_string(),
            active_sound_pack: "default".to_string(),
        }
    }
}

fn load_store(app: &AppHandle) -> Result<Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    app.store(STORE_NAME).map_err(|error| error.to_string())
}

pub fn read_config(app: &AppHandle) -> Result<AppConfig, String> {
    let store = load_store(app)?;
    match store.get(CONFIG_KEY) {
        Some(value) => serde_json::from_value(value).map_err(|error| error.to_string()),
        None => {
            let config = AppConfig::default();
            store.set(
                CONFIG_KEY,
                serde_json::to_value(&config).map_err(|error| error.to_string())?,
            );
            store.save().map_err(|error| error.to_string())?;
            Ok(config)
        }
    }
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<AppConfig, String> {
    let store = load_store(app)?;
    store.set(
        CONFIG_KEY,
        serde_json::to_value(config).map_err(|error| error.to_string())?,
    );
    store.save().map_err(|error| error.to_string())?;
    Ok(config.clone())
}

pub fn save_position(app: &AppHandle, position: WindowPosition) -> Result<WindowPosition, String> {
    let mut config = read_config(app)?;
    config.position = Some(position.clone());
    save_config(app, &config)?;
    Ok(position)
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> Result<AppConfig, String> {
    read_config(&app)
}

#[tauri::command]
pub fn set_config(app: AppHandle, config: AppConfig) -> Result<AppConfig, String> {
    save_config(&app, &config)
}