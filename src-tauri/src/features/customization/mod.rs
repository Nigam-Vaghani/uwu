use crate::features::config::{self, AppConfig};
use crate::features::reminders::Reminder;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const STORE_NAME: &str = "settings.json";
const BUNDLE_VERSION: &str = "1.0.0";
pub const SKIN_CHANGED: &str = "customization:skin-changed";
pub const THEME_CHANGED: &str = "customization:theme-changed";
pub const SOUND_PACK_CHANGED: &str = "customization:sound-pack-changed";
pub const BUNDLE_IMPORTED: &str = "customization:bundle-imported";

const BUILTIN_SKINS: &[&str] = &["default", "ghosty"];
const BUILTIN_SOUND_PACKS: &[&str] = &["default"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinManifest {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub frame_width: u32,
    pub frame_height: u32,
    pub preview: String,
    pub builtin: bool,
    pub asset_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundPackManifest {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalityInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkinSummaryFile {
    pub id: Option<String>,
    pub name: String,
    pub author: String,
    pub version: String,
    pub frame_width: u32,
    pub frame_height: u32,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedSkinManifest {
    pub manifest: serde_json::Value,
    pub asset_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SoundPackManifestFile {
    pub id: Option<String>,
    pub name: String,
    pub author: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UwuBundleManifest {
    pub version: String,
    pub exported_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UwuBundle {
    pub manifest: UwuBundleManifest,
    pub app_config: AppConfig,
    pub ai_config: serde_json::Value,
    pub tts_config: serde_json::Value,
    pub reminders: Vec<Reminder>,
}

fn user_skins_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("skins");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn user_sounds_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("sounds");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn read_skin_manifest(path: &Path, builtin: bool) -> Result<SkinManifest, String> {
    let content = fs::read_to_string(path.join("manifest.json")).map_err(|error| error.to_string())?;
    let file: SkinSummaryFile = serde_json::from_str(&content).map_err(|error| error.to_string())?;
    let id = file
        .id
        .or_else(|| path.file_name().and_then(|name| name.to_str().map(str::to_string)))
        .ok_or_else(|| "Skin folder is missing an id".to_string())?;

    Ok(SkinManifest {
        id,
        name: file.name,
        author: file.author,
        version: file.version,
        frame_width: file.frame_width,
        frame_height: file.frame_height,
        preview: file.preview,
        builtin,
        asset_path: if builtin {
            None
        } else {
            Some(path.to_string_lossy().to_string())
        },
    })
}

fn read_sound_pack_manifest(path: &Path, builtin: bool) -> Result<SoundPackManifest, String> {
    let content = fs::read_to_string(path.join("manifest.json")).map_err(|error| error.to_string())?;
    let file: SoundPackManifestFile =
        serde_json::from_str(&content).map_err(|error| error.to_string())?;
    let id = file
        .id
        .or_else(|| path.file_name().and_then(|name| name.to_str().map(str::to_string)))
        .ok_or_else(|| "Sound pack folder is missing an id".to_string())?;

    Ok(SoundPackManifest {
        id,
        name: file.name,
        author: file.author,
        version: file.version,
        builtin,
    })
}

fn scan_skin_dir(base: &Path, builtin: bool) -> Result<Vec<SkinManifest>, String> {
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut skins = Vec::new();
    for entry in fs::read_dir(base).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry.file_type().map_err(|error| error.to_string())?.is_dir() {
            continue;
        }

        let path = entry.path();
        if !path.join("manifest.json").exists() {
            continue;
        }

        skins.push(read_skin_manifest(&path, builtin)?);
    }

    skins.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(skins)
}

fn scan_sound_dir(base: &Path, builtin: bool) -> Result<Vec<SoundPackManifest>, String> {
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    for entry in fs::read_dir(base).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry.file_type().map_err(|error| error.to_string())?.is_dir() {
            continue;
        }

        let path = entry.path();
        if !path.join("manifest.json").exists() {
            continue;
        }

        packs.push(read_sound_pack_manifest(&path, builtin)?);
    }

    packs.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(packs)
}

fn builtin_skin_manifests() -> Vec<SkinManifest> {
    BUILTIN_SKINS
        .iter()
        .map(|id| SkinManifest {
            id: (*id).to_string(),
            name: if *id == "ghosty" {
                "Ghosty".to_string()
            } else {
                "Default".to_string()
            },
            author: "UWU Team".to_string(),
            version: "1.0.0".to_string(),
            frame_width: 64,
            frame_height: 64,
            preview: "preview.png".to_string(),
            builtin: true,
            asset_path: None,
        })
        .collect()
}

fn builtin_sound_pack_manifests() -> Vec<SoundPackManifest> {
    BUILTIN_SOUND_PACKS
        .iter()
        .map(|id| SoundPackManifest {
            id: (*id).to_string(),
            name: "Default".to_string(),
            author: "UWU Team".to_string(),
            version: "1.0.0".to_string(),
            builtin: true,
        })
        .collect()
}

fn personality_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(resource) = app.path().resource_dir() {
        dirs.push(resource.join("assets").join("personalities"));
    }
    dirs.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("public")
            .join("assets")
            .join("personalities"),
    );
    dirs
}

fn read_store_value(app: &AppHandle, key: &str) -> Result<serde_json::Value, String> {
    let store = app.store(STORE_NAME).map_err(|error| error.to_string())?;
    Ok(store.get(key).unwrap_or(serde_json::Value::Null))
}

fn list_personalities_from_dir(base: &Path) -> Result<Vec<PersonalityInfo>, String> {
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut personalities = Vec::new();
    for entry in fs::read_dir(base).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Invalid personality filename".to_string())?;
        let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let value: serde_json::Value =
            serde_json::from_str(&content).map_err(|error| error.to_string())?;
        let name = value
            .get("name")
            .and_then(|item| item.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| capitalize(stem));

        personalities.push(PersonalityInfo {
            id: capitalize(stem),
            name,
        });
    }

    personalities.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(personalities)
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn zip_add_file<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    path: &str,
    bytes: &[u8],
) -> Result<(), String> {
    writer
        .start_file(path, SimpleFileOptions::default())
        .map_err(|error| error.to_string())?;
    writer
        .write_all(bytes)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn builtin_skin_path(skin_id: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("public")
        .join("skins")
        .join(skin_id)
}

fn resolve_skin_dir(app: &AppHandle, skin_id: &str) -> Result<(PathBuf, bool), String> {
    if BUILTIN_SKINS.contains(&skin_id) {
        let dev_path = builtin_skin_path(skin_id);
        if dev_path.join("manifest.json").exists() {
            return Ok((dev_path, true));
        }

        if let Ok(resource) = app.path().resource_dir() {
            let prod_path = resource.join("skins").join(skin_id);
            if prod_path.join("manifest.json").exists() {
                return Ok((prod_path, true));
            }
        }
    }

    let custom = user_skins_dir(app)?.join(skin_id);
    if custom.join("manifest.json").exists() {
        return Ok((custom, false));
    }

    Err(format!("Skin '{skin_id}' was not found"))
}

#[tauri::command]
pub fn get_skin_manifest(app: AppHandle, skin_id: String) -> Result<ResolvedSkinManifest, String> {
    let (path, builtin) = resolve_skin_dir(&app, &skin_id)?;
    let content = fs::read_to_string(path.join("manifest.json")).map_err(|error| error.to_string())?;
    let manifest: serde_json::Value =
        serde_json::from_str(&content).map_err(|error| error.to_string())?;

    Ok(ResolvedSkinManifest {
        manifest,
        asset_path: if builtin {
            None
        } else {
            Some(path.to_string_lossy().to_string())
        },
    })
}

#[tauri::command]
pub fn list_skins(app: AppHandle) -> Result<Vec<SkinManifest>, String> {
    let mut skins = builtin_skin_manifests();
    let custom = scan_skin_dir(&user_skins_dir(&app)?, false)?;
    skins.extend(custom);
    Ok(skins)
}

#[tauri::command]
pub fn set_active_skin(app: AppHandle, skin_id: String) -> Result<AppConfig, String> {
    let mut config = config::read_config(&app)?;
    config.active_skin = skin_id.clone();
    let saved = config::save_config(&app, &config)?;
    app.emit(SKIN_CHANGED, &skin_id)
        .map_err(|error| error.to_string())?;
    Ok(saved)
}

#[tauri::command]
pub fn list_sound_packs(app: AppHandle) -> Result<Vec<SoundPackManifest>, String> {
    let mut packs = builtin_sound_pack_manifests();
    let custom = scan_sound_dir(&user_sounds_dir(&app)?, false)?;
    packs.extend(custom);
    Ok(packs)
}

#[tauri::command]
pub fn set_active_sound_pack(app: AppHandle, pack_id: String) -> Result<AppConfig, String> {
    let mut config = config::read_config(&app)?;
    config.active_sound_pack = pack_id.clone();
    let saved = config::save_config(&app, &config)?;
    app.emit(SOUND_PACK_CHANGED, &pack_id)
        .map_err(|error| error.to_string())?;
    Ok(saved)
}

#[tauri::command]
pub fn set_active_theme(app: AppHandle, theme_id: String) -> Result<AppConfig, String> {
    let mut config = config::read_config(&app)?;
    config.active_theme = theme_id.clone();
    let saved = config::save_config(&app, &config)?;
    app.emit(THEME_CHANGED, &theme_id)
        .map_err(|error| error.to_string())?;
    Ok(saved)
}

#[tauri::command]
pub fn list_personalities(app: AppHandle) -> Result<Vec<PersonalityInfo>, String> {
    let mut seen = std::collections::HashSet::new();
    let mut personalities = Vec::new();

    for dir in personality_dirs(&app) {
        for item in list_personalities_from_dir(&dir)? {
            if seen.insert(item.id.clone()) {
                personalities.push(item);
            }
        }
    }

    personalities.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(personalities)
}

#[tauri::command]
pub fn export_uwu_bundle(app: AppHandle, output_path: String) -> Result<String, String> {
    let reminders = crate::features::reminders::get_reminders(app.clone())?;
    let bundle = UwuBundle {
        manifest: UwuBundleManifest {
            version: BUNDLE_VERSION.to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
        },
        app_config: config::read_config(&app)?,
        ai_config: read_store_value(&app, "ai-config")?,
        tts_config: read_store_value(&app, "tts-config")?,
        reminders,
    };

    let file = File::create(&output_path).map_err(|error| error.to_string())?;
    let mut writer = ZipWriter::new(file);
    let json = serde_json::to_string_pretty(&bundle).map_err(|error| error.to_string())?;
    zip_add_file(&mut writer, "manifest.json", json.as_bytes())?;
    writer.finish().map_err(|error| error.to_string())?;
    Ok(output_path)
}

#[tauri::command]
pub fn import_uwu_bundle(app: AppHandle, input_path: String) -> Result<AppConfig, String> {
    let file = File::open(&input_path).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    let mut manifest_bytes = Vec::new();
    archive
        .by_name("manifest.json")
        .map_err(|error| error.to_string())?
        .read_to_end(&mut manifest_bytes)
        .map_err(|error| error.to_string())?;

    let bundle: UwuBundle =
        serde_json::from_slice(&manifest_bytes).map_err(|error| error.to_string())?;

    if bundle.manifest.version != BUNDLE_VERSION {
        return Err(format!(
            "Unsupported bundle version: {}",
            bundle.manifest.version
        ));
    }

    let saved = config::save_config(&app, &bundle.app_config)?;

    let store = app.store(STORE_NAME).map_err(|error| error.to_string())?;
    if !bundle.ai_config.is_null() {
        store.set("ai-config", bundle.ai_config);
    }
    if !bundle.tts_config.is_null() {
        store.set("tts-config", bundle.tts_config);
    }
    store.save().map_err(|error| error.to_string())?;

    crate::features::reminders::replace_reminders(&app, bundle.reminders)?;

    app.emit(SKIN_CHANGED, &saved.active_skin)
        .map_err(|error| error.to_string())?;
    app.emit(THEME_CHANGED, &saved.active_theme)
        .map_err(|error| error.to_string())?;
    app.emit(SOUND_PACK_CHANGED, &saved.active_sound_pack)
        .map_err(|error| error.to_string())?;
    app.emit(BUNDLE_IMPORTED, &saved)
        .map_err(|error| error.to_string())?;

    Ok(saved)
}
