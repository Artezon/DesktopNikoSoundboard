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
    pub dir: PathBuf,
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

fn load_char_config(dir: &Path) -> Option<CharacterConfig> {
    let json_path = dir.join("char.json");
    let json_str = std::fs::read_to_string(json_path).ok()?;
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
    config.dir = dir.to_path_buf();
    Some(config)
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
            if !path.is_dir() {
                return None;
            }
            load_char_config(&path)
        })
        .collect();

    chars.sort_by(|a, b| a.name.cmp(&b.name));
    chars
}
