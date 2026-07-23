use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use zip::ZipArchive;

pub enum CharDataSource<'a> {
    Dir(&'a Path),
    Archive(&'a Path, ZipArchive<fs::File>),
}

impl<'a> CharDataSource<'a> {
    pub fn new(path: &'a Path) -> Option<Self> {
        if path.is_dir() {
            Some(Self::Dir(path))
        } else {
            let file = fs::File::open(path).ok()?;
            Some(Self::Archive(path, ZipArchive::new(file).ok()?))
        }
    }

    pub fn path(&self) -> &'a Path {
        match self {
            CharDataSource::Dir(path) => path,
            CharDataSource::Archive(path, _) => path,
        }
    }

    pub fn read_content(&mut self, name: &str) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Dir(dir) => Ok(fs::read(dir.join(name))?),
            Self::Archive(_, archive) => {
                let mut entry = archive.by_name(name)?;
                let mut data = Vec::new();
                io::Read::read_to_end(&mut entry, &mut data)?;
                Ok(data)
            }
        }
    }

    pub fn load_config(&mut self) -> Option<CharacterConfig> {
        let json_str = String::from_utf8(self.read_content("char.json").ok()?).ok()?;

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
        config.path = self.path().to_path_buf();
        Some(config)
    }
}

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

pub fn app_dir() -> Option<PathBuf> {
    #[cfg(debug_assertions)]
    {
        env::current_dir().ok()
    }
    #[cfg(not(debug_assertions))]
    {
        env::current_exe().ok()?.parent().map(PathBuf::from)
    }
}

pub fn find_characters_dir() -> Option<PathBuf> {
    let dir = app_dir()?.join("characters");
    dir.is_dir().then_some(dir)
}

pub fn load_char_list() -> Vec<CharacterConfig> {
    let chars_dir = match find_characters_dir() {
        Some(d) => d,
        None => return Vec::new(),
    };

    let entries: Vec<_> = match fs::read_dir(&chars_dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return Vec::new(),
    };

    let mut chars: Vec<CharacterConfig> = entries
        .iter()
        .filter_map(|e| {
            let path = e.path();
            CharDataSource::new(&path)?.load_config()
        })
        .collect();

    chars.sort_by(|a, b| a.name.cmp(&b.name));
    chars
}
