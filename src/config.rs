use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub presets: HashMap<String, Preset>,
    
    #[serde(default)]
    pub ui: UiConfig,
    
    #[serde(default)]
    pub search: SearchDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub description: String,
    pub pattern: Option<String>,
    pub file_types: Vec<String>,
    pub ignore_case: bool,
    pub hidden: bool,
    pub glob: Vec<String>,
    pub max_depth: Option<usize>,
    pub additional_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    
    #[serde(default = "default_true")]
    pub syntax_highlighting: bool,
    
    #[serde(default = "default_preview_lines")]
    pub preview_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDefaults {
    #[serde(default)]
    pub ignore_case: bool,
    
    #[serde(default = "default_true")]
    pub smart_case: bool,
    
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

fn default_preview_lines() -> usize {
    5
}

fn default_max_results() -> usize {
    1000
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            syntax_highlighting: true,
            preview_lines: default_preview_lines(),
        }
    }
}

impl Default for SearchDefaults {
    fn default() -> Self {
        Self {
            ignore_case: false,
            smart_case: true,
            max_results: default_max_results(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Self::default_config();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        
        Ok(config_dir.join("ripgrep-tui").join("config.toml"))
    }

    fn default_config() -> Self {
        let mut presets = HashMap::new();
        
        // Add some useful default presets
        presets.insert("rust".to_string(), Preset {
            description: "Search in Rust files".to_string(),
            pattern: None,
            file_types: vec!["rust".to_string()],
            ignore_case: false,
            hidden: false,
            glob: vec![],
            max_depth: None,
            additional_args: vec![],
        });
        
        presets.insert("code".to_string(), Preset {
            description: "Search in common code files".to_string(),
            pattern: None,
            file_types: vec!["rust".to_string(), "python".to_string(), "js".to_string(), 
                           "ts".to_string(), "go".to_string(), "cpp".to_string()],
            ignore_case: false,
            hidden: false,
            glob: vec![],
            max_depth: None,
            additional_args: vec![],
        });
        
        presets.insert("docs".to_string(), Preset {
            description: "Search in documentation files".to_string(),
            pattern: None,
            file_types: vec!["md".to_string(), "txt".to_string(), "rst".to_string()],
            ignore_case: true,
            hidden: false,
            glob: vec![],
            max_depth: None,
            additional_args: vec![],
        });

        Self {
            presets,
            ui: UiConfig::default(),
            search: SearchDefaults::default(),
        }
    }
}
