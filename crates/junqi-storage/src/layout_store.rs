use junqi_core::layout::{builtin_layouts, Layout};
use std::fs;
use std::path::PathBuf;

/// 获取用户主目录
pub fn dirs_next() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .ok()
}

fn layouts_dir() -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    path.push(".junqi");
    path.push("layouts");
    path
}

/// 保存自定义布阵
pub fn save_layout(layout: &Layout) -> Result<(), LayoutStoreError> {
    let dir = layouts_dir();
    fs::create_dir_all(&dir).map_err(|e| LayoutStoreError::Io(e.to_string()))?;

    let filename = sanitize_filename(&layout.name);
    let file_path = dir.join(format!("{}.json", filename));

    let json = serde_json::to_string_pretty(layout)
        .map_err(|e| LayoutStoreError::Serialize(e.to_string()))?;

    fs::write(&file_path, json).map_err(|e| LayoutStoreError::Io(e.to_string()))?;
    Ok(())
}

/// 加载所有用户自定义布阵
pub fn load_custom_layouts() -> Result<Vec<Layout>, LayoutStoreError> {
    let dir = layouts_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut layouts = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| LayoutStoreError::Io(e.to_string()))?;

    for entry in entries {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(&path).map_err(|e| LayoutStoreError::Io(e.to_string()))?;
            let layout: Layout = serde_json::from_str(&content)
                .map_err(|e| LayoutStoreError::Deserialize(e.to_string()))?;
            layouts.push(layout);
        }
    }

    Ok(layouts)
}

/// 获取所有布阵（内置 + 自定义）
pub fn all_layouts() -> Result<Vec<Layout>, LayoutStoreError> {
    let mut layouts = builtin_layouts();
    let custom = load_custom_layouts()?;
    layouts.extend(custom);
    Ok(layouts)
}

/// 删除指定名称的自定义布阵
pub fn delete_layout(name: &str) -> Result<(), LayoutStoreError> {
    let dir = layouts_dir();
    let filename = sanitize_filename(name);
    let file_path = dir.join(format!("{}.json", filename));
    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| LayoutStoreError::Io(e.to_string()))?;
    }
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' { c } else { '_' })
        .collect()
}

/// 布阵存储错误类型
#[derive(Debug, thiserror::Error)]
pub enum LayoutStoreError {
    #[error("文件操作错误: {0}")]
    Io(String),
    #[error("序列化错误: {0}")]
    Serialize(String),
    #[error("反序列化错误: {0}")]
    Deserialize(String),
}
