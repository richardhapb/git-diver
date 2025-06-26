use std::{fmt::Display, fs::File, io::Read};

use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct Config {
    repos: Vec<Repo>,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub author_email: String,
    pub path: String,
    pub branch: String,
}

struct ConfigPath(String);

impl Default for ConfigPath {
    fn default() -> Self {
        let config_dir = dirs::home_dir().expect("config dire resolution");
        debug!(?config_dir, "Config dir found");
        let full_path = config_dir
            .join(".config")
            .join("git-diver")
            .join("config.toml");

        debug!(?full_path, "Config file");

        let path_str = full_path.to_string_lossy().to_string();

        debug!(?path_str, "Resolved config file");

        Self(path_str)
    }
}

impl Display for ConfigPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ConfigPath {
    fn new(path: Option<&str>) -> Self {
        match path {
            Some(path) => Self(path.to_string()),
            None => Self::default(),
        }
    }
}

impl Config {
    pub fn from_file(path: Option<&str>) -> Result<Self, std::io::Error> {
        let config_path = ConfigPath::new(path);
        let mut file = File::open(config_path.to_string())?;
        let mut config_str = String::new();
        file.read_to_string(&mut config_str)?;

        let config: Self = toml::from_str(&config_str).unwrap();

        Ok(config)
    }

    pub fn get_repos(&self) -> &Vec<Repo> {
        &self.repos
    }
}
