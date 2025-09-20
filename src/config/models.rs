use memoize::memoize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_api_url() -> String {
    "https://api.openai.com/v1/".to_string()
}
fn default_model() -> String {
    "gpt-4o".to_string()
}
fn default_max_tokens() -> i32 {
    8192
}
fn default_temperature() -> f32 {
    0.6
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct AppConfig {
    #[serde(default = "default_api_url")]
    pub api_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            api_url: default_api_url(),
            model: default_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

fn get_config(config_path: &PathBuf) -> AppConfig {
    match fs::read_to_string(config_path) {
        Ok(content) => match serde_yaml::from_str(&content) {
            Ok(config) => {
                println!("Configuration loaded and merged from: {:?}", config_path);
                config
            },
            Err(e) => {
                eprintln!(
                    "Error parsing config file {:?}: {}. Using default configuration.",
                    config_path, e
                );
                AppConfig::default()
            },
        },
        Err(e) => {
            eprintln!(
                "Error reading config file {:?}: {}. Using default configuration.",
                config_path, e
            );
            AppConfig::default()
        },
    }
}

#[memoize]
pub fn load_config() -> AppConfig {
    let mut config_path: PathBuf = match dirs::config_dir() {
        Some(path) => path,
        None => {
            eprintln!("Could not determine XDG config directory. Using default configuration.");
            return AppConfig::default();
        },
    };

    println!("config_path: {:?}", config_path);

    config_path.push("pipe-gpt");
    config_path.push("config.yaml");

    get_config(&config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn setup_temp_config_env() -> (PathBuf, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let config_app_dir = temp_dir.path().join("pipe-gpt");
        std::fs::create_dir_all(&config_app_dir).unwrap();
        let config_file_path = config_app_dir.join("config.yaml");
        (config_file_path, temp_dir)
    }

    fn write_config_to_temp_file(config_file_path: PathBuf, content: &str) -> PathBuf {
        let mut file = std::fs::File::create(&config_file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        config_file_path
    }

    fn teardown_temp_config(temp_dir: tempfile::TempDir) {
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_load_config_from_full_file() {
        let config_content = r#"
model: "gpt-3.5-turbo"
api_url: "https://api.openai.com/v1/new_for_test"
max_tokens: 2048
temperature: 0.8
        "#;
        let (config_app_dir, temp_dir) = setup_temp_config_env();
        write_config_to_temp_file(config_app_dir.clone(), config_content);

        let loaded_config = get_config(&config_app_dir);

        assert_eq!(loaded_config.model, "gpt-3.5-turbo");
        assert_eq!(
            loaded_config.api_url,
            "https://api.openai.com/v1/new_for_test"
        );
        assert_eq!(loaded_config.max_tokens, 2048);
        assert_eq!(loaded_config.temperature, 0.8);

        teardown_temp_config(temp_dir);
    }

    #[test]
    fn test_load_config_partial_file_merges_defaults() {
        let config_content = r#"
model: custom-model
max_tokens: 1000
        "#;
        let (config_app_dir, temp_dir) = setup_temp_config_env();
        write_config_to_temp_file(config_app_dir.clone(), config_content);

        let loaded_config = get_config(&config_app_dir);

        assert_eq!(loaded_config.model, "custom-model");
        assert_eq!(loaded_config.max_tokens, 1000);
        assert_eq!(loaded_config.temperature, AppConfig::default().temperature);

        teardown_temp_config(temp_dir);
    }

    #[test]
    fn test_load_config_default_if_not_found() {
        let (config_app_dir, temp_dir) = setup_temp_config_env();

        let loaded_config = get_config(&config_app_dir);
        assert_eq!(loaded_config, AppConfig::default());

        teardown_temp_config(temp_dir);
    }

    #[test]
    fn test_load_config_default_if_malformed() {
        let config_content = r#"
model: "malformed"
max_tokens: not-a-number # This will cause a deserialization error
temperature: 0.5
        "#;
        let (config_app_dir, temp_dir) = setup_temp_config_env();
        write_config_to_temp_file(config_app_dir.clone(), config_content);

        let loaded_config = get_config(&config_app_dir);
        assert_eq!(loaded_config, AppConfig::default());

        teardown_temp_config(temp_dir);
    }

    #[test]
    fn test_load_config_empty_file_uses_all_defaults() {
        let config_content = r#""#;
        let (config_app_dir, temp_dir) = setup_temp_config_env();
        write_config_to_temp_file(config_app_dir.clone(), config_content);

        let loaded_config = get_config(&config_app_dir);
        assert_eq!(loaded_config, AppConfig::default());

        teardown_temp_config(temp_dir);
    }
}
