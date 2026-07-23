use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct CharacterConfig {
    pub name: String,
    pub action_text: Option<String>,
    pub images: CharacterImages,
    pub sounds: Vec<String>,
    pub sound_random_weights: Option<Vec<f64>>,
    pub pitch: Option<Vec<f64>>,
    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Deserialize)]
pub struct CharacterImages {
    pub idle: CharacterImage,
    pub speaking: CharacterImage,
}

#[derive(Deserialize)]
pub struct CharacterImage {
    pub file: String,
    pub pixel_art: bool,
}

fn find_characters_dir() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let dir = parent.join("characters");
            if dir.is_dir() {
                return Some(dir);
            }
        }
    }
    let dir = PathBuf::from("characters");
    if dir.is_dir() { Some(dir) } else { None }
}

pub fn load_char_list() -> Vec<CharacterConfig> {
    let chars_dir = match find_characters_dir() {
        Some(d) => d,
        None => return Vec::new(),
    };

    let entries: Vec<_> = match std::fs::read_dir(&chars_dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return Vec::new(),
    };

    let mut chars: Vec<CharacterConfig> = entries
        .iter()
        .filter_map(|e| {
            let path = e.path();
            if path.is_dir() {
                load_char_config(&path)
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("chara"))
            {
                load_char_config(&path)
            } else {
                None
            }
        })
        .collect();

    chars.sort_by(|a, b| a.name.cmp(&b.name));
    chars
}

fn load_char_config(path: &Path) -> Option<CharacterConfig> {
    let json_str = if path.is_dir() {
        std::fs::read_to_string(path.join("char.json")).ok()?
    } else {
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("chara"))
        {
            return None;
        }
        let file = std::fs::File::open(path).ok()?;
        let mut archive = zip::ZipArchive::new(file).ok()?;
        let mut entry = archive.by_name("char.json").ok()?;
        let mut s = String::new();
        std::io::Read::read_to_string(&mut entry, &mut s).ok()?;
        s
    };

    let mut config: CharacterConfig = serde_json::from_str(&json_str).ok()?;
    if let Some(pitch) = &config.pitch {
        if pitch.iter().any(|&p| p <= 0.0) {
            return None;
        }
    }
    if let Some(weights) = &config.sound_random_weights {
        if weights.len() != config.sounds.len() || weights.iter().any(|&w| w < 0.0) {
            return None;
        }
    }
    config.path = path.to_path_buf();
    Some(config)
}

pub enum CharAssetSource<'a> {
    Dir(&'a Path),
    Archive(zip::ZipArchive<std::fs::File>),
}

pub fn get_char_asset_source(config: &CharacterConfig) -> Option<CharAssetSource<'_>> {
    if config.path.is_dir() {
        Some(CharAssetSource::Dir(&config.path))
    } else {
        let file = std::fs::File::open(&config.path).ok()?;
        Some(CharAssetSource::Archive(zip::ZipArchive::new(file).ok()?))
    }
}

pub fn read_char_asset(source: &mut CharAssetSource, name: &str) -> anyhow::Result<Vec<u8>> {
    match source {
        CharAssetSource::Dir(dir) => Ok(std::fs::read(dir.join(name))?),
        CharAssetSource::Archive(archive) => {
            let mut entry = archive.by_name(name)?;
            let mut data = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut data)?;
            Ok(data)
        }
    }
}
