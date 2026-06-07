use crate::features::ai::complete_prompt;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

const STORE_NAME: &str = "settings.json";
const QUOTE_CONFIG_KEY: &str = "quote-config";
const QUOTE_EVENT: &str = "quote:generated";

pub const BUILTIN_QUOTE_TYPES: &[&str] = &["funny", "motivational", "coding", "wisdom", "chill"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteConfig {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub selected_type: String,
    pub custom_types: Vec<String>,
}

impl Default for QuoteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 5,
            selected_type: "motivational".to_string(),
            custom_types: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotePayload {
    pub text: String,
    pub quote_type: String,
}

struct QuoteSchedulerState {
    last_fired_at: AtomicI64,
    config: Mutex<QuoteConfig>,
}

impl QuoteSchedulerState {
    fn new(config: QuoteConfig) -> Self {
        Self {
            last_fired_at: AtomicI64::new(0),
            config: Mutex::new(config),
        }
    }
}

fn load_store(app: &AppHandle) -> Result<Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    app.store(STORE_NAME).map_err(|error| error.to_string())
}

pub fn read_quote_config(app: &AppHandle) -> Result<QuoteConfig, String> {
    let store = load_store(app)?;
    match store.get(QUOTE_CONFIG_KEY) {
        Some(value) => serde_json::from_value(value).map_err(|error| error.to_string()),
        None => Ok(QuoteConfig::default()),
    }
}

fn save_quote_config(app: &AppHandle, config: &QuoteConfig) -> Result<(), String> {
    let store = load_store(app)?;
    store.set(
        QUOTE_CONFIG_KEY,
        serde_json::to_value(config).map_err(|error| error.to_string())?,
    );
    store.save().map_err(|error| error.to_string())?;
    Ok(())
}

fn sanitize_quote(text: &str) -> String {
    text.replace(['*', '#', '`', '"'], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn build_quote_prompt(quote_type: &str, pet_name: &str) -> String {
    format!(
        "Write exactly one short {quote_type} quote for a friendly desktop pet companion named {pet_name}. \
         Maximum 18 words. Plain text only — no quotes, no author name, no emoji, no markdown."
    )
}

async fn generate_quote_text(app: &AppHandle, quote_type: &str) -> Result<String, String> {
    let pet_name = crate::features::config::read_config(app)
        .map(|config| config.pet_name)
        .unwrap_or_else(|_| "Uwu-chan".to_string());

    let prompt = build_quote_prompt(quote_type, &pet_name);
    let response = complete_prompt(app, &prompt).await?;
    let cleaned = sanitize_quote(&response);

    if cleaned.is_empty() {
        return Err("AI returned an empty quote.".to_string());
    }

    Ok(cleaned)
}

async fn fire_quote(app: &AppHandle, quote_type: &str) -> Result<(), String> {
    let text = generate_quote_text(app, quote_type).await?;
    let payload = QuotePayload {
        text: text.clone(),
        quote_type: quote_type.to_string(),
    };

    app.emit(QUOTE_EVENT, payload)
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_quote_config(app: AppHandle) -> Result<QuoteConfig, String> {
    read_quote_config(&app)
}

#[tauri::command]
pub fn set_quote_config(app: AppHandle, config: QuoteConfig) -> Result<QuoteConfig, String> {
    let normalized = QuoteConfig {
        enabled: config.enabled,
        interval_minutes: config.interval_minutes.clamp(1, 1440),
        selected_type: if config.selected_type.trim().is_empty() {
            "motivational".to_string()
        } else {
            config.selected_type.trim().to_string()
        },
        custom_types: config
            .custom_types
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect(),
    };

    save_quote_config(&app, &normalized)?;

    if let Some(state) = app.try_state::<Arc<QuoteSchedulerState>>() {
        if let Ok(mut guard) = state.config.lock() {
            *guard = normalized.clone();
        }
    }

    Ok(normalized)
}

#[tauri::command]
pub async fn preview_quote(app: AppHandle, quote_type: String) -> Result<QuotePayload, String> {
    let quote_type = if quote_type.trim().is_empty() {
        "motivational".to_string()
    } else {
        quote_type.trim().to_string()
    };
    let text = generate_quote_text(&app, &quote_type).await?;
    Ok(QuotePayload {
        text,
        quote_type,
    })
}

pub fn start_scheduler(app: AppHandle, state: Arc<QuoteSchedulerState>) {
    tauri::async_runtime::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(30));

        loop {
            tick.tick().await;

            let config = match state.config.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => continue,
            };

            if !config.enabled {
                continue;
            }

            let interval_secs = (config.interval_minutes * 60) as i64;
            let now = crate::features::database::chrono_now();
            let last = state.last_fired_at.load(Ordering::Relaxed);
            if last > 0 && now - last < interval_secs {
                continue;
            }

            if fire_quote(&app, &config.selected_type).await.is_ok() {
                state.last_fired_at.store(now, Ordering::Relaxed);
            }
        }
    });
}

pub fn init(app: &AppHandle) -> Result<(), String> {
    let config = read_quote_config(app)?;
    let state = Arc::new(QuoteSchedulerState::new(config));
    app.manage(state.clone());
    start_scheduler(app.clone(), state);
    Ok(())
}
