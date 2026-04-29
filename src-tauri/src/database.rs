use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::models::{Prompt, PromptInput, PromptUpdateInput, Setting, Tag};

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub db_path: PathBuf,
    pub data_dir: PathBuf,
}

pub fn open_database(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let conn = Connection::open(path).map_err(|error| error.to_string())?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS prompts (
          id TEXT PRIMARY KEY,
          title TEXT NOT NULL,
          body TEXT NOT NULL,
          alias TEXT,
          notes TEXT,
          is_favorite INTEGER NOT NULL DEFAULT 0,
          usage_count INTEGER NOT NULL DEFAULT 0,
          last_used_at TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tags (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL UNIQUE,
          color TEXT,
          created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS prompt_tags (
          prompt_id TEXT NOT NULL,
          tag_id TEXT NOT NULL,
          PRIMARY KEY (prompt_id, tag_id),
          FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE,
          FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
          key TEXT PRIMARY KEY,
          value TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_prompts_title ON prompts(title);
        CREATE INDEX IF NOT EXISTS idx_prompts_alias ON prompts(alias);
        CREATE INDEX IF NOT EXISTS idx_prompts_favorite ON prompts(is_favorite);
        CREATE INDEX IF NOT EXISTS idx_prompts_last_used ON prompts(last_used_at);
        ",
    )
    .map_err(|error| error.to_string())
}

pub fn now() -> String {
    Utc::now().to_rfc3339()
}

pub fn list_prompts(conn: &Connection) -> Result<Vec<Prompt>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, title, body, alias, notes, is_favorite, usage_count,
                   last_used_at, created_at, updated_at
            FROM prompts
            ORDER BY is_favorite DESC, usage_count DESC, updated_at DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Prompt {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                alias: row.get(3)?,
                notes: row.get(4)?,
                is_favorite: row.get::<_, i64>(5)? != 0,
                usage_count: row.get(6)?,
                last_used_at: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                tags: Vec::new(),
            })
        })
        .map_err(|error| error.to_string())?;

    let mut prompts = Vec::new();
    for row in rows {
        let mut prompt = row.map_err(|error| error.to_string())?;
        prompt.tags = tags_for_prompt(conn, &prompt.id)?;
        prompts.push(prompt);
    }

    Ok(prompts)
}

pub fn list_tags(conn: &Connection) -> Result<Vec<Tag>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, created_at FROM tags ORDER BY name ASC")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn create_prompt(conn: &Connection, input: PromptInput) -> Result<Prompt, String> {
    validate_prompt(&input.title, &input.body)?;

    let id = Uuid::new_v4().to_string();
    let timestamp = now();
    conn.execute(
        "
        INSERT INTO prompts
          (id, title, body, alias, notes, is_favorite, usage_count, last_used_at, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, NULL, ?7, ?7)
        ",
        params![
            &id,
            input.title.trim(),
            input.body.trim(),
            clean_option(input.alias),
            clean_option(input.notes),
            bool_to_int(input.is_favorite),
            &timestamp
        ],
    )
    .map_err(|error| error.to_string())?;

    replace_prompt_tags(conn, &id, &input.tags)?;
    get_prompt(conn, &id)
}

pub fn update_prompt(conn: &Connection, input: PromptUpdateInput) -> Result<Prompt, String> {
    validate_prompt(&input.title, &input.body)?;

    let changed = conn
        .execute(
            "
            UPDATE prompts
            SET title = ?2,
                body = ?3,
                alias = ?4,
                notes = ?5,
                is_favorite = ?6,
                updated_at = ?7
            WHERE id = ?1
            ",
            params![
                &input.id,
                input.title.trim(),
                input.body.trim(),
                clean_option(input.alias),
                clean_option(input.notes),
                bool_to_int(input.is_favorite),
                now()
            ],
        )
        .map_err(|error| error.to_string())?;

    if changed == 0 {
        return Err("Prompt not found".to_string());
    }

    replace_prompt_tags(conn, &input.id, &input.tags)?;
    get_prompt(conn, &input.id)
}

pub fn delete_prompt(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM prompts WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn mark_used(conn: &Connection, id: &str) -> Result<Prompt, String> {
    let changed = conn
        .execute(
            "
            UPDATE prompts
            SET usage_count = usage_count + 1,
                last_used_at = ?2,
                updated_at = ?2
            WHERE id = ?1
            ",
            params![id, now()],
        )
        .map_err(|error| error.to_string())?;

    if changed == 0 {
        return Err("Prompt not found".to_string());
    }

    get_prompt(conn, id)
}

pub fn get_prompt(conn: &Connection, id: &str) -> Result<Prompt, String> {
    let mut prompt = conn
        .query_row(
            "
            SELECT id, title, body, alias, notes, is_favorite, usage_count,
                   last_used_at, created_at, updated_at
            FROM prompts
            WHERE id = ?1
            ",
            params![id],
            |row| {
                Ok(Prompt {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    body: row.get(2)?,
                    alias: row.get(3)?,
                    notes: row.get(4)?,
                    is_favorite: row.get::<_, i64>(5)? != 0,
                    usage_count: row.get(6)?,
                    last_used_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    tags: Vec::new(),
                })
            },
        )
        .map_err(|error| error.to_string())?;

    prompt.tags = tags_for_prompt(conn, &prompt.id)?;
    Ok(prompt)
}

pub fn upsert_tag(conn: &Connection, name: String, color: Option<String>) -> Result<Tag, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Tag name is required".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let timestamp = now();
    conn.execute(
        "
        INSERT INTO tags (id, name, color, created_at)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(name) DO UPDATE SET color = excluded.color
        ",
        params![&id, &name, &color, &timestamp],
    )
    .map_err(|error| error.to_string())?;

    get_tag_by_name(conn, &name)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<Setting>, String> {
    conn.query_row(
        "SELECT key, value, updated_at FROM settings WHERE key = ?1",
        params![key],
        |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())
}

pub fn set_setting(conn: &Connection, key: String, value: String) -> Result<Setting, String> {
    if key.trim().is_empty() {
        return Err("Setting key is required".to_string());
    }

    let timestamp = now();
    conn.execute(
        "
        INSERT INTO settings (key, value, updated_at)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        ",
        params![&key, &value, &timestamp],
    )
    .map_err(|error| error.to_string())?;

    get_setting(conn, &key)?.ok_or_else(|| "Setting not found".to_string())
}

fn validate_prompt(title: &str, body: &str) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("Prompt title is required".to_string());
    }
    if body.trim().is_empty() {
        return Err("Prompt body is required".to_string());
    }
    Ok(())
}

fn clean_option(value: Option<String>) -> Option<String> {
    value.and_then(|inner| {
        let trimmed = inner.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn bool_to_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn tags_for_prompt(conn: &Connection, prompt_id: &str) -> Result<Vec<Tag>, String> {
    let mut stmt = conn
        .prepare(
            "
            SELECT tags.id, tags.name, tags.color, tags.created_at
            FROM tags
            INNER JOIN prompt_tags ON tags.id = prompt_tags.tag_id
            WHERE prompt_tags.prompt_id = ?1
            ORDER BY tags.name ASC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![prompt_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn replace_prompt_tags(conn: &Connection, prompt_id: &str, names: &[String]) -> Result<(), String> {
    conn.execute(
        "DELETE FROM prompt_tags WHERE prompt_id = ?1",
        params![prompt_id],
    )
    .map_err(|error| error.to_string())?;

    for name in normalized_tag_names(names) {
        let tag = upsert_tag(conn, name, None)?;
        conn.execute(
            "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
            params![prompt_id, tag.id],
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn normalized_tag_names(names: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for name in names {
        let trimmed = name.trim();
        if !trimmed.is_empty()
            && !normalized
                .iter()
                .any(|known: &String| known.eq_ignore_ascii_case(trimmed))
        {
            normalized.push(trimmed.to_string());
        }
    }
    normalized
}

fn get_tag_by_name(conn: &Connection, name: &str) -> Result<Tag, String> {
    conn.query_row(
        "SELECT id, name, color, created_at FROM tags WHERE name = ?1",
        params![name],
        |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory database opens");
        init_schema(&conn).expect("schema initializes");
        conn
    }

    fn prompt_input(title: &str, tags: Vec<&str>) -> PromptInput {
        PromptInput {
            title: title.to_string(),
            body: format!("Body for {title}"),
            alias: Some(format!("{title}-alias")),
            notes: Some("note".to_string()),
            is_favorite: true,
            tags: tags.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn prompt_lifecycle_tracks_tags_usage_settings_and_delete() {
        let conn = conn();

        let created = create_prompt(
            &conn,
            prompt_input("qa", vec!["review", "review", " coding "]),
        )
        .expect("prompt is created");
        assert_eq!(created.title, "qa");
        assert_eq!(created.alias.as_deref(), Some("qa-alias"));
        assert!(created.is_favorite);
        assert_eq!(created.tags.len(), 2);

        let listed = list_prompts(&conn).expect("prompts list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);

        let updated = update_prompt(
            &conn,
            PromptUpdateInput {
                id: created.id.clone(),
                title: "qa updated".to_string(),
                body: "updated body".to_string(),
                alias: Some("updated".to_string()),
                notes: None,
                is_favorite: false,
                tags: vec!["ops".to_string()],
            },
        )
        .expect("prompt is updated");
        assert_eq!(updated.title, "qa updated");
        assert!(!updated.is_favorite);
        assert_eq!(updated.tags.len(), 1);
        assert_eq!(updated.tags[0].name, "ops");

        let used = mark_used(&conn, &updated.id).expect("prompt is marked used");
        assert_eq!(used.usage_count, 1);
        assert!(used.last_used_at.is_some());

        let setting = set_setting(&conn, "language".to_string(), "en".to_string())
            .expect("setting is written");
        assert_eq!(setting.key, "language");
        assert_eq!(
            get_setting(&conn, "language")
                .expect("setting lookup works")
                .expect("setting exists")
                .value,
            "en"
        );

        delete_prompt(&conn, &updated.id).expect("prompt is deleted");
        assert!(list_prompts(&conn)
            .expect("prompts list after delete")
            .is_empty());
    }

    #[test]
    fn prompt_validation_rejects_missing_required_fields() {
        let conn = conn();

        let missing_title = create_prompt(
            &conn,
            PromptInput {
                title: " ".to_string(),
                body: "body".to_string(),
                alias: None,
                notes: None,
                is_favorite: false,
                tags: Vec::new(),
            },
        );
        assert_eq!(missing_title.unwrap_err(), "Prompt title is required");

        let missing_body = create_prompt(
            &conn,
            PromptInput {
                title: "title".to_string(),
                body: " ".to_string(),
                alias: None,
                notes: None,
                is_favorite: false,
                tags: Vec::new(),
            },
        );
        assert_eq!(missing_body.unwrap_err(), "Prompt body is required");
    }
}
