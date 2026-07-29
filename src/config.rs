use iced::Theme;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WindowSize {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) qtor_url: String,
    pub(crate) qtor_username: String,
    pub(crate) qtor_pass: String,
    pub(crate) uses_custom_titlebar: bool,
    pub(crate) window_size: WindowSize,
    #[serde(default, with = "theme_serde")]
    pub(crate) theme: Option<Theme>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigError {
    #[error("config file io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

mod theme_serde {
    use iced::Theme;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(theme: &Option<Theme>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        theme
            .as_ref()
            .map(ToString::to_string) // ← repo: theme.to_string() (snapshot filenames)
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Theme>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = Option::<String>::deserialize(deserializer)?;
        Ok(name.and_then(|name| {
            Theme::ALL
                .iter()
                .find(|theme| theme.to_string() == name) // ← repo: Theme::ALL.iter().position(...)
                .cloned()
        }))
    }
}

impl Config {
    fn path() -> PathBuf {
        dirs::config_dir()
            .expect("config dir")
            .join("nyaa-rs")
            .join("config.toml")
    }

    pub(crate) fn load() -> Option<Config> {
        let path = Self::path();
        match fs::read_to_string(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                warn!("failed to read {}: {e}", path.display());
                None
            }
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    info!("loaded config from {}", path.display());
                    Some(config)
                }
                Err(e) => {
                    warn!("failed to parse {}: {e}", path.display());
                    None
                }
            },
        }
    }

    pub(crate) fn save(&self) -> Result<(), ConfigError> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            qtor_url: "http://localhost:8080".into(),
            qtor_username: "admin".into(),
            qtor_pass: "adminadmin".into(),
            theme: Some(Theme::Ferra),
            uses_custom_titlebar: false,
            window_size: WindowSize {
                width: 1280.0,
                height: 720.0,
            },
        }
    }
}
