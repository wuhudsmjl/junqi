use chrono::Utc;
use junqi_core::replay::Replay;
use std::fs;
use std::path::PathBuf;

use crate::layout_store::dirs_next;

/// 未保存复盘滑动窗口大小
const UNSAVED_WINDOW: usize = 20;

fn replays_dir() -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    path.push(".junqi");
    path.push("replays");
    path
}

/// 保存复盘记录
pub fn save_replay(replay: &Replay) -> Result<(), ReplayStoreError> {
    let dir = replays_dir();
    fs::create_dir_all(&dir).map_err(|e| ReplayStoreError::Io(e.to_string()))?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}_vs_{}.json", timestamp, replay.red_name, replay.blue_name);
    let file_path = dir.join(filename);

    let json = serde_json::to_string_pretty(replay)
        .map_err(|e| ReplayStoreError::Serialize(e.to_string()))?;

    fs::write(&file_path, json).map_err(|e| ReplayStoreError::Io(e.to_string()))?;

    cleanup_unsaved()?;

    Ok(())
}

/// 将复盘标记为已保存/未保存（重写文件）
pub fn set_saved(filename: &str, saved: bool) -> Result<(), ReplayStoreError> {
    let file_path = replays_dir().join(format!("{}.json", filename));
    let content = fs::read_to_string(&file_path).map_err(|e| ReplayStoreError::Io(e.to_string()))?;
    let mut replay: Replay = serde_json::from_str(&content)
        .map_err(|e| ReplayStoreError::Deserialize(e.to_string()))?;
    replay.saved = saved;
    let json = serde_json::to_string_pretty(&replay)
        .map_err(|e| ReplayStoreError::Serialize(e.to_string()))?;
    fs::write(&file_path, json).map_err(|e| ReplayStoreError::Io(e.to_string()))?;
    Ok(())
}

/// 删除旧的未保存复盘，仅保留最近 UNSAVED_WINDOW 场
fn cleanup_unsaved() -> Result<(), ReplayStoreError> {
    let replays = load_replays()?;
    let unsaved: Vec<&ReplayInfo> = replays.iter().filter(|r| !r.replay.saved).collect();
    if unsaved.len() > UNSAVED_WINDOW {
        let mut sorted = unsaved.clone();
        sorted.sort_by(|a, b| b.filename.cmp(&a.filename));
        for info in &sorted[UNSAVED_WINDOW..] {
            let file_path = replays_dir().join(format!("{}.json", info.filename));
            let _ = fs::remove_file(&file_path);
        }
    }
    Ok(())
}

/// 加载所有已保存的复盘记录
pub fn load_replays() -> Result<Vec<ReplayInfo>, ReplayStoreError> {
    let dir = replays_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut replays = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| ReplayStoreError::Io(e.to_string()))?;

    for entry in entries {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(&path).map_err(|e| ReplayStoreError::Io(e.to_string()))?;
            let replay: Replay = serde_json::from_str(&content)
                .map_err(|e| ReplayStoreError::Deserialize(e.to_string()))?;
            replays.push(ReplayInfo {
                filename: path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
                replay,
            });
        }
    }

    replays.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(replays)
}

/// 删除复盘记录
pub fn delete_replay(filename: &str) -> Result<(), ReplayStoreError> {
    let dir = replays_dir();
    let file_path = dir.join(format!("{}.json", filename));
    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| ReplayStoreError::Io(e.to_string()))?;
    }
    Ok(())
}

/// 复盘信息
#[derive(Debug, Clone)]
pub struct ReplayInfo {
    pub filename: String,
    pub replay: Replay,
}

/// 复盘存储错误类型
#[derive(Debug, thiserror::Error)]
pub enum ReplayStoreError {
    #[error("文件操作错误: {0}")]
    Io(String),
    #[error("序列化错误: {0}")]
    Serialize(String),
    #[error("反序列化错误: {0}")]
    Deserialize(String),
}
