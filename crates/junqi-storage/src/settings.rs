use junqi_core::types::AiDifficulty;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::layout_store::dirs_next;

fn settings_path() -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    path.push(".junqi");
    path.push("settings.json");
    path
}

/// 用户偏好设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub ai_difficulty: AiDifficulty,
    pub sound_enabled: bool,
    pub player_name: String,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            ai_difficulty: AiDifficulty::Medium,
            sound_enabled: true,
            player_name: "玩家".to_string(),
            theme: "classic".to_string(),
        }
    }
}

/// 加载用户设置
pub fn load_settings() -> Settings {
    let path = settings_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Settings::default(),
        }
    } else {
        Settings::default()
    }
}

/// 保存用户设置
pub fn save_settings(settings: &Settings) -> Result<(), SettingsError> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| SettingsError::Io(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| SettingsError::Serialize(e.to_string()))?;
    fs::write(&path, json).map_err(|e| SettingsError::Io(e.to_string()))?;
    Ok(())
}

/// 用户设置错误类型
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("文件操作错误: {0}")]
    Io(String),
    #[error("序列化错误: {0}")]
    Serialize(String),
}
