use serde::{Serialize, Deserialize};
use serde_json;
use std::env;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::application::gactions::Theme;

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum UserSettingsKey {
    Theme,
}

#[derive(Serialize, Deserialize)]
pub enum UserSettingsValue {
    ThemeValue(Theme)
}

pub struct UserSettings {
    user_settings: HashMap<UserSettingsKey, UserSettingsValue>,
    config_name: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        let mut instance = UserSettings::new();

        if let Err(load_config_err) = instance.load_config() {
            match instance.create_config() {
                Ok(_) => (),
                Err(create_config_err) => {
                    println!("Failed to create config, saving will not be enabled");
                }
            }
        }

        instance
    }
}

impl UserSettings {
    fn new() -> Self {
        Self {
            user_settings: HashMap::new(),
            config_name: "config.json".to_string(),
        }
    }

    pub fn get_setting(&self, key: UserSettingsKey) -> Option<&UserSettingsValue> {
        self.user_settings.get(&key)
    }

    pub fn set_setting(&mut self, key: UserSettingsKey, value: UserSettingsValue) {
        self.user_settings.insert(key, value);
    }

    pub fn save_config(&self) -> Result<(), String> {
        let config_file_path = self.get_config_file_path();
        let json_text = serde_json::to_string(&self.user_settings)
            .map_err(|err|format!("Could not read config file: {}", err))?;

        std::fs::write(&config_file_path, json_text)
            .map_err(|err|format!("Could not write to config file: {}", err))?;
        Ok(())
    }

    pub fn load_config(&mut self) -> Result<(), String> {
        let config_file_path = self.get_config_file_path();
        let json_text = std::fs::read_to_string(&config_file_path)
            .map_err(|err|format!("Could not write to config file: {}", err))?;

        self.user_settings = serde_json::from_str(&json_text)
            .map_err(|err|format!("Could not read config file: {}", err))?;
        Ok(())
    }

    pub fn create_config(&mut self) -> Result<(), String> {
        // Once more settings are added, we can consider creating a "Default" trait to add these programmatically.
        self.user_settings.insert(UserSettingsKey::Theme, UserSettingsValue::ThemeValue(Theme::SystemPreference));
        self.save_config()?;
        Ok(())
    }

    // TODO: Consider retrieving this from a crate like dirs
    fn get_config_file_path(&self) -> PathBuf {
        let exe_path = env::current_exe().expect("Error: Could not determine file path");

        let mut current_path = exe_path.clone();
        current_path.pop();
        current_path.push(&self.config_name);

        current_path
    }
}

