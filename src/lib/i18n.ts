export type Locale = "en" | "zh-CN";

export type TranslationValues = Record<string, string | number>;

const LOCALE_KEY = "promptpop.locale";

const en = {
  alias: "Alias",
  aliasPlaceholder: "Short command, for example review",
  all: "All",
  appAriaPromptLauncher: "Prompt launcher",
  appAriaPromptLibrary: "Prompt library",
  appAriaPromptResults: "Prompt results",
  appAriaSelectedPreview: "Selected prompt preview",
  body: "Body",
  bodyPlaceholder: "Write the prompt body",
  clipboardPermissionDenied: "Clipboard write permission denied",
  close: "Close",
  copiedPrompt: 'Copied "{title}"',
  copy: "Copy",
  copyPromptFailed: "Unable to copy prompt",
  copyShortcut: "copy",
  createdPrompt: 'Created "{title}"',
  delete: "Delete",
  deletedPrompt: 'Deleted "{title}"',
  editPrompt: "Edit Prompt",
  editShortcut: "edit",
  favoritePrompt: "Favorite prompt",
  favorites: "Favorites",
  filters: "Prompt filters",
  language: "Language",
  languageChanged: "Language changed",
  keyboardFirst: "Keyboard first",
  lastUsed: "Last used",
  launcher: "Launcher",
  launcherDismissed: "Launcher dismissed",
  library: "Library",
  loadPromptsFailed: "Failed to load prompts",
  localOnly: "Local",
  loadingPrompts: "Loading prompts...",
  never: "Never",
  new: "New",
  newPrompt: "New Prompt",
  noMetadata: "No metadata",
  noPromptsFound: "No prompts found",
  noPromptsHint: "Create one in the library or change the search.",
  none: "None",
  notes: "Notes",
  notesPlaceholder: "Private notes",
  pastePromptFailed: "Unable to paste prompt",
  pasteShortcut: "paste",
  pastedPrompt: 'Pasted "{title}"',
  pinned: "Pinned",
  preview: "Preview",
  previewBody: "Body preview",
  previewClosed: "Preview closed",
  previewOpened: 'Previewing "{title}"',
  previewShortcut: "preview",
  promptLauncher: "Prompt Launcher",
  promptNotFound: "Prompt not found",
  promptsLoaded: "{count} prompts loaded",
  ready: "Ready",
  recent: "Recent",
  savePrompt: "Save Prompt",
  savePromptFailed: "Unable to save prompt",
  saving: "Saving...",
  search: "Search",
  searchPlaceholder: "Title, alias, tag, or body",
  tags: "Tags",
  tagsPlaceholder: "writing, coding, review",
  title: "Title",
  titleAndBodyRequired: "Title and body are required",
  titlePlaceholder: "Prompt title",
  updatedPrompt: 'Updated "{title}"',
  usePlural: "{count} uses",
  useSingular: "{count} use"
} as const;

export type TranslationKey = keyof typeof en;

const zhCN: Record<TranslationKey, string> = {
  alias: "别名",
  aliasPlaceholder: "短命令，例如 review",
  all: "全部",
  appAriaPromptLauncher: "提示词启动器",
  appAriaPromptLibrary: "提示词库",
  appAriaPromptResults: "提示词结果",
  appAriaSelectedPreview: "已选提示词预览",
  body: "正文",
  bodyPlaceholder: "编写提示词正文",
  clipboardPermissionDenied: "剪贴板写入权限被拒绝",
  close: "关闭",
  copiedPrompt: "已复制「{title}」",
  copy: "复制",
  copyPromptFailed: "无法复制提示词",
  copyShortcut: "复制",
  createdPrompt: "已创建「{title}」",
  delete: "删除",
  deletedPrompt: "已删除「{title}」",
  editPrompt: "编辑提示词",
  editShortcut: "编辑",
  favoritePrompt: "收藏提示词",
  favorites: "收藏",
  filters: "提示词筛选",
  language: "语言",
  languageChanged: "语言已切换",
  keyboardFirst: "键盘优先",
  lastUsed: "上次使用",
  launcher: "启动器",
  launcherDismissed: "启动器已收起",
  library: "词库",
  loadPromptsFailed: "加载提示词失败",
  localOnly: "本地",
  loadingPrompts: "正在加载提示词...",
  never: "从未",
  new: "新建",
  newPrompt: "新建提示词",
  noMetadata: "无元数据",
  noPromptsFound: "没有找到提示词",
  noPromptsHint: "可以在词库中新建，或调整搜索条件。",
  none: "无",
  notes: "备注",
  notesPlaceholder: "私人备注",
  pastePromptFailed: "无法粘贴提示词",
  pasteShortcut: "粘贴",
  pastedPrompt: "已粘贴「{title}」",
  pinned: "置顶",
  preview: "预览",
  previewBody: "正文预览",
  previewClosed: "预览已关闭",
  previewOpened: "正在预览「{title}」",
  previewShortcut: "预览",
  promptLauncher: "提示词启动器",
  promptNotFound: "未找到提示词",
  promptsLoaded: "已加载 {count} 条提示词",
  ready: "就绪",
  recent: "最近",
  savePrompt: "保存提示词",
  savePromptFailed: "无法保存提示词",
  saving: "正在保存...",
  search: "搜索",
  searchPlaceholder: "标题、别名、标签或正文",
  tags: "标签",
  tagsPlaceholder: "写作, 编程, 评审",
  title: "标题",
  titleAndBodyRequired: "标题和正文为必填项",
  titlePlaceholder: "提示词标题",
  updatedPrompt: "已更新「{title}」",
  usePlural: "使用 {count} 次",
  useSingular: "使用 {count} 次"
};

const messages: Record<Locale, Record<TranslationKey, string>> = {
  en,
  "zh-CN": zhCN
};

export const locales: { code: Locale; label: string }[] = [
  { code: "en", label: "EN" },
  { code: "zh-CN", label: "中文" }
];

export function detectLocale(): Locale {
  const saved = readSavedLocale();
  if (saved) return saved;

  const preferred = typeof navigator === "undefined" ? [] : navigator.languages ?? [navigator.language];
  return preferred.some((language) => language.toLowerCase().startsWith("zh")) ? "zh-CN" : "en";
}

export function getMessages(locale: Locale): Record<TranslationKey, string> {
  return messages[locale];
}

export function saveLocale(locale: Locale): void {
  localStorage.setItem(LOCALE_KEY, locale);
}

export function t(locale: Locale, key: TranslationKey, values: TranslationValues = {}): string {
  return messages[locale][key].replace(/\{(\w+)\}/g, (_, name: string) => String(values[name] ?? ""));
}

function readSavedLocale(): Locale | null {
  if (typeof localStorage === "undefined") return null;
  const saved = localStorage.getItem(LOCALE_KEY);
  return saved === "en" || saved === "zh-CN" ? saved : null;
}
