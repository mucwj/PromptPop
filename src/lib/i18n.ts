export type Locale = "en" | "zh-CN";

export type TranslationValues = Record<string, string | number>;

const LOCALE_KEY = "promptpop.locale";

const en = {
  alias: "Alias",
  aliasPlaceholder: "Short command, for example review",
  all: "All",
  appAriaPromptLauncher: "Snippet launcher",
  appAriaPromptLibrary: "Snippet library",
  appAriaPromptResults: "Snippet results",
  appAriaSelectedPreview: "Selected snippet preview",
  body: "Snippet",
  bodyPlaceholder: "Write a short reusable snippet",
  clipboardPermissionDenied: "Clipboard write permission denied",
  close: "Close",
  copiedPrompt: 'Copied "{title}"',
  copy: "Copy",
  copyPromptFailed: "Unable to copy snippet",
  copyShortcut: "copy",
  createdPrompt: 'Created "{title}"',
  delete: "Delete",
  deletedPrompt: 'Deleted "{title}"',
  editPrompt: "Edit Snippet",
  editShortcut: "edit",
  favoritePrompt: "Favorite snippet",
  favorites: "Favorites",
  filters: "Snippet filters",
  language: "Language",
  languageChanged: "Language changed",
  keyboardFirst: "Keyboard first",
  lastUsed: "Last used",
  launcher: "Launcher",
  launcherDismissed: "Launcher dismissed",
  library: "Snippets",
  loadPromptsFailed: "Failed to load snippets",
  localOnly: "Local",
  loadingPrompts: "Loading snippets...",
  never: "Never",
  new: "New",
  newPrompt: "New Snippet",
  noMetadata: "No metadata",
  noPromptsFound: "No snippets found",
  noPromptsHint: "Create a snippet in the snippet library or change the search.",
  none: "None",
  notes: "Notes",
  notesPlaceholder: "Private notes",
  pastePromptFailed: "Unable to paste snippet",
  pasteShortcut: "paste",
  pastedPrompt: 'Pasted "{title}"',
  pinned: "Pinned",
  preview: "Preview",
  previewBody: "Snippet preview",
  previewClosed: "Preview closed",
  previewOpened: 'Previewing "{title}"',
  previewShortcut: "preview",
  promptLauncher: "Snippet Launcher",
  promptNotFound: "Snippet not found",
  promptsLoaded: "{count} snippets loaded",
  ready: "Ready",
  recent: "Recent",
  savePrompt: "Save Snippet",
  savePromptFailed: "Unable to save snippet",
  saving: "Saving...",
  search: "Search",
  searchPlaceholder: "Title, alias, tag, or snippet body",
  tags: "Tags",
  tagsPlaceholder: "writing, coding, review",
  title: "Title",
  titleAndBodyRequired: "Title and snippet are required",
  titlePlaceholder: "Snippet title",
  updatedPrompt: 'Updated "{title}"',
  usePlural: "{count} uses",
  useSingular: "{count} use"
} as const;

export type TranslationKey = keyof typeof en;

const zhCN: Record<TranslationKey, string> = {
  alias: "别名",
  aliasPlaceholder: "短命令，例如 review",
  all: "全部",
  appAriaPromptLauncher: "片段启动器",
  appAriaPromptLibrary: "片段库",
  appAriaPromptResults: "片段结果",
  appAriaSelectedPreview: "已选片段预览",
  body: "短片段正文",
  bodyPlaceholder: "编写可快速复用的短片段",
  clipboardPermissionDenied: "剪贴板写入权限被拒绝",
  close: "关闭",
  copiedPrompt: "已复制「{title}」",
  copy: "复制",
  copyPromptFailed: "无法复制片段",
  copyShortcut: "复制",
  createdPrompt: "已创建「{title}」",
  delete: "删除",
  deletedPrompt: "已删除「{title}」",
  editPrompt: "编辑片段",
  editShortcut: "编辑",
  favoritePrompt: "收藏片段",
  favorites: "收藏",
  filters: "片段筛选",
  language: "语言",
  languageChanged: "语言已切换",
  keyboardFirst: "键盘优先",
  lastUsed: "上次使用",
  launcher: "启动器",
  launcherDismissed: "启动器已收起",
  library: "片段库",
  loadPromptsFailed: "加载片段失败",
  localOnly: "本地",
  loadingPrompts: "正在加载片段...",
  never: "从未",
  new: "新建",
  newPrompt: "新建片段",
  noMetadata: "无元数据",
  noPromptsFound: "没有找到片段",
  noPromptsHint: "可以在片段库中新建，或调整搜索条件。",
  none: "无",
  notes: "备注",
  notesPlaceholder: "私人备注",
  pastePromptFailed: "无法粘贴片段",
  pasteShortcut: "粘贴",
  pastedPrompt: "已粘贴「{title}」",
  pinned: "置顶",
  preview: "预览",
  previewBody: "片段预览",
  previewClosed: "预览已关闭",
  previewOpened: "正在预览「{title}」",
  previewShortcut: "预览",
  promptLauncher: "片段启动器",
  promptNotFound: "未找到片段",
  promptsLoaded: "已加载 {count} 条片段",
  ready: "就绪",
  recent: "最近",
  savePrompt: "保存片段",
  savePromptFailed: "无法保存片段",
  saving: "正在保存...",
  search: "搜索",
  searchPlaceholder: "标题、别名、标签或短正文",
  tags: "标签",
  tagsPlaceholder: "写作, 编程, 评审",
  title: "标题",
  titleAndBodyRequired: "标题和短片段正文为必填项",
  titlePlaceholder: "片段标题",
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
