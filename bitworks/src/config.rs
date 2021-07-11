use serde::Deserialize;

pub const TILE_SIZE: f32 = 48.0;
pub const TILE_HALFSIZE: f32 = 24.0;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub log_diagnostics: bool,
}

impl Config {
    pub fn from_ron(path: &str) -> Result<Self, ron::Error> {
        let file = std::fs::File::open(path)?;
        ron::de::from_reader(file)
    }
}
