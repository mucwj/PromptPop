# PromptPop Design Plan

## 1. Product Positioning

PromptPop is a lightweight personal AI prompt launcher for desktop users.
It is designed for people who repeatedly use prompts across ChatGPT, Claude,
IDEs, browsers, note apps, email clients, and other writing or coding tools.

The product should feel closer to Raycast, Alfred, or a clipboard manager than
to a document library. The core value is reducing friction: press a global
shortcut, find a favorite prompt, and insert it into the current workflow.

Primary positioning:

> A local-first desktop prompt launcher that keeps your best AI prompts one
> shortcut away.

Chinese positioning:

> 一个本地优先的个人 AI 提示词快捷调用工具，让常用提示词随时可呼出、可搜索、可插入。

## 2. Design Principles

- Fast before feature-rich: opening, searching, and selecting prompts must feel
  instant.
- Local-first: prompts may contain private workflows, client context, or
  internal process knowledge, so local storage is the default.
- Keyboard-first: every common action should be possible without touching the
  mouse.
- Beautiful but quiet: the UI should feel polished and native, not decorative or
  heavy.
- Small surface area: the main launcher should do one thing well; deeper
  management can live in a separate window.
- Cross-platform-ready: macOS is the first target, but the architecture should
  not block Windows or Linux support.

## 3. Recommended Technology Stack

### Desktop Runtime

Use Tauri 2.

Reasons:

- Does not require Swift.
- Uses the operating system WebView instead of bundling Chromium.
- Lower app size and memory footprint than Electron for this category of app.
- Good support for system-level desktop features through Rust and Tauri plugins.
- Cross-platform path for macOS, Windows, and Linux.

### Frontend

Use Svelte with TypeScript.

Reasons:

- Small runtime and simple state model.
- Good fit for a focused productivity tool.
- Easier to keep fast than a large React stack.
- Works well with Vite and Tauri.

Styling:

- CSS variables for theme tokens.
- Tailwind CSS can be used if the team prefers utility classes.
- Avoid heavy component libraries in the first version.
- If a component library is needed later, prefer shadcn-svelte-style copied
  components over a large runtime dependency.

### Backend

Use Rust inside Tauri for system integration and persistence boundaries.

Rust responsibilities:

- Register global shortcuts.
- Show, hide, focus, and position launcher windows.
- Manage system tray or menu bar behavior.
- Read and write SQLite data.
- Handle clipboard operations.
- Optionally trigger automatic paste with explicit user permission.
- Provide import/export APIs.

### Storage

Use SQLite.

Initial search can use ordinary indexed queries. Add SQLite FTS5 once prompt
content search becomes important.

Suggested crates:

- `rusqlite` or `sqlx` for SQLite access.
- `serde` for request/response data structures.
- Tauri plugins for global shortcuts, clipboard, dialog, opener, and shell only
  where needed.

## 4. Core User Experience

### Main Flow

1. User presses a global shortcut, for example `Option + Space`.
2. PromptPop opens a compact floating launcher.
3. Search input is focused immediately.
4. User types a keyword, alias, tag, or prompt title.
5. Results update instantly.
6. User presses `Enter` to copy the selected prompt.
7. Optional: user presses `Command + Enter` to paste into the current app.
8. Launcher closes and the user returns to their previous workflow.

### Launcher Behavior

The launcher should be small, fast, and keyboard-first.

Expected controls:

- `Option + Space`: toggle launcher.
- `Escape`: close launcher.
- `Enter`: copy selected prompt.
- `Command + Enter`: paste selected prompt, if enabled.
- `Arrow Up / Down`: move selection.
- `Command + 1...9`: select pinned/favorite prompts.
- `Command + K`: focus search, if focus moves.
- `Command + E`: edit selected prompt.
- `Command + N`: create new prompt.

### Main Windows

PromptPop should have two main surfaces:

- Launcher: compact, transient, keyboard-first.
- Library: full management window for creating, editing, tagging, and organizing
  prompts.

The launcher should not become a full management UI. Keep it fast and calm.

## 5. MVP Scope

Version 0.1 should include:

- macOS desktop app.
- Menu bar or tray resident mode.
- Global shortcut to open launcher.
- Prompt CRUD.
- Prompt title, body, tags, favorite flag, and alias.
- Search by title, alias, tag, and body.
- Favorites section.
- Recent prompts section.
- Copy selected prompt to clipboard.
- Optional automatic paste setting.
- Local SQLite database.
- JSON or Markdown export.
- Basic light/dark theme.

Out of scope for MVP:

- Cloud sync.
- Team sharing.
- Prompt marketplace.
- Browser extension.
- AI-assisted prompt generation.
- Complex version history.
- Plugin system.

## 6. Data Model

Initial tables:

```sql
CREATE TABLE prompts (
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

CREATE TABLE tags (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE prompt_tags (
  prompt_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  PRIMARY KEY (prompt_id, tag_id),
  FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

Later additions:

- `prompt_versions` for version history.
- `prompt_variables` for structured template variables.
- `collections` for folders or smart groups.
- FTS virtual table for full-text search.

## 7. Prompt Template Variables

PromptPop should eventually support template variables:

```text
请帮我为 {{产品名称}} 写一段面向 {{目标用户}} 的产品介绍，语气要 {{语气}}。
```

When the user selects this prompt, PromptPop can show a small variable form
before copying or pasting the final generated text.

This should not be part of the first build unless the MVP is already stable.

## 8. UI Direction

Visual tone:

- Clean, focused, modern desktop utility.
- Compact layout with high information density.
- Soft contrast, clear focus states, restrained motion.
- Avoid marketing-style hero screens inside the app.

Launcher layout:

- Top search input.
- Left or top filter chips for Favorites, Recent, All, Tags.
- Result list with title, short preview, tags, and shortcut hint.
- Right-side preview only if it does not slow down the launcher.

Library layout:

- Sidebar for collections/tags.
- Main list for prompts.
- Editor panel for selected prompt.
- Metadata controls for tags, alias, favorite, and notes.

## 9. Architecture

High-level modules:

```text
PromptPop
├── src/                  # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   ├── stores/
│   │   ├── api/
│   │   └── search/
│   └── routes or views/
├── src-tauri/            # Rust desktop backend
│   ├── src/
│   │   ├── commands/
│   │   ├── database/
│   │   ├── shortcuts/
│   │   ├── clipboard/
│   │   └── windows/
│   └── tauri.conf.json
└── docs/
    └── design.md
```

Frontend talks to Rust through Tauri commands. Rust owns persistence and system
operations. The frontend should not directly manage platform-specific behavior.

## 10. Security and Privacy

Default posture:

- Store data locally.
- Do not send prompts to external services.
- Make cloud sync an explicit future feature, not an assumed behavior.
- Avoid collecting analytics in the first version.
- For automatic paste, clearly request accessibility permission on macOS and
  explain why it is needed.

Important macOS permissions:

- Accessibility permission may be required for automatic paste.
- Global shortcut registration may fail when another app already owns the same
  shortcut.
- Clipboard access should be transparent and predictable.

## 11. Roadmap

### Phase 1: MVP

- Tauri + Svelte app shell.
- SQLite persistence.
- Prompt CRUD.
- Launcher window.
- Global shortcut.
- Copy selected prompt.
- Favorites and recent prompts.

### Phase 2: Workflow Polish

- Automatic paste.
- Template variables.
- Import/export.
- Fuzzy search improvements.
- Theme customization.
- Keyboard shortcut settings.

### Phase 3: Power User Features

- Version history.
- Collections and smart filters.
- Prompt snippets with aliases.
- Markdown preview.
- Quick capture from clipboard.
- Local backup.

### Phase 4: Cross-platform and Sync

- Windows build.
- Linux build.
- Optional encrypted sync.
- Optional browser extension.
- Optional mobile companion or web vault.

## 12. Open Decisions

- Whether to use plain Svelte or SvelteKit.
- Whether the first build should include automatic paste or only clipboard copy.
- Whether prompts should be edited in Markdown-only mode or plain text first.
- Whether collections are needed in MVP or tags are enough.
- Whether app naming in package identifiers should use `promptpop` or
  `com.promptpop.app`.

## 13. Initial Recommendation

Start with:

- Tauri 2
- Svelte + TypeScript
- SQLite
- Local-first data
- macOS-first packaging
- Launcher-first UX

The first product milestone should be:

> Press a global shortcut, search a favorite prompt, press Enter, and have the
> prompt ready in the clipboard within one second.
