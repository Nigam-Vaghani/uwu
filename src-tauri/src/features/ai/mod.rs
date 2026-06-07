mod context;
mod fallback;
mod gemini;
mod groq;
mod ollama;
mod provider;

pub use context::{assemble_context, build_prompt, record_event, RecentEventsState};

use fallback::FallbackProvider;
use gemini::GeminiProvider;
use groq::GroqProvider;
use ollama::OllamaProvider;
use provider::AiProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::features::events::PetEventPayload;

const STORE_NAME: &str = "settings.json";
const AI_CONFIG_KEY: &str = "ai-config";
const AI_API_KEY: &str = "ai-api-key";
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderKind {
    None,
    Groq,
    Gemini,
    Ollama,
}

impl Default for AiProviderKind {
    fn default() -> Self {
        Self::Groq
    }
}

impl AiProviderKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Groq => "groq",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
        }
    }

    fn default_model(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Groq => "llama3-8b-8192",
            Self::Gemini => "gemini-2.0-flash",
            Self::Ollama => "llama3",
        }
    }

    fn requires_api_key(self) -> bool {
        matches!(self, Self::Groq | Self::Gemini)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredAiConfig {
    pub provider: AiProviderKind,
    pub model: String,
    pub ollama_base_url: String,
}

impl Default for StoredAiConfig {
    fn default() -> Self {
        Self {
            provider: AiProviderKind::Groq,
            model: AiProviderKind::Groq.default_model().to_string(),
            ollama_base_url: "http://localhost:11434".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfigResponse {
    pub provider: AiProviderKind,
    pub model: String,
    pub ollama_base_url: String,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAiConfigInput {
    pub provider: AiProviderKind,
    pub model: String,
    pub ollama_base_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCompleteResponse {
    pub text: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionTestResult {
    pub ok: bool,
    pub message: String,
    pub model: String,
}

fn load_store(app: &AppHandle) -> Result<Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    app.store(STORE_NAME).map_err(|error| error.to_string())
}

fn read_stored_config(app: &AppHandle) -> Result<StoredAiConfig, String> {
    let store = load_store(app)?;
    match store.get(AI_CONFIG_KEY) {
        Some(value) => serde_json::from_value(value).map_err(|error| error.to_string()),
        None => Ok(StoredAiConfig::default()),
    }
}

fn read_api_key(app: &AppHandle) -> Result<Option<String>, String> {
    let store = load_store(app)?;
    if let Some(value) = store.get(AI_API_KEY) {
        let key = value
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "Stored API key is invalid".to_string())?;
        if !key.trim().is_empty() {
            return Ok(Some(key));
        }
    }

    if let Ok(key) = std::env::var("GROQ_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(Some(key));
        }
    }

    Ok(None)
}

pub fn resolve_api_key(app: &AppHandle) -> Result<Option<String>, String> {
    read_api_key(app)
}

pub async fn complete_prompt(app: &AppHandle, prompt: &str) -> Result<String, String> {
    let stored = read_stored_config(app)?;
    let api_key = resolve_api_key(app)?;
    let provider = create_provider(&stored, api_key)?;
    provider.complete(prompt).await
}

fn save_stored_config(app: &AppHandle, config: &StoredAiConfig) -> Result<(), String> {
    let store = load_store(app)?;
    store.set(
        AI_CONFIG_KEY,
        serde_json::to_value(config).map_err(|error| error.to_string())?,
    );
    store.save().map_err(|error| error.to_string())?;
    Ok(())
}

fn save_api_key(app: &AppHandle, api_key: &str) -> Result<(), String> {
    let store = load_store(app)?;
    store.set(AI_API_KEY, serde_json::Value::String(api_key.to_string()));
    store.save().map_err(|error| error.to_string())?;
    Ok(())
}

fn create_provider(
    config: &StoredAiConfig,
    api_key: Option<String>,
) -> Result<Box<dyn AiProvider>, String> {
    let model = if config.model.trim().is_empty() {
        config.provider.default_model().to_string()
    } else {
        config.model.clone()
    };

    match config.provider {
        AiProviderKind::None => Err("No AI provider configured".to_string()),
        AiProviderKind::Groq => {
            let key = api_key.filter(|value| !value.trim().is_empty()).ok_or_else(|| {
                "Groq API key is missing. Add one in AI settings.".to_string()
            })?;
            Ok(Box::new(GroqProvider::new(key, model)))
        }
        AiProviderKind::Gemini => {
            let key = api_key.filter(|value| !value.trim().is_empty()).ok_or_else(|| {
                "Gemini API key is missing. Add one in AI settings.".to_string()
            })?;
            Ok(Box::new(GeminiProvider::new(key, model)))
        }
        AiProviderKind::Ollama => Ok(Box::new(OllamaProvider::new(
            config.ollama_base_url.clone(),
            model,
        ))),
    }
}

fn sanitize_response(text: &str) -> String {
    text.replace(['*', '#', '`'], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

async fn complete_with_router(
    app: &AppHandle,
    payload: &PetEventPayload,
) -> Result<AiCompleteResponse, String> {
    let (context, _config) = assemble_context(app, payload)?;
    let prompt = build_prompt(&context, payload);
    let stored = read_stored_config(app)?;
    let api_key = read_api_key(app)?;
    let fallback = FallbackProvider::new()?;

    if stored.provider != AiProviderKind::None {
        if let Ok(provider) = create_provider(&stored, api_key.clone()) {
            let prompt_owned = prompt.clone();
            let attempt = tokio::time::timeout(PROVIDER_TIMEOUT, provider.complete(&prompt_owned)).await;

            match attempt {
                Ok(Ok(response)) if !response.trim().is_empty() => {
                    return Ok(AiCompleteResponse {
                        text: sanitize_response(&response),
                        source: stored.provider.as_str().to_string(),
                    });
                }
                Ok(Err(_error)) => {}
                Err(_) => {}
                _ => {}
            }
        }
    }

    let text = fallback.complete_for_event(payload.event_type, &context, payload)?;
    Ok(AiCompleteResponse {
        text,
        source: "fallback".to_string(),
    })
}

#[tauri::command]
pub async fn ai_complete(
    app: AppHandle,
    recent_events: State<'_, RecentEventsState>,
    payload: PetEventPayload,
) -> Result<AiCompleteResponse, String> {
    record_event(recent_events, &format!("{:?}", payload.event_type));
    complete_with_router(&app, &payload).await
}

#[tauri::command]
pub async fn test_ai_connection(app: AppHandle) -> Result<AiConnectionTestResult, String> {
    let stored = read_stored_config(&app)?;
    let model = if stored.model.trim().is_empty() {
        stored.provider.default_model().to_string()
    } else {
        stored.model.clone()
    };

    if stored.provider == AiProviderKind::None {
        return Ok(AiConnectionTestResult {
            ok: true,
            message: "Rule-based fallback is active (no remote provider configured).".to_string(),
            model: "fallback".to_string(),
        });
    }

    let api_key = read_api_key(&app)?;
    if stored.provider.requires_api_key() && api_key.as_ref().is_none_or(|key| key.trim().is_empty())
    {
        return Ok(AiConnectionTestResult {
            ok: false,
            message: "API key is required for the selected provider.".to_string(),
            model,
        });
    }

    let provider = create_provider(&stored, api_key)?;
    let prompt = "Reply with exactly: connection ok";
    let attempt = tokio::time::timeout(PROVIDER_TIMEOUT, provider.complete(prompt)).await;

    match attempt {
        Ok(Ok(response)) if !response.trim().is_empty() => Ok(AiConnectionTestResult {
            ok: true,
            message: format!("Connected — {} ready", model),
            model,
        }),
        Ok(Ok(_)) => Ok(AiConnectionTestResult {
            ok: false,
            message: "Provider returned an empty response.".to_string(),
            model,
        }),
        Ok(Err(error)) => Ok(AiConnectionTestResult {
            ok: false,
            message: error,
            model,
        }),
        Err(_) => Ok(AiConnectionTestResult {
            ok: false,
            message: "Connection timed out after 3 seconds.".to_string(),
            model,
        }),
    }
}

#[tauri::command]
pub fn get_ai_config(app: AppHandle) -> Result<AiConfigResponse, String> {
    let stored = read_stored_config(&app)?;
    let has_api_key = read_api_key(&app)?.is_some();

    Ok(AiConfigResponse {
        provider: stored.provider,
        model: stored.model,
        ollama_base_url: stored.ollama_base_url,
        has_api_key,
    })
}

#[tauri::command]
pub fn set_ai_config(app: AppHandle, input: SetAiConfigInput) -> Result<AiConfigResponse, String> {
    let model = if input.model.trim().is_empty() {
        input.provider.default_model().to_string()
    } else {
        input.model.trim().to_string()
    };

    let stored = StoredAiConfig {
        provider: input.provider,
        model,
        ollama_base_url: if input.ollama_base_url.trim().is_empty() {
            "http://localhost:11434".to_string()
        } else {
            input.ollama_base_url.trim().to_string()
        },
    };

    save_stored_config(&app, &stored)?;

    if let Some(api_key) = input.api_key {
        let trimmed = api_key.trim();
        if !trimmed.is_empty() {
            save_api_key(&app, trimmed)?;
        }
    }

    get_ai_config(app)
}
