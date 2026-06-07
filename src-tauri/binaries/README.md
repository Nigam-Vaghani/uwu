# Piper TTS (optional)

Phase 5 supports Piper as a local neural TTS provider. The binary is **not** bundled by default.

## Setup

1. Download [Piper](https://github.com/rhasspy/piper/releases) for Windows.
2. Place `piper.exe` in this folder (`src-tauri/binaries/piper.exe`).
3. Download a voice model (`.onnx` + `.onnx.json`) from the Piper voices repository.
4. In **Settings → Voice**, choose **Piper TTS** and set the full path to your `.onnx` model file.

## Usage

Text is sent to Piper via stdin; synthesized WAV audio is played through the Rust `rodio` crate.

If Piper is missing or misconfigured, the app falls back to Web Speech API or Windows SAPI automatically.
