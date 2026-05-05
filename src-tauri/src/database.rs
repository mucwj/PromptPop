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

const STARTER_SNIPPET_VERSION: &str = "1";
const STARTER_SNIPPET_SETTING_KEY: &str = "starterSnippets.seededVersion";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StarterSnippetLocale {
    En,
    ZhCn,
}

impl StarterSnippetLocale {
    fn as_str(self) -> &'static str {
        match self {
            StarterSnippetLocale::En => "en",
            StarterSnippetLocale::ZhCn => "zh-CN",
        }
    }
}

struct StarterSnippet {
    title: &'static str,
    body: &'static str,
    alias: &'static str,
    notes: &'static str,
    is_favorite: bool,
    tags: &'static [&'static str],
}

const EN_STARTER_SNIPPETS: &[StarterSnippet] = &[
    StarterSnippet {
        title: "Rewrite for clarity",
        body: "Rewrite the following text to be clearer and more natural while preserving the meaning:",
        alias: "clarify",
        notes: "Short snippet for emails, docs, and product copy.",
        is_favorite: true,
        tags: &["Writing", "Rewrite", "Common"],
    },
    StarterSnippet {
        title: "Shorten text",
        body: "Make the following text shorter while keeping the key information:",
        alias: "shorten",
        notes: "Good for trimming long paragraphs.",
        is_favorite: true,
        tags: &["Writing", "Shorten", "Common"],
    },
    StarterSnippet {
        title: "Expand into a paragraph",
        body: "Turn the following notes into one complete, natural paragraph:",
        alias: "expand",
        notes: "Useful when turning rough points into prose.",
        is_favorite: false,
        tags: &["Writing", "Expand"],
    },
    StarterSnippet {
        title: "Make tone professional",
        body: "Rewrite this in a more professional but still natural tone:",
        alias: "tone",
        notes: "Useful for work messages and customer replies.",
        is_favorite: false,
        tags: &["Writing", "Tone"],
    },
    StarterSnippet {
        title: "Summarize key points",
        body: "Summarize the key points below using short bullets:",
        alias: "sum",
        notes: "Useful for articles, chats, and docs.",
        is_favorite: true,
        tags: &["Summary", "Reading", "Common"],
    },
    StarterSnippet {
        title: "Extract action items",
        body: "Extract action items as Task / Owner / Due date / Status:",
        alias: "todo",
        notes: "Useful for meeting notes and project syncs.",
        is_favorite: true,
        tags: &["Meeting", "Action items", "Summary"],
    },
    StarterSnippet {
        title: "Find risks and questions",
        body: "Point out issues, risks, and open questions in the following content:",
        alias: "risk",
        notes: "Useful for plans, PRDs, and feedback.",
        is_favorite: false,
        tags: &["Risk", "Analysis", "Product"],
    },
    StarterSnippet {
        title: "Explain a concept",
        body: "Explain the following in plain language and include one example:",
        alias: "explain",
        notes: "Useful for learning and technical concepts.",
        is_favorite: false,
        tags: &["Learning", "Explain"],
    },
    StarterSnippet {
        title: "Give 3 alternatives",
        body: "Give 3 alternative phrasings: concise, professional, and friendly:",
        alias: "alt",
        notes: "Useful for naming, copy, and replies.",
        is_favorite: false,
        tags: &["Writing", "Alternatives"],
    },
    StarterSnippet {
        title: "Proofread",
        body: "Check grammar, typos, and awkward phrasing, then provide a revised version:",
        alias: "proof",
        notes: "Useful before sending final text.",
        is_favorite: false,
        tags: &["Writing", "Proofread"],
    },
    StarterSnippet {
        title: "Translate to natural Chinese",
        body: "Translate this into natural Chinese while preserving tone and formatting:",
        alias: "zh",
        notes: "Useful for translating English into Chinese.",
        is_favorite: true,
        tags: &["Translation", "Chinese", "Common"],
    },
    StarterSnippet {
        title: "Translate to natural English",
        body: "Translate this into natural English while preserving tone and formatting:",
        alias: "en",
        notes: "Useful for translating Chinese drafts into English.",
        is_favorite: false,
        tags: &["Translation", "English"],
    },
    StarterSnippet {
        title: "Code review",
        body: "Review the code below. Prioritize bugs, edge cases, and missing tests:",
        alias: "review",
        notes: "Useful before submitting a change.",
        is_favorite: true,
        tags: &["Code", "Review", "Test"],
    },
    StarterSnippet {
        title: "Explain code",
        body: "Explain what this code does, focusing on inputs, outputs, and key logic:",
        alias: "code",
        notes: "Useful when reading unfamiliar code.",
        is_favorite: false,
        tags: &["Code", "Explain"],
    },
    StarterSnippet {
        title: "Generate test cases",
        body: "Generate tests for the function/module below, covering main branches and edge cases:",
        alias: "test",
        notes: "Useful when adding focused coverage.",
        is_favorite: false,
        tags: &["Code", "Test"],
    },
    StarterSnippet {
        title: "Debug analysis",
        body: "Analyze likely causes from the symptoms below and give the smallest debugging steps:",
        alias: "debug",
        notes: "Useful for logs and unexpected behavior.",
        is_favorite: false,
        tags: &["Code", "Debug"],
    },
    StarterSnippet {
        title: "Write a short reply",
        body: "Write a brief, polite reply that I can send directly:",
        alias: "reply",
        notes: "Useful for email, chat, and customer communication.",
        is_favorite: true,
        tags: &["Reply", "Work", "Common"],
    },
    StarterSnippet {
        title: "Turn into a checklist",
        body: "Organize the following content into a clear checklist:",
        alias: "list",
        notes: "Useful for cleaning up messy information.",
        is_favorite: false,
        tags: &["Organize", "Checklist"],
    },
];

const ZH_CN_STARTER_SNIPPETS: &[StarterSnippet] = &[
    StarterSnippet {
        title: "改写更清晰",
        body: "请把下面内容改写得更清晰、更自然，保留原意：",
        alias: "clarify",
        notes: "短片段。适合邮件、文档和产品文案。",
        is_favorite: true,
        tags: &["写作", "改写", "常用"],
    },
    StarterSnippet {
        title: "缩短文本",
        body: "请把下面内容压缩得更短，保留关键信息：",
        alias: "shorten",
        notes: "用于把长段落变成更轻的表达。",
        is_favorite: true,
        tags: &["写作", "压缩", "常用"],
    },
    StarterSnippet {
        title: "扩写成段落",
        body: "请把下面要点扩写成一段完整、自然的文字：",
        alias: "expand",
        notes: "适合从提纲生成正文。",
        is_favorite: false,
        tags: &["写作", "扩写"],
    },
    StarterSnippet {
        title: "语气更专业",
        body: "请用更专业但不生硬的语气改写：",
        alias: "tone",
        notes: "适合工作沟通和客户回复。",
        is_favorite: false,
        tags: &["写作", "语气"],
    },
    StarterSnippet {
        title: "总结要点",
        body: "请总结下面内容的关键要点，用简短 bullet 输出：",
        alias: "sum",
        notes: "适合文章、聊天记录和文档摘要。",
        is_favorite: true,
        tags: &["总结", "阅读", "常用"],
    },
    StarterSnippet {
        title: "提取行动项",
        body: "请提取行动项，按“事项 / 负责人 / 截止时间 / 状态”输出：",
        alias: "todo",
        notes: "适合会议纪要和项目同步。",
        is_favorite: true,
        tags: &["会议", "行动项", "总结"],
    },
    StarterSnippet {
        title: "提取问题风险",
        body: "请指出下面内容里的问题、风险和需要确认的点：",
        alias: "risk",
        notes: "适合方案、PRD 和客户反馈。",
        is_favorite: false,
        tags: &["风险", "分析", "产品"],
    },
    StarterSnippet {
        title: "解释概念",
        body: "请用通俗语言解释下面内容，并给一个例子：",
        alias: "explain",
        notes: "适合学习、阅读和技术概念。",
        is_favorite: false,
        tags: &["学习", "解释"],
    },
    StarterSnippet {
        title: "给 3 个备选说法",
        body: "请给出 3 个不同表达方式，分别偏简洁、专业、友好：",
        alias: "alt",
        notes: "适合命名、文案和回复措辞。",
        is_favorite: false,
        tags: &["写作", "备选"],
    },
    StarterSnippet {
        title: "检查错别字语病",
        body: "请检查错别字、语病和不自然表达，并给出修改版：",
        alias: "proof",
        notes: "适合最终发送前检查。",
        is_favorite: false,
        tags: &["写作", "校对"],
    },
    StarterSnippet {
        title: "翻译成自然中文",
        body: "请翻译成自然中文，保留原有语气和格式：",
        alias: "zh",
        notes: "适合英文内容转中文。",
        is_favorite: true,
        tags: &["翻译", "中文", "常用"],
    },
    StarterSnippet {
        title: "翻译成自然英文",
        body: "Please translate this into natural English while preserving tone and formatting:",
        alias: "en",
        notes: "Useful for turning Chinese drafts into natural English.",
        is_favorite: false,
        tags: &["翻译", "英文"],
    },
    StarterSnippet {
        title: "代码审查",
        body: "请审查下面代码，优先指出 bug、边界情况和缺少的测试：",
        alias: "review",
        notes: "适合提交前自查。",
        is_favorite: true,
        tags: &["代码", "审查", "测试"],
    },
    StarterSnippet {
        title: "解释代码",
        body: "请解释这段代码在做什么，重点说明输入、输出和关键逻辑：",
        alias: "code",
        notes: "适合读陌生代码。",
        is_favorite: false,
        tags: &["代码", "解释"],
    },
    StarterSnippet {
        title: "生成测试用例",
        body: "请为下面函数/模块生成覆盖主要分支和边界情况的测试用例：",
        alias: "test",
        notes: "适合补测试。",
        is_favorite: false,
        tags: &["代码", "测试"],
    },
    StarterSnippet {
        title: "调试分析",
        body: "请根据下面现象分析可能原因，并给出最小排查步骤：",
        alias: "debug",
        notes: "适合错误日志和异常现象。",
        is_favorite: false,
        tags: &["代码", "调试"],
    },
    StarterSnippet {
        title: "写简短回复",
        body: "请帮我写一段简短、礼貌、可直接发送的回复：",
        alias: "reply",
        notes: "适合邮件、IM 和客户沟通。",
        is_favorite: true,
        tags: &["回复", "工作", "常用"],
    },
    StarterSnippet {
        title: "转成清单",
        body: "请把下面内容整理成清晰的清单：",
        alias: "list",
        notes: "适合整理杂乱信息。",
        is_favorite: false,
        tags: &["整理", "清单"],
    },
];

pub fn open_database(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let conn = Connection::open(path).map_err(|error| error.to_string())?;
    init_schema(&conn)?;
    seed_starter_snippets_if_needed(&conn, None)?;
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

pub fn restore_starter_snippets(conn: &Connection, locale: Option<&str>) -> Result<usize, String> {
    let locale = normalize_starter_snippet_locale(locale);
    let snippets = starter_snippets_for_locale(locale);
    let mut restored = 0;

    for snippet in snippets {
        if starter_snippet_exists(conn, snippet)? {
            continue;
        }

        insert_starter_snippet(conn, snippet)?;
        restored += 1;
    }

    set_setting(
        conn,
        STARTER_SNIPPET_SETTING_KEY.to_string(),
        format!("{}:{}", STARTER_SNIPPET_VERSION, locale.as_str()),
    )?;
    Ok(restored)
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

fn seed_starter_snippets_if_needed(conn: &Connection, locale: Option<&str>) -> Result<(), String> {
    if prompt_count(conn)? == 0 && get_setting(conn, STARTER_SNIPPET_SETTING_KEY)?.is_none() {
        restore_starter_snippets(conn, locale)?;
    }

    Ok(())
}

fn starter_snippets_for_locale(locale: StarterSnippetLocale) -> &'static [StarterSnippet] {
    match locale {
        StarterSnippetLocale::En => EN_STARTER_SNIPPETS,
        StarterSnippetLocale::ZhCn => ZH_CN_STARTER_SNIPPETS,
    }
}

fn normalize_starter_snippet_locale(locale: Option<&str>) -> StarterSnippetLocale {
    let locale = locale
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(system_locale)
        .unwrap_or_else(|| "en".to_string());

    if locale.to_ascii_lowercase().starts_with("zh") {
        StarterSnippetLocale::ZhCn
    } else {
        StarterSnippetLocale::En
    }
}

#[cfg(target_os = "windows")]
fn system_locale() -> Option<String> {
    windows_user_locale().or_else(env_locale)
}

#[cfg(target_os = "windows")]
fn windows_user_locale() -> Option<String> {
    const LOCALE_NAME_MAX_LENGTH: usize = 85;
    let mut buffer = [0u16; LOCALE_NAME_MAX_LENGTH];
    let length =
        unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH as i32) };

    if length <= 1 {
        return None;
    }

    Some(String::from_utf16_lossy(&buffer[..(length as usize - 1)]))
}

#[cfg(target_os = "windows")]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn GetUserDefaultLocaleName(locale_name: *mut u16, locale_name_len: i32) -> i32;
}

#[cfg(target_os = "macos")]
fn system_locale() -> Option<String> {
    macos_user_locale().or_else(env_locale)
}

#[cfg(target_os = "macos")]
fn macos_user_locale() -> Option<String> {
    let output = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLocale"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let locale = String::from_utf8(output.stdout).ok()?;
    let locale = locale.trim().replace('_', "-");
    if locale.is_empty() {
        None
    } else {
        Some(locale)
    }
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn system_locale() -> Option<String> {
    env_locale()
}

fn env_locale() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|key| std::env::var(key).ok())
        .map(|value| value.split('.').next().unwrap_or(&value).replace('_', "-"))
}

fn prompt_count(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))
        .map_err(|error| error.to_string())
}

fn starter_snippet_exists(conn: &Connection, snippet: &StarterSnippet) -> Result<bool, String> {
    conn.query_row(
        "
        SELECT 1
        FROM prompts
        WHERE lower(coalesce(alias, '')) = lower(?1)
           OR lower(title) = lower(?2)
        LIMIT 1
        ",
        params![snippet.alias, snippet.title],
        |_| Ok(()),
    )
    .optional()
    .map(|row| row.is_some())
    .map_err(|error| error.to_string())
}

fn insert_starter_snippet(conn: &Connection, snippet: &StarterSnippet) -> Result<Prompt, String> {
    create_prompt(
        conn,
        PromptInput {
            title: snippet.title.to_string(),
            body: snippet.body.to_string(),
            alias: Some(snippet.alias.to_string()),
            notes: Some(snippet.notes.to_string()),
            is_favorite: snippet.is_favorite,
            tags: snippet.tags.iter().map(|tag| (*tag).to_string()).collect(),
        },
    )
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

    #[test]
    fn starter_snippets_seed_only_once_for_empty_database() {
        let conn = conn();

        assert_eq!(prompt_count(&conn).expect("prompt count works"), 0);
        seed_starter_snippets_if_needed(&conn, Some("en-US")).expect("starter snippets seed");

        let prompts = list_prompts(&conn).expect("seeded prompts list");
        assert_eq!(prompts.len(), EN_STARTER_SNIPPETS.len());
        assert!(prompts.iter().any(|prompt| {
            prompt.alias.as_deref() == Some("clarify") && prompt.body == EN_STARTER_SNIPPETS[0].body
        }));
        let expected_tag_count = EN_STARTER_SNIPPETS
            .iter()
            .flat_map(|snippet| snippet.tags.iter())
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(
            list_tags(&conn).expect("seeded tags list").len(),
            expected_tag_count
        );

        seed_starter_snippets_if_needed(&conn, Some("zh-CN"))
            .expect("starter snippets do not reseed");
        assert_eq!(
            list_prompts(&conn)
                .expect("prompts list after second seed")
                .len(),
            EN_STARTER_SNIPPETS.len()
        );
        assert_eq!(
            restore_starter_snippets(&conn, Some("en-US"))
                .expect("restore skips existing snippets"),
            0
        );
    }

    #[test]
    fn starter_snippet_locale_routes_chinese_and_defaults_to_english() {
        assert_eq!(
            normalize_starter_snippet_locale(Some("zh-Hans-CN")),
            StarterSnippetLocale::ZhCn
        );
        assert_eq!(
            normalize_starter_snippet_locale(Some("en-US")),
            StarterSnippetLocale::En
        );
        assert_eq!(
            starter_snippets_for_locale(StarterSnippetLocale::ZhCn)[0].body,
            ZH_CN_STARTER_SNIPPETS[0].body
        );
    }
}
