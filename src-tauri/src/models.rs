use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub id: String,
    pub title: String,
    pub body: String,
    pub alias: Option<String>,
    pub notes: Option<String>,
    pub is_favorite: bool,
    pub usage_count: i64,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptInput {
    pub title: String,
    pub body: String,
    pub alias: Option<String>,
    pub notes: Option<String>,
    pub is_favorite: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptUpdateInput {
    pub id: String,
    pub title: String,
    pub body: String,
    pub alias: Option<String>,
    pub notes: Option<String>,
    pub is_favorite: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEnvironment {
    pub app_version: String,
    pub tauri_version: String,
    pub database_path: String,
    pub data_dir: String,
    pub logs_dir: String,
    pub exports_dir: String,
    pub backups_dir: String,
    pub launch_at_login: bool,
    pub accessibility_trusted: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFile {
    pub path: String,
    pub bytes: u64,
}
