use super::provider::{TtsConfig, TtsSpeaker, VoiceInfo};

pub struct SapiSpeaker;

impl SapiSpeaker {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

impl TtsSpeaker for SapiSpeaker {
    fn speak(&self, text: &str, config: &TtsConfig) -> Result<(), String> {
        use tts::Tts;

        let mut tts = Tts::default().map_err(|error| error.to_string())?;

        if !config.voice_id.is_empty() {
            let voices = tts.voices().map_err(|error| error.to_string())?;
            if let Some(voice) = voices.into_iter().find(|voice| {
                voice.id() == config.voice_id || voice.name() == config.voice_id
            }) {
                tts.set_voice(&voice).map_err(|error| error.to_string())?;
            }
        }

        let _ = tts.set_rate(config.rate as f32);

        tts.speak(text, false)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn list_voices(&self) -> Result<Vec<VoiceInfo>, String> {
        use tts::Tts;

        let tts = Tts::default().map_err(|error| error.to_string())?;
        let voices = tts.voices().map_err(|error| error.to_string())?;

        Ok(voices
            .into_iter()
            .filter_map(|voice| {
                let id = voice.id();
                let name = voice.name();
                if id.is_empty() && name.is_empty() {
                    return None;
                }
                Some(VoiceInfo {
                    id: if id.is_empty() { name.clone() } else { id },
                    name,
                    language: Some(voice.language().to_string()),
                })
            })
            .collect())
    }

    fn stop(&self) -> Result<(), String> {
        use tts::Tts;

        let mut tts = Tts::default().map_err(|error| error.to_string())?;
        tts.stop()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}
