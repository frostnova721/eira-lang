use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct EssenceConfig {
    pub archive: ArchiveConfig,
}

#[derive(Deserialize, Debug)]
pub struct ArchiveConfig {
    pub name: String,
    pub incarnation: String,
    pub entry: String,
    pub root: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub incarnation: String,
    pub entry_point: String,
    pub root_dir: String,
    // pub dependencies: HashMap<String, String>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            incarnation: String::new(),
            entry_point: String::new(),
            root_dir: String::new(),
            // dependencies: HashMap::new(),
        }
    }

    pub fn load_from_toml(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let config: EssenceConfig = toml::from_str(&content)?;

        Ok(Self {
            name: config.archive.name,
            incarnation: config.archive.incarnation,
            entry_point: config.archive.entry,
            root_dir: config.archive.root.unwrap_or_else(|| ".".to_string()),
        })
    }

    // This function takes the starting path and crawls upwards looking for essence.toml
    pub fn find_root(start_path: &Path) -> Option<PathBuf> {
        let mut current_dir = if start_path.is_file() {
            start_path.parent().unwrap_or(Path::new("")).to_path_buf()
        } else {
            start_path.to_path_buf()
        };

        if current_dir.as_os_str().is_empty() {
            current_dir = PathBuf::from(".");
        }

        loop {
            let potential_config = current_dir.join("essence.toml");

            if potential_config.exists() {
                return Some(potential_config); // Found it!
            }
            if !current_dir.pop() {
                break; 
            }
        }

        None // No config found 
    }

    // pub fn add_file(&mut self, file_path: String) {
    //     self.files.push(file_path);
    // }

    // pub fn add_dependency(&mut self, name: String, version: String) {
    //     self.dependencies.insert(name, version);
    // }
}
