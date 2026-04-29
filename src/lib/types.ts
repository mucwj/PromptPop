export type PromptTag = {
  id: string;
  name: string;
  color: string | null;
  createdAt: string;
};

export type Prompt = {
  id: string;
  title: string;
  body: string;
  alias: string | null;
  notes: string | null;
  isFavorite: boolean;
  usageCount: number;
  lastUsedAt: string | null;
  createdAt: string;
  updatedAt: string;
  tags: PromptTag[];
};

export type PromptInput = {
  title: string;
  body: string;
  alias: string | null;
  notes: string | null;
  isFavorite: boolean;
  tags: string[];
};

export type PromptUpdateInput = PromptInput & {
  id: string;
};

export type LocaleCode = "en" | "zh-CN";

export type SettingSectionId = "general" | "shortcuts" | "paste" | "data" | "appearance" | "advanced";

export type ThemeChoice = "system" | "light" | "dark";

export type DefaultViewChoice = "launcher" | "library";

export type DensityChoice = "compact" | "comfortable";

export type ShortcutId =
  | "globalLauncher"
  | "focusSearch"
  | "copySelected"
  | "pasteSelected"
  | "editSelected"
  | "jumpResult";

export type ShortcutBindings = Record<ShortcutId, string>;

export type AppSettings = {
  locale: LocaleCode;
  theme: ThemeChoice;
  defaultView: DefaultViewChoice;
  density: DensityChoice;
  launchAtLogin: boolean;
  autoPasteAfterSelection: boolean;
  localBackup: boolean;
  showPreviewPane: boolean;
  showUsageCount: boolean;
  showTagsInLauncher: boolean;
  developerMode: boolean;
  shortcuts: ShortcutBindings;
};

export type SettingRecord = {
  key: string;
  value: string;
  updatedAt: string;
};

export type AppEnvironment = {
  appVersion: string;
  tauriVersion: string;
  databasePath: string;
  dataDir: string;
  logsDir: string;
  exportsDir: string;
  backupsDir: string;
  launchAtLogin: boolean;
  accessibilityTrusted: boolean | null;
};

export type SavedFile = {
  path: string;
  bytes: number;
};
