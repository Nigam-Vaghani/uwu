use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TtsProviderKind {
    WebSpeech,
    WindowsSapi,
    Piper,
    Silent,
}

impl Default for TtsProviderKind {
    fn default() -> Self {
        Self::WebSpeech
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsConfig {
    pub provider: TtsProviderKind,
    pub voice_id: String,
    pub rate: f64,
    pub pitch: f64,
    pub volume: f64,
    pub speak_on_reminder: bool,
    pub speak_on_ai: bool,
    pub piper_model_path: String,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            provider: TtsProviderKind::WebSpeech,
            voice_id: String::new(),
            rate: 1.0,
            pitch: 1.0,
            volume: 0.8,
            speak_on_reminder: true,
            speak_on_ai: true,
            piper_model_path: String::new(),
        }
    }
}

pub trait TtsSpeaker: Send + Sync {
    fn speak(&self, text: &str, config: &TtsConfig) -> Result<(), String>;
    fn list_voices(&self) -> Result<Vec<VoiceInfo>, String>;
    fn stop(&self) -> Result<(), String>;
}
