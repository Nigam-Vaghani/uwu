use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rodio::{Decoder, OutputStream, Sink};

use super::provider::{TtsConfig, TtsSpeaker, VoiceInfo};

pub struct PiperSpeaker {
    cancel_flag: Arc<AtomicBool>,
    active_sink: Arc<Mutex<Option<Sink>>>,
}

impl PiperSpeaker {
    pub fn new(cancel_flag: Arc<AtomicBool>, active_sink: Arc<Mutex<Option<Sink>>>) -> Self {
        Self {
            cancel_flag,
            active_sink,
        }
    }

    fn resolve_piper_binary() -> Result<PathBuf, String> {
        let candidates = [
            PathBuf::from("binaries/piper.exe"),
            PathBuf::from("src-tauri/binaries/piper.exe"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let bundled = dir.join("binaries/piper.exe");
                if bundled.exists() {
                    return Ok(bundled);
                }
            }
        }

        Err(
            "Piper binary not found. Place piper.exe in src-tauri/binaries/ and a .onnx model path in voice settings."
                .to_string(),
        )
    }

    fn resolve_model_path(config: &TtsConfig) -> Result<PathBuf, String> {
        if config.piper_model_path.trim().is_empty() {
            return Err("Piper model path is not configured.".to_string());
        }

        let path = PathBuf::from(config.piper_model_path.trim());
        if !path.exists() {
            return Err(format!(
                "Piper model not found at {}",
                path.display()
            ));
        }

        Ok(path)
    }

    fn play_wav(&self, wav_path: &Path, volume: f32) -> Result<(), String> {
        self.cancel_flag.store(false, Ordering::SeqCst);

        let file = fs::File::open(wav_path).map_err(|error| error.to_string())?;
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|error| error.to_string())?;
        let sink = Sink::try_new(&stream_handle).map_err(|error| error.to_string())?;

        let source = Decoder::new(file).map_err(|error| error.to_string())?;
        sink.set_volume(volume.clamp(0.0, 1.0));
        sink.append(source);

        {
            let mut active = self.active_sink.lock().map_err(|error| error.to_string())?;
            *active = Some(sink);
        }

        loop {
            if self.cancel_flag.load(Ordering::SeqCst) {
                if let Ok(mut active) = self.active_sink.lock() {
                    if let Some(sink) = active.take() {
                        sink.stop();
                    }
                }
                return Ok(());
            }

            let done = {
                let active = self.active_sink.lock().map_err(|error| error.to_string())?;
                active
                    .as_ref()
                    .map(|sink| sink.empty())
                    .unwrap_or(true)
            };

            if done {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        if let Ok(mut active) = self.active_sink.lock() {
            active.take();
        }

        Ok(())
    }
}

impl TtsSpeaker for PiperSpeaker {
    fn speak(&self, text: &str, config: &TtsConfig) -> Result<(), String> {
        let piper = Self::resolve_piper_binary()?;
        let model = Self::resolve_model_path(config)?;

        let temp_dir = std::env::temp_dir();
        let wav_path = temp_dir.join(format!("uwu-piper-{}.wav", std::process::id()));

        let mut child = Command::new(&piper)
            .arg("--model")
            .arg(&model)
            .arg("--output_file")
            .arg(&wav_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("Failed to start Piper: {error}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|error| error.to_string())?;
        }

        let output = child
            .wait_with_output()
            .map_err(|error| error.to_string())?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = fs::remove_file(&wav_path);
            return Err(format!("Piper failed: {stderr}"));
        }

        if self.cancel_flag.load(Ordering::SeqCst) {
            let _ = fs::remove_file(&wav_path);
            return Ok(());
        }

        let result = self.play_wav(&wav_path, config.volume as f32);
        let _ = fs::remove_file(&wav_path);
        result
    }

    fn list_voices(&self) -> Result<Vec<VoiceInfo>, String> {
        Ok(vec![VoiceInfo {
            id: "piper-default".to_string(),
            name: "Piper (configured model)".to_string(),
            language: Some("en".to_string()),
        }])
    }

    fn stop(&self) -> Result<(), String> {
        self.cancel_flag.store(true, Ordering::SeqCst);
        if let Ok(mut active) = self.active_sink.lock() {
            if let Some(sink) = active.take() {
                sink.stop();
            }
        }
        Ok(())
    }
}
