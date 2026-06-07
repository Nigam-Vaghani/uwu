mod piper;
mod provider;
mod sapi;

pub use provider::{TtsConfig, TtsProviderKind, VoiceInfo};

use piper::PiperSpeaker;
use provider::TtsSpeaker;
use sapi::SapiSpeaker;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

const STORE_NAME: &str = "settings.json";
const TTS_CONFIG_KEY: &str = "tts-config";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTtsConfigInput {
    pub provider: TtsProviderKind,
    pub voice_id: String,
    pub rate: f64,
    pub pitch: f64,
    pub volume: f64,
    pub speak_on_reminder: bool,
    pub speak_on_ai: bool,
    pub piper_model_path: String,
}

pub struct TtsState {
    cancel_flag: Arc<AtomicBool>,
    active_sink: Arc<Mutex<Option<rodio::Sink>>>,
}

impl TtsState {
    pub fn new() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            active_sink: Arc::new(Mutex::new(None)),
        }
    }

    fn reset_cancel(&self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
    }
}

fn load_store(app: &AppHandle) -> Result<Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    app.store(STORE_NAME).map_err(|error| error.to_string())
}

fn read_stored_config(app: &AppHandle) -> Result<TtsConfig, String> {
    let store = load_store(app)?;
    match store.get(TTS_CONFIG_KEY) {
        Some(value) => serde_json::from_value(value).map_err(|error| error.to_string()),
        None => Ok(TtsConfig::default()),
    }
}

fn save_stored_config(app: &AppHandle, config: &TtsConfig) -> Result<(), String> {
    let store = load_store(app)?;
    store.set(
        TTS_CONFIG_KEY,
        serde_json::to_value(config).map_err(|error| error.to_string())?,
    );
    store.save().map_err(|error| error.to_string())?;
    Ok(())
}

fn normalize_config(mut config: TtsConfig) -> TtsConfig {
    config.rate = config.rate.clamp(0.5, 2.0);
    config.pitch = config.pitch.clamp(0.5, 2.0);
    config.volume = config.volume.clamp(0.0, 1.0);
    config
}

fn create_speaker(_state: &TtsState) -> Result<Box<dyn TtsSpeaker>, String> {
    Ok(Box::new(SapiSpeaker::new()?))
}

fn create_piper_speaker(state: &TtsState) -> Box<dyn TtsSpeaker> {
    Box::new(PiperSpeaker::new(
        state.cancel_flag.clone(),
        state.active_sink.clone(),
    ))
}

#[tauri::command]
pub fn speak(
    app: AppHandle,
    state: State<'_, TtsState>,
    text: String,
    provider: Option<TtsProviderKind>,
) -> Result<(), String> {
    let mut config = read_stored_config(&app)?;
    if let Some(override_provider) = provider {
        config.provider = override_provider;
    }
    if config.provider == TtsProviderKind::Silent || config.provider == TtsProviderKind::WebSpeech
    {
        return Ok(());
    }

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    state.reset_cancel();

    match config.provider {
        TtsProviderKind::WindowsSapi => {
            let speaker = create_speaker(&state)?;
            speaker.speak(trimmed, &config)
        }
        TtsProviderKind::Piper => {
            let speaker = create_piper_speaker(&state);
            speaker.speak(trimmed, &config)
        }
        _ => Ok(()),
    }
}

#[tauri::command]
pub fn stop_speaking(state: State<'_, TtsState>, app: AppHandle) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::SeqCst);

    if let Ok(mut active) = state.active_sink.lock() {
        if let Some(sink) = active.take() {
            sink.stop();
        }
    }

    if let Ok(speaker) = create_speaker(&state) {
        let _ = speaker.stop();
    }

    let piper = create_piper_speaker(&state);
    let _ = piper.stop();

    state.cancel_flag.store(false, Ordering::SeqCst);
    let _ = app;
    Ok(())
}

#[tauri::command]
pub fn get_tts_config(app: AppHandle) -> Result<TtsConfig, String> {
    read_stored_config(&app).map(normalize_config)
}

#[tauri::command]
pub fn set_tts_config(app: AppHandle, input: SetTtsConfigInput) -> Result<TtsConfig, String> {
    let config = normalize_config(TtsConfig {
        provider: input.provider,
        voice_id: input.voice_id,
        rate: input.rate,
        pitch: input.pitch,
        volume: input.volume,
        speak_on_reminder: input.speak_on_reminder,
        speak_on_ai: input.speak_on_ai,
        piper_model_path: input.piper_model_path,
    });

    save_stored_config(&app, &config)?;
    Ok(config)
}

#[tauri::command]
pub fn list_voices(_app: AppHandle, provider: TtsProviderKind) -> Result<Vec<VoiceInfo>, String> {
    match provider {
        TtsProviderKind::WindowsSapi => {
            let speaker = SapiSpeaker::new()?;
            speaker.list_voices()
        }
        TtsProviderKind::Piper => {
            let state = TtsState::new();
            let speaker = create_piper_speaker(&state);
            speaker.list_voices()
        }
        TtsProviderKind::WebSpeech | TtsProviderKind::Silent => Ok(Vec::new()),
    }
}
