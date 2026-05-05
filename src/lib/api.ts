import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppEnvironment,
  AppSettings,
  LocaleCode,
  Prompt,
  PromptInput,
  PromptTag,
  PromptUpdateInput,
  SavedFile,
  SettingRecord
} from "./types";
import { defaultShortcutBindings, migrateShortcutBindingsForPlatform } from "./platform";
import { STARTER_SNIPPET_COUNT, starterSnippetInputsForLocale } from "./starter-snippets";

const PROMPTS_KEY = "promptpop.mock.prompts";
const TAGS_KEY = "promptpop.mock.tags";
const SETTINGS_PREFIX = "promptpop.setting.";
const EXPORT_PREFIX = "promptpop-export";

const isTauri = () => typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);

const now = () => new Date().toISOString();
const id = () => crypto.randomUUID();

function systemStarterSnippetLocale(): string {
  if (typeof navigator === "undefined") return "en";
  return navigator.languages?.find(Boolean) ?? navigator.language ?? "en";
}

const starterSnippetInputs = starterSnippetInputsForLocale(systemStarterSnippetLocale());
const seedTagNames = [...new Set(starterSnippetInputs.flatMap((prompt) => prompt.tags))];
const seedTags: PromptTag[] = seedTagNames.map((name) => ({
  id: id(),
  name,
  color: null,
  createdAt: now()
}));

const seedPrompts: Prompt[] = starterSnippetInputs.map((input) => ({
  id: id(),
  title: input.title,
  body: input.body,
  alias: input.alias,
  notes: input.notes,
  isFavorite: input.isFavorite,
  usageCount: 0,
  lastUsedAt: null,
  createdAt: now(),
  updatedAt: now(),
  tags: seedTags.filter((tag) => input.tags.some((name) => name.toLowerCase() === tag.name.toLowerCase()))
}));

export const defaultSettings: AppSettings = {
  locale: "en",
  theme: "system",
  defaultView: "launcher",
  density: "compact",
  launchAtLogin: false,
  autoPasteAfterSelection: false,
  localBackup: true,
  showPreviewPane: true,
  showUsageCount: true,
  showTagsInLauncher: true,
  developerMode: false,
  shortcuts: defaultShortcutBindings()
};

const settingKeys = [
  "locale",
  "theme",
  "defaultView",
  "density",
  "launchAtLogin",
  "autoPasteAfterSelection",
  "localBackup",
  "showPreviewPane",
  "showUsageCount",
  "showTagsInLauncher",
  "developerMode",
  "shortcuts.globalLauncher",
  "shortcuts.focusSearch",
  "shortcuts.copySelected",
  "shortcuts.pasteSelected",
  "shortcuts.editSelected",
  "shortcuts.jumpResult"
] as const;

function readStore<T>(key: string, fallback: T): T {
  const raw = localStorage.getItem(key);
  if (!raw) {
    localStorage.setItem(key, JSON.stringify(fallback));
    return fallback;
  }
  return JSON.parse(raw) as T;
}

function writeStore<T>(key: string, value: T): T {
  localStorage.setItem(key, JSON.stringify(value));
  return value;
}

function readSettingStore(key: string): string | null {
  return localStorage.getItem(`${SETTINGS_PREFIX}${key}`);
}

function writeSettingStore(key: string, value: string): SettingRecord {
  const record: SettingRecord = { key, value, updatedAt: now() };
  localStorage.setItem(`${SETTINGS_PREFIX}${key}`, value);
  return record;
}

function readBoolean(value: string | null, fallback: boolean): boolean {
  if (value === "true") return true;
  if (value === "false") return false;
  return fallback;
}

function readChoice<T extends string>(value: string | null, allowed: readonly T[], fallback: T): T {
  return allowed.includes(value as T) ? (value as T) : fallback;
}

function withSetting(settings: AppSettings, key: string, value: string | null): AppSettings {
  if (value === null) return settings;

  switch (key) {
    case "locale":
      return { ...settings, locale: readChoice<LocaleCode>(value, ["en", "zh-CN"], settings.locale) };
    case "theme":
      return { ...settings, theme: readChoice(value, ["system", "light", "dark"], settings.theme) };
    case "defaultView":
      return { ...settings, defaultView: readChoice(value, ["launcher", "library"], settings.defaultView) };
    case "density":
      return { ...settings, density: readChoice(value, ["compact", "comfortable"], settings.density) };
    case "launchAtLogin":
      return { ...settings, launchAtLogin: readBoolean(value, settings.launchAtLogin) };
    case "autoPasteAfterSelection":
      return {
        ...settings,
        autoPasteAfterSelection: readBoolean(value, settings.autoPasteAfterSelection)
      };
    case "localBackup":
      return { ...settings, localBackup: readBoolean(value, settings.localBackup) };
    case "showPreviewPane":
      return { ...settings, showPreviewPane: readBoolean(value, settings.showPreviewPane) };
    case "showUsageCount":
      return { ...settings, showUsageCount: readBoolean(value, settings.showUsageCount) };
    case "showTagsInLauncher":
      return { ...settings, showTagsInLauncher: readBoolean(value, settings.showTagsInLauncher) };
    case "developerMode":
      return { ...settings, developerMode: readBoolean(value, settings.developerMode) };
    default:
      if (key.startsWith("shortcuts.")) {
        const shortcutId = key.replace("shortcuts.", "") as keyof AppSettings["shortcuts"];
        if (shortcutId in settings.shortcuts && value.trim()) {
          return {
            ...settings,
            shortcuts: { ...settings.shortcuts, [shortcutId]: value }
          };
        }
      }
      return settings;
  }
}

function settingValue(settings: AppSettings, key: string): string {
  if (key.startsWith("shortcuts.")) {
    const shortcutId = key.replace("shortcuts.", "") as keyof AppSettings["shortcuts"];
    return settings.shortcuts[shortcutId];
  }

  const value = settings[key as keyof Omit<AppSettings, "shortcuts">];
  return typeof value === "boolean" ? String(value) : value;
}

function normalizeTagNames(names: string[]): string[] {
  return [...new Set(names.map((tag) => tag.trim()).filter(Boolean))];
}

function getMockTags(): PromptTag[] {
  return readStore(TAGS_KEY, seedTags);
}

function getMockPrompts(): Prompt[] {
  return readStore(PROMPTS_KEY, seedPrompts);
}

function tagObjects(names: string[]): PromptTag[] {
  const existing = getMockTags();
  const allTags = [...existing];

  for (const name of normalizeTagNames(names)) {
    if (!allTags.some((tag) => tag.name.toLowerCase() === name.toLowerCase())) {
      allTags.push({ id: id(), name, color: null, createdAt: now() });
    }
  }

  writeStore(TAGS_KEY, allTags);
  return allTags.filter((tag) =>
    names.some((name) => name.toLowerCase() === tag.name.toLowerCase())
  );
}

function matchesStarterSnippet(existing: Prompt, input: PromptInput): boolean {
  const existingAlias = existing.alias?.trim().toLowerCase();
  const inputAlias = input.alias?.trim().toLowerCase();
  return (
    Boolean(inputAlias && existingAlias === inputAlias) ||
    existing.title.trim().toLowerCase() === input.title.trim().toLowerCase()
  );
}

function createMockPrompt(input: PromptInput): Prompt {
  return {
    id: id(),
    title: input.title,
    body: input.body,
    alias: input.alias,
    notes: input.notes,
    isFavorite: input.isFavorite,
    usageCount: 0,
    lastUsedAt: null,
    createdAt: now(),
    updatedAt: now(),
    tags: tagObjects(input.tags)
  };
}

async function writeClipboardText(text: string): Promise<void> {
  try {
    await navigator.clipboard?.writeText(text);
    return;
  } catch {
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.setAttribute("readonly", "true");
    textarea.style.position = "fixed";
    textarea.style.left = "-9999px";
    textarea.style.top = "0";
    document.body.appendChild(textarea);
    textarea.select();

    try {
      const copied = document.execCommand("copy");
      if (!copied) throw new Error("Clipboard write permission denied");
    } finally {
      document.body.removeChild(textarea);
    }
  }
}

export async function listPrompts(): Promise<Prompt[]> {
  if (isTauri()) return invoke<Prompt[]>("list_prompts");
  return getMockPrompts();
}

export async function restoreStarterSnippets(locale: LocaleCode): Promise<number> {
  if (isTauri()) return invoke<number>("restore_starter_snippets", { locale });

  const existing = getMockPrompts();
  const restored = starterSnippetInputsForLocale(locale)
    .filter((input) => !existing.some((prompt) => matchesStarterSnippet(prompt, input)))
    .map(createMockPrompt);

  if (restored.length > 0) {
    writeStore(PROMPTS_KEY, [...restored, ...existing]);
  }

  return restored.length;
}

export { STARTER_SNIPPET_COUNT };

export async function createPrompt(input: PromptInput): Promise<Prompt> {
  if (isTauri()) return invoke<Prompt>("create_prompt", { input });
  const prompt = createMockPrompt(input);
  writeStore(PROMPTS_KEY, [prompt, ...getMockPrompts()]);
  return prompt;
}

export async function updatePrompt(input: PromptUpdateInput): Promise<Prompt> {
  if (isTauri()) return invoke<Prompt>("update_prompt", { input });
  let updated: Prompt | null = null;
  const prompts = getMockPrompts().map((prompt) => {
    if (prompt.id !== input.id) return prompt;
    updated = {
      ...prompt,
      title: input.title,
      body: input.body,
      alias: input.alias,
      notes: input.notes,
      isFavorite: input.isFavorite,
      updatedAt: now(),
      tags: tagObjects(input.tags)
    };
    return updated;
  });
  writeStore(PROMPTS_KEY, prompts);
  if (!updated) throw new Error("Prompt not found");
  return updated;
}

export async function deletePrompt(id: string): Promise<void> {
  if (isTauri()) return invoke("delete_prompt", { id });
  writeStore(
    PROMPTS_KEY,
    getMockPrompts().filter((prompt) => prompt.id !== id)
  );
}

export async function usePrompt(id: string): Promise<Prompt> {
  if (isTauri()) return invoke<Prompt>("use_prompt", { id });
  let used: Prompt | null = null;
  const prompts = getMockPrompts().map((prompt) => {
    if (prompt.id !== id) return prompt;
    used = {
      ...prompt,
      usageCount: prompt.usageCount + 1,
      lastUsedAt: now(),
      updatedAt: now()
    };
    return used;
  });
  writeStore(PROMPTS_KEY, prompts);
  if (!used) throw new Error("Prompt not found");
  return used;
}

export async function copyPrompt(id: string): Promise<Prompt> {
  if (isTauri()) {
    return invoke<Prompt>("copy_prompt", { id });
  } else {
    const prompt = await usePrompt(id);
    await writeClipboardText(prompt.body);
    return prompt;
  }
}

export async function pastePrompt(id: string): Promise<Prompt> {
  if (isTauri()) {
    return invoke<Prompt>("paste_prompt", { id });
  } else {
    const prompt = await usePrompt(id);
    await writeClipboardText(prompt.body);
    return prompt;
  }
}

export async function listTags(): Promise<PromptTag[]> {
  if (isTauri()) return invoke<PromptTag[]>("list_tags");
  return getMockTags();
}

export async function getSetting(key: string): Promise<string | null> {
  if (isTauri()) {
    const record = await invoke<SettingRecord | null>("get_setting", { key });
    return record?.value ?? null;
  }
  return readSettingStore(key);
}

export async function setSetting(key: string, value: string): Promise<SettingRecord> {
  if (isTauri()) return invoke<SettingRecord>("set_setting", { key, value });
  return writeSettingStore(key, value);
}

export async function loadSettings(seed: Partial<AppSettings> = {}): Promise<AppSettings> {
  let settings: AppSettings = {
    ...defaultSettings,
    ...seed,
    shortcuts: { ...defaultSettings.shortcuts, ...seed.shortcuts }
  };

  const values = await Promise.all(settingKeys.map(async (key) => [key, await getSetting(key)] as const));
  for (const [key, value] of values) {
    settings = withSetting(settings, key, value);
  }

  return {
    ...settings,
    shortcuts: migrateShortcutBindingsForPlatform(settings.shortcuts)
  };
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  await Promise.all(settingKeys.map((key) => setSetting(key, settingValue(settings, key))));
}

export async function getAppEnvironment(): Promise<AppEnvironment> {
  if (isTauri()) return invoke<AppEnvironment>("app_environment");

  return {
    appVersion: "0.1.0",
    tauriVersion: "browser",
    databasePath: "localStorage://promptpop.mock.prompts",
    dataDir: "localStorage://PromptPop",
    logsDir: "browser console",
    exportsDir: "Downloads",
    backupsDir: "localStorage",
    launchAtLogin: readBoolean(readSettingStore("launchAtLogin"), defaultSettings.launchAtLogin),
    accessibilityTrusted: null
  };
}

export async function registerLauncherShortcut(shortcut: string): Promise<string> {
  if (isTauri()) return invoke<string>("register_launcher_shortcut", { shortcut });
  return shortcut;
}

export async function setLaunchAtLogin(enabled: boolean): Promise<boolean> {
  if (isTauri()) return invoke<boolean>("set_launch_at_login", { enabled });
  writeSettingStore("launchAtLogin", String(enabled));
  return enabled;
}

export async function hideLauncher(): Promise<void> {
  if (isTauri()) await invoke("hide_launcher");
}

export async function configureWindowMode(mode: "launcher" | "peek" | "workspace"): Promise<void> {
  if (isTauri()) await invoke("configure_window_mode", { mode });
}

export async function openSettingsTarget(
  target: "data" | "logs" | "exports" | "backups" | "accessibility"
): Promise<void> {
  if (isTauri()) {
    await invoke("open_settings_target", { target });
  }
}

export async function testPasteAutomation(): Promise<void> {
  if (isTauri()) {
    await invoke("test_paste");
    return;
  }
  await writeClipboardText("PromptPop paste test");
}

export async function saveExport(format: "json" | "markdown", contents: string): Promise<SavedFile> {
  if (isTauri()) return invoke<SavedFile>("save_export", { format, contents });

  const extension = format === "json" ? "json" : "md";
  const filename = `${EXPORT_PREFIX}-${new Date().toISOString().replace(/[:.]/g, "-")}.${extension}`;
  const blob = new Blob([contents], {
    type: format === "json" ? "application/json" : "text/markdown"
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
  return { path: filename, bytes: new TextEncoder().encode(contents).byteLength };
}

export async function backupDatabase(): Promise<SavedFile> {
  if (isTauri()) return invoke<SavedFile>("backup_database");
  return {
    path: "localStorage://PromptPop",
    bytes: new TextEncoder().encode(JSON.stringify(getMockPrompts())).byteLength
  };
}

export async function onLauncherShortcut(callback: () => void): Promise<() => void> {
  if (!isTauri()) return () => undefined;
  return listen("promptpop:launcher-shortcut", callback);
}

export async function onOpenSettingsRequest(callback: () => void): Promise<() => void> {
  if (!isTauri()) return () => undefined;
  return listen("promptpop:open-settings", callback);
}
