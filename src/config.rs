use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub printer_name: String,
    pub label_width_mm: f64,
    pub label_height_mm: f64,
    pub region: Option<[i32; 4]>, // x, y, w, h
    pub print_enabled: bool,
    pub print_orientation: u8, // 0=Portrait, 1=Landscape, 2=Portrait 180°, 3=Landscape 180°
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            printer_name: String::new(),
            label_width_mm: 50.8,
            label_height_mm: 38.1,
            region: None,
            print_enabled: true,
            print_orientation: 0, // Default: Portrait
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_path = if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata).join("PVZHelper").join("config.json")
        } else {
            PathBuf::from("config.json")
        };

        Self { config_path }
    }

    pub fn load(&self) -> AppConfig {
        if let Ok(content) = fs::read_to_string(&self.config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        AppConfig::default()
    }

    pub fn save(&self, config: &AppConfig) -> anyhow::Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
}
