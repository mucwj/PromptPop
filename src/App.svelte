<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    copyPrompt,
    createPrompt,
    backupDatabase,
    configureWindowMode,
    deletePrompt,
    defaultSettings,
    getAppEnvironment,
    hideLauncher,
    loadSettings,
    listPrompts,
    listTags,
    onLauncherShortcut,
    onOpenSettingsRequest,
    openSettingsTarget,
    pastePrompt,
    registerLauncherShortcut,
    restoreStarterSnippets,
    saveExport,
    saveSettings,
    setLaunchAtLogin,
    setSetting,
    testPasteAutomation,
    updatePrompt
  } from "./lib/api";
  import {
    detectLocale,
    getMessages,
    locales,
    saveLocale,
    t,
    type Locale,
    type TranslationKey,
    type TranslationValues
  } from "./lib/i18n";
  import {
    detectDesktopPlatform,
    isShortcutModifier,
    shortcutKeyLabel,
    shortcutModifierExample
  } from "./lib/platform";
  import type {
    AppEnvironment,
    AppSettings,
    DefaultViewChoice,
    DensityChoice,
    Prompt,
    PromptInput,
    PromptTag,
    SavedFile,
    ShortcutId,
    ThemeChoice
  } from "./lib/types";

  type Filter = "all" | "favorites" | "recent";
  type View = "launcher" | "library" | "settings";
  type WindowMode = "launcher" | "peek" | "workspace";
  type SettingSection = "general" | "shortcuts" | "paste" | "data" | "appearance" | "advanced";

  const emptyDraft: PromptInput = {
    title: "",
    body: "",
    alias: "",
    notes: "",
    isFavorite: false,
    tags: []
  };

  const desktopPlatform = detectDesktopPlatform();
  const isMacOS = desktopPlatform === "macos";

  const settingsSectionIds: SettingSection[] = [
    "general",
    "shortcuts",
    "paste",
    "data",
    "appearance",
    "advanced"
  ];

  const shortcutRowMeta: Array<{ id: ShortcutId; active?: boolean }> = [
    { id: "globalLauncher", active: true },
    { id: "focusSearch" },
    { id: "copySelected" },
    { id: "pasteSelected" },
    { id: "editSelected" },
    { id: "jumpResult" }
  ];

  const settingsCopy = {
    en: {
      appAria: "PromptPop Settings",
      sidebarTitle: "PromptPop Settings",
      sidebarSubtitle: "Preferences",
      searchPlaceholder: "Search settings",
      navAria: "Settings sections",
      buttonLabel: "Settings",
      savedTitle: "Saved",
      savedHint: "Synced locally just now",
      savePill: "Saved",
      localOnly: "Local only",
      localReady: "Settings ready",
      savedMessage: "Settings saved",
      statuses: {
        shortcutEditReady: "Shortcut editing is ready",
        openSystemSettingsRequested: "Open System Settings requested",
        testPasteQueued: "Test Paste queued",
        openFolderRequested: "Open folder requested",
        jsonExportRequested: "JSON export requested",
        markdownExportRequested: "Markdown export requested",
        importRequested: "Import requested",
        backupRequested: "Backup requested",
        starterSnippetsAlreadyPresent: "Starter snippets already present",
        starterSnippetsRestored: "Starter snippets restored",
        appearanceApplied: "Appearance applied",
        openLogsRequested: "Open logs requested",
        diagnosticsCopied: "Diagnostics copied",
        resetConfirmationRequired: "Reset confirmation required"
      },
      sectionMeta: {
        general: { icon: "G", label: "General" },
        shortcuts: { icon: "K", label: "Shortcuts" },
        paste: { icon: "P", label: "Paste & Permissions" },
        data: { icon: "D", label: "Data" },
        appearance: { icon: "A", label: "Appearance" },
        advanced: { icon: "X", label: "Advanced" }
      },
      options: {
        english: "English",
        chinese: "Chinese",
        system: "System",
        light: "Light",
        dark: "Dark",
        launcher: "Launcher",
        library: "Snippets",
        compact: "Compact",
        comfortable: "Comfortable",
        open: "Open",
        edit: "Edit",
        import: "Import",
        restoreSnippets: "Restore snippets",
        reset: "Reset",
        backUpNow: "Back up now",
        copyReport: "Copy report",
        openFolder: "Open folder",
        openSystemSettings: "Open System Settings",
        testPaste: "Test Paste",
        applyAppearance: "Apply across launcher and snippets"
      },
      general: {
        heading: "General",
        description: "Set PromptPop's default behavior, interface language, and launch behavior.",
        languageTitle: "Language",
        languageDescription: "Choose the language used by settings and launcher text.",
        themeTitle: "Theme",
        themeDescription: "Follow the system appearance or choose light or dark mode.",
        launchTitle: "Launch at login",
        launchDescription: "Start PromptPop automatically after logging in to macOS.",
        defaultViewTitle: "Default view",
        defaultViewDescription: "Choose which view opens from the global launcher shortcut.",
        localFirstTitle: "Local first",
        localFirstDescription:
          "These preferences are stored in the local database and are not synced to the cloud. Shortcuts and permissions still require separate macOS authorization.",
        sqliteTitle: "SQLite · Local",
        sqliteDescription: "Preferences and snippets stay on this device.",
        shortcutTitle: "Common shortcuts",
        launcherShortcut: "Launcher",
        focusSearchShortcut: "Focus search",
        accessibilityTitle: "Accessibility",
        accessibilityDescription:
          "Auto paste requires macOS Accessibility permission; the shortcuts page shows paste permission status."
      },
      shortcuts: {
        heading: "Shortcuts",
        description: "Manage keyboard actions for launching, searching, editing, and inserting snippets.",
        recordingPrefix: "Recording",
        recordingTitle: "Recording shortcut",
        recordingHint: "Press the new shortcut. Esc cancels.",
        jumpRecordingHint: "Press the prefix modifier, for example {modifier}. Results use that prefix plus 1...9.",
        conflictPrefix: "Conflict with",
        conflictHint: "Choose a different shortcut to avoid unpredictable behavior.",
        unavailableHint: "Shortcut unavailable. It may already be used by the system or another app.",
        rows: {
          globalLauncher: {
            title: "Global launcher",
            description: "Open the PromptPop launcher from any app."
          },
          focusSearch: {
            title: "Focus search",
            description: "Move focus to the launcher search input."
          },
          copySelected: {
            title: "Copy selected snippet",
            description: "Copy the highlighted snippet content."
          },
          pasteSelected: {
            title: "Paste selected snippet",
            description: "Paste the highlighted snippet into the active app."
          },
          editSelected: {
            title: "Edit selected snippet",
            description: "Open the editor for the selected snippet."
          },
          jumpResult: {
            title: "Quick use result",
            description: "Choose a prefix key; results use that prefix plus 1 through 9 to use a snippet immediately."
          }
        }
      },
      paste: {
        eyebrow: "Settings",
        heading: "Paste & Permissions",
        description:
          "Control how PromptPop pastes selected snippets and verify the macOS permissions needed for reliable automation.",
        accessibilityTitle: "macOS Accessibility",
        accessibilityDescription: "Required to paste into other apps",
        required: "Required",
        clipboardTitle: "Clipboard fallback",
        clipboardDescription: "Available when paste permission is missing",
        granted: "Granted",
        autoPasteTitle: "Paste on quick use",
        autoPasteDescription:
          "When enabled, result number shortcuts send the snippet to the active app instead of only copying it.",
        clipboardNote:
          "Clipboard fallback stays active. When quick paste is disabled, result number shortcuts copy the snippet and hide PromptPop.",
        helpTitle: "Permission help",
        helpDescription:
          "Open System Settings, enable PromptPop under Accessibility, then return here and run Test Paste to confirm the active app accepts the paste event.",
        lastChecked: "Last checked 2 min ago"
      },
      data: {
        heading: "Data",
        description: "Manage the local SQLite database, language-aware starter snippets, import, export, and backups.",
        databaseTitle: "SQLite database location",
        exportTitle: "Export snippets",
        exportDescription: "Save a portable copy of snippets and tags.",
        importTitle: "Import snippets",
        importDescription: "Merge JSON or Markdown exports into the local snippet library.",
        starterTitle: "Starter snippets",
        starterDescription: "Restore the 18 starter snippets for the current app language without overwriting existing snippets.",
        backupTitle: "Local backup",
        backupDescription: "Keep a timestamped copy beside the database.",
        storageTitle: "Storage summary",
        storageDescription: "PromptPop keeps snippet content, tags, and backups on this device.",
        prompts: "Snippets",
        tags: "Tags",
        lastBackup: "Last backup",
        readyTitle: "Ready for export",
        readyDescription: "Database is readable and starter snippets can be restored by language.",
        backupEnabledTitle: "Local backup enabled",
        backupEnabledDescription: "Create promptpop-backup.sqlite before importing or restoring language-aware starter snippets."
      },
      appearance: {
        heading: "Appearance",
        description: "Tune launcher density, preview, and typography.",
        densityTitle: "Density",
        densityDescription: "Choose how tightly PromptPop packs snippet rows and metadata.",
        previewTitle: "Enable preview peek",
        previewDescription: "Use Space to temporarily inspect the selected snippet without leaving the launcher.",
        usageTitle: "Show usage count",
        usageDescription: "Display run counts beside frequently used snippets.",
        tagsTitle: "Show tags in launcher",
        tagsDescription: "Reveal snippet categories inline under each launcher result.",
        uiFontTitle: "UI font",
        uiFontDescription: "Geist keeps settings, labels, and metadata crisp at compact density.",
        fontPreviewLabel: "Launcher label · 12px",
        promptFontTitle: "Snippet preview font",
        promptFontDescription: "JetBrains Mono preserves indentation and placeholder syntax in snippet bodies.",
        promptPreviewSample: "{{topic}} summary",
        promptPreviewNote: "Keep concise",
        behaviorTitle: "Preview behavior",
        behaviorDescription:
          "Appearance changes apply to launcher chrome first, then snippet rows and the preview pane.",
        themeDirectionTitle: "Theme preview direction",
        themeDirectionDescription: "Preview left-to-right from current theme to target theme before applying.",
        narrowTitle: "Narrow windows",
        narrowDescription:
          "Below 720px, settings stack above previews. The preview pane condenses before metadata hides."
      },
      advanced: {
        heading: "Advanced",
        description: "Diagnostics, logs, and experimental controls for troubleshooting.",
        appPill: "Tauri app",
        developerTitle: "Developer mode",
        developerDescription: "Show debug commands, internal IDs, and more detailed error messages.",
        logsTitle: "Open logs folder",
        logsDescription: "View logs related to Tauri and the local database.",
        diagnosticsTitle: "Diagnostics",
        appVersion: "App version",
        databaseRevision: "Database revision",
        tauriVersion: "Tauri version",
        osPermission: "OS permission",
        accessibilityGranted: "Accessibility granted",
        resetTitle: "Reset settings",
        resetDescription: "Restore default preferences without deleting SQLite snippet data.",
        dangerConfirmation: "Dangerous actions require keyboard confirmation before they run.",
        principleTitle: "Advanced settings principles",
        principleDescription:
          "These operations are mainly for troubleshooting. Dangerous actions require confirmation, and logs and diagnostics stay on this device by default.",
        systemHealth: "System health",
        database: "Database",
        shortcutHook: "Shortcut hook",
        active: "active",
        pending: "pending",
        logs: "Logs",
        noRestartTitle: "No restart required",
        noRestartDescription: "Most advanced changes apply immediately."
      }
    },
    "zh-CN": {
      appAria: "PromptPop 设置",
      sidebarTitle: "PromptPop 设置",
      sidebarSubtitle: "偏好设置",
      searchPlaceholder: "搜索设置",
      navAria: "设置分区",
      buttonLabel: "设置",
      savedTitle: "已保存",
      savedHint: "已保存到本机",
      savePill: "已保存",
      localOnly: "仅本地",
      localReady: "设置已就绪",
      savedMessage: "设置已保存",
      statuses: {
        shortcutEditReady: "快捷键编辑已就绪",
        openSystemSettingsRequested: "已请求打开系统设置",
        testPasteQueued: "测试粘贴已加入队列",
        openFolderRequested: "已请求打开文件夹",
        jsonExportRequested: "已请求导出 JSON",
        markdownExportRequested: "已请求导出 Markdown",
        importRequested: "已请求导入",
        backupRequested: "已请求备份",
        starterSnippetsAlreadyPresent: "默认短片段已存在",
        starterSnippetsRestored: "默认短片段已恢复",
        appearanceApplied: "外观设置已应用",
        openLogsRequested: "已请求打开日志",
        diagnosticsCopied: "诊断信息已复制",
        resetConfirmationRequired: "需要确认后才能重置"
      },
      sectionMeta: {
        general: { icon: "通", label: "通用" },
        shortcuts: { icon: "键", label: "快捷键" },
        paste: { icon: "贴", label: "粘贴与权限" },
        data: { icon: "数", label: "数据" },
        appearance: { icon: "外", label: "外观" },
        advanced: { icon: "高", label: "高级" }
      },
      options: {
        english: "英语",
        chinese: "中文",
        system: "跟随系统",
        light: "浅色",
        dark: "深色",
        launcher: "启动器",
        library: "片段库",
        compact: "紧凑",
        comfortable: "舒适",
        open: "打开",
        edit: "编辑",
        import: "导入",
        restoreSnippets: "恢复片段",
        reset: "重置",
        backUpNow: "立即备份",
        copyReport: "复制报告",
        openFolder: "打开文件夹",
        openSystemSettings: "打开系统设置",
        testPaste: "测试粘贴",
        applyAppearance: "应用到启动器和片段库"
      },
      general: {
        heading: "通用",
        description: "设置 PromptPop 的默认行为、界面语言和启动方式。",
        languageTitle: "语言",
        languageDescription: "选择设置页和启动器文字使用的语言。",
        themeTitle: "主题",
        themeDescription: "跟随系统外观，或指定浅色、深色模式。",
        launchTitle: "开机启动",
        launchDescription: "登录 macOS 后自动启动 PromptPop。",
        defaultViewTitle: "默认打开页面",
        defaultViewDescription: "选择通过全局启动器快捷键打开后的页面。",
        localFirstTitle: "本地优先",
        localFirstDescription:
          "这些偏好会保存在本机数据库中，不会同步到云端。快捷键和权限仍需要 macOS 单独授权。",
        sqliteTitle: "SQLite · 本地",
        sqliteDescription: "偏好与短片段会保存在本机。",
        shortcutTitle: "常用快捷键",
        launcherShortcut: "启动器",
        focusSearchShortcut: "聚焦搜索",
        accessibilityTitle: "辅助功能",
        accessibilityDescription: "自动粘贴需要 macOS 辅助功能权限；快捷键页面会显示粘贴权限状态。"
      },
      shortcuts: {
        heading: "快捷键",
        description: "管理启动、搜索、编辑和插入片段的键盘操作。",
        recordingPrefix: "正在录制",
        recordingTitle: "正在录制快捷键",
        recordingHint: "按下新的快捷键。Esc 取消。",
        jumpRecordingHint: "请按下前置修饰键，例如 {modifier}。实际使用时会是该按键加 1 到 9 并立即使用片段。",
        conflictPrefix: "与以下快捷键冲突：",
        conflictHint: "请换一个组合，避免触发行为不可预测。",
        unavailableHint: "快捷键不可用，可能已被系统或其他应用占用。",
        rows: {
          globalLauncher: {
            title: "全局启动器",
            description: "从任意应用打开 PromptPop 启动器。"
          },
          focusSearch: {
            title: "聚焦搜索",
            description: "将焦点移动到启动器搜索框。"
          },
          copySelected: {
            title: "复制选中的片段",
            description: "复制当前高亮的短片段内容。"
          },
          pasteSelected: {
            title: "粘贴选中的片段",
            description: "把当前高亮的短片段粘贴到活跃应用。"
          },
          editSelected: {
            title: "编辑选中的片段",
            description: "打开所选片段的编辑面板。"
          },
          jumpResult: {
            title: "快速使用结果",
            description: "选择前置按键；实际使用时是该按键加 1 到 9，并立即使用对应片段。"
          }
        }
      },
      paste: {
        eyebrow: "设置",
        heading: "粘贴与权限",
        description: "控制 PromptPop 如何粘贴选中的短片段，并检查可靠自动化所需的 macOS 权限。",
        accessibilityTitle: "macOS 辅助功能",
        accessibilityDescription: "粘贴到其他应用时需要此权限",
        required: "必需",
        clipboardTitle: "剪贴板兜底",
        clipboardDescription: "缺少粘贴权限时仍可使用",
        granted: "已授权",
        autoPasteTitle: "快速使用时粘贴",
        autoPasteDescription: "启用后，结果数字快捷键会把短片段发送到当前活跃应用，而不是只复制。",
        clipboardNote: "剪贴板兜底始终可用。关闭快速粘贴时，结果数字快捷键会复制片段并隐藏 PromptPop。",
        helpTitle: "权限帮助",
        helpDescription:
          "打开系统设置，在辅助功能中启用 PromptPop，然后回到这里运行测试粘贴，确认活跃应用能接收粘贴事件。",
        lastChecked: "上次检查：2 分钟前"
      },
      data: {
        heading: "数据",
        description: "管理本地 SQLite 数据库、按语言初始化的默认短片段、导入导出和备份。",
        databaseTitle: "SQLite 数据库位置",
        exportTitle: "导出片段",
        exportDescription: "保存短片段和标签的可迁移副本。",
        importTitle: "导入片段",
        importDescription: "将 JSON 或 Markdown 导出合并到本地片段库。",
        starterTitle: "默认短片段",
        starterDescription: "恢复与当前界面语言匹配的 18 条短片段；不会覆盖已有内容。",
        backupTitle: "本地备份",
        backupDescription: "在数据库旁保留带时间戳的副本。",
        storageTitle: "存储概览",
        storageDescription: "PromptPop 会把短片段、标签和备份保存在这台设备上。",
        prompts: "短片段",
        tags: "标签",
        lastBackup: "上次备份",
        readyTitle: "可导出",
        readyDescription: "数据库可读取，默认短片段会按语言恢复。",
        backupEnabledTitle: "已启用本地备份",
        backupEnabledDescription: "导入或按当前语言恢复默认短片段前可先创建 promptpop-backup.sqlite。"
      },
      appearance: {
        heading: "外观",
        description: "调整启动器密度、预览和字体。",
        densityTitle: "密度",
        densityDescription: "选择 PromptPop 如何紧凑地展示片段行和元数据。",
        previewTitle: "启用快速预览",
        previewDescription: "在启动器中按 Space 临时查看已选片段，无需离开当前搜索。",
        usageTitle: "显示使用次数",
        usageDescription: "在常用片段旁显示运行次数。",
        tagsTitle: "在启动器中显示标签",
        tagsDescription: "在每条启动器结果下方显示片段分类。",
        uiFontTitle: "界面字体",
        uiFontDescription: "Geist 让设置、标签和元数据在紧凑密度下保持清晰。",
        fontPreviewLabel: "启动器标签 · 12px",
        promptFontTitle: "片段预览字体",
        promptFontDescription: "JetBrains Mono 会保留短片段正文中的缩进和占位符语法。",
        promptPreviewSample: "{{主题}} 摘要",
        promptPreviewNote: "保持简洁",
        behaviorTitle: "预览行为",
        behaviorDescription: "外观变化会先应用到启动器框架，再应用到片段行和预览面板。",
        themeDirectionTitle: "主题预览方向",
        themeDirectionDescription: "从当前主题向目标主题从左到右预览后再应用。",
        narrowTitle: "窄窗口",
        narrowDescription: "低于 720px 时，设置会堆叠在预览上方。元数据隐藏前，预览面板会先收窄。"
      },
      advanced: {
        heading: "高级",
        description: "用于排障的诊断、日志和实验控件。",
        appPill: "Tauri 应用",
        developerTitle: "开发者模式",
        developerDescription: "显示调试命令、内部 ID 和更详细的错误信息。",
        logsTitle: "打开日志文件夹",
        logsDescription: "查看与 Tauri 和本地数据库相关的日志。",
        diagnosticsTitle: "诊断",
        appVersion: "应用版本",
        databaseRevision: "数据库版本",
        tauriVersion: "Tauri 版本",
        osPermission: "系统权限",
        accessibilityGranted: "辅助功能已授权",
        resetTitle: "重置设置",
        resetDescription: "恢复默认偏好设置，但不删除 SQLite 片段数据。",
        dangerConfirmation: "危险操作会先要求键盘确认，不会立即执行。",
        principleTitle: "高级设置原则",
        principleDescription: "这些操作主要用于排障。危险操作保持二次确认，日志和诊断信息默认只留在本机。",
        systemHealth: "系统状态",
        database: "数据库",
        shortcutHook: "快捷键挂钩",
        active: "活跃",
        pending: "待处理",
        logs: "日志",
        noRestartTitle: "无需重启",
        noRestartDescription: "大多数高级更改会立即生效。"
      }
    }
  } satisfies Record<Locale, unknown>;

  let prompts: Prompt[] = [];
  let tags: PromptTag[] = [];
  let query = "";
  let filter: Filter = "all";
  let selectedIndex = 0;
  let selectedId: string | null = null;
  let previewPeekOpen = false;
  let appliedWindowMode: WindowMode | null = null;
  let requestedWindowMode: WindowMode | null = null;
  let windowModeRequestId = 0;
  let windowModeQueue = Promise.resolve();
  let editingId: string | null = null;
  let view: View = "launcher";
  let activeSettingsSection: SettingSection = "general";
  let locale: Locale = detectLocale();
  let text = getMessages(locale);
  let draft: PromptInput = { ...emptyDraft };
  let tagText = "";
  let status = t(locale, "ready");
  let loading = true;
  let saving = false;
  let searchInput: HTMLInputElement;
  let importInput: HTMLInputElement;
  let settingsQuery = "";
  let recordingShortcutId: ShortcutId | null = null;
  let resetSettingsArmed = false;
  let appEnvironment: AppEnvironment | null = null;
  let lastBackupAt: string | null = null;
  let lastExportPath: string | null = null;
  let shortcutNotice = "";
  let themeChoice: ThemeChoice = defaultSettings.theme;
  let defaultViewChoice: DefaultViewChoice = defaultSettings.defaultView;
  let densityChoice: DensityChoice = defaultSettings.density;
  let launchAtLogin = defaultSettings.launchAtLogin;
  let autoPasteAfterSelection = defaultSettings.autoPasteAfterSelection;
  let localBackup = defaultSettings.localBackup;
  let showPreviewPane = defaultSettings.showPreviewPane;
  let showUsageCount = defaultSettings.showUsageCount;
  let showTagsInLauncher = defaultSettings.showTagsInLauncher;
  let developerMode = defaultSettings.developerMode;
  let shortcutBindings = { ...defaultSettings.shortcuts };

  $: text = getMessages(locale);
  $: settings = settingsCopy[locale];
  $: platformPasteSettings = pasteSettingsForPlatform(settings.paste);
  $: platformGeneralPermission = generalPermissionForPlatform(settings.general);
  $: platformLaunchDescription = launchDescriptionForPlatform(settings.general.launchDescription);
  $: platformLocalFirstDescription = localFirstDescriptionForPlatform(settings.general.localFirstDescription);
  $: pasteAutomationReady = !isMacOS || appEnvironment?.accessibilityTrusted === true;
  $: pasteAutomationStatus = pasteAutomationReady
    ? platformPasteSettings.granted
    : platformPasteSettings.lastChecked;
  $: settingsSections = settingsSectionIds
    .map((id) => ({
      id,
      icon: settings.sectionMeta[id].icon,
      label: settings.sectionMeta[id].label
    }))
    .filter((item) => item.label.toLowerCase().includes(settingsQuery.trim().toLowerCase()));
  $: shortcutRows = shortcutRowMeta.map((row) => ({
    ...row,
    keys: shortcutLabels(shortcutBindings[row.id]),
    title: settings.shortcuts.rows[row.id].title,
    description: settings.shortcuts.rows[row.id].description,
    conflict: shortcutConflict(row.id),
    recording: recordingShortcutId === row.id
  }));
  $: filteredPrompts = getFilteredPrompts(prompts, filter, query, locale);
  $: selectedPrompt = filteredPrompts[selectedIndex] ?? filteredPrompts[0] ?? null;
  $: libraryPrompt =
    prompts.find((prompt) => prompt.id === selectedId) ?? filteredPrompts[0] ?? prompts[0] ?? null;
  $: favoriteCount = prompts.filter((prompt) => prompt.isFavorite).length;
  $: recentCount = prompts.filter((prompt) => prompt.lastUsedAt).length;
  $: selectedVariables = selectedPrompt ? extractVariables(selectedPrompt.body) : [];
  $: searchSummary = getSearchSummary(filteredPrompts.length, prompts.length, query);
  $: if ((!selectedPrompt || view !== "launcher" || !showPreviewPane) && previewPeekOpen) previewPeekOpen = false;
  $: syncWindowMode(view === "launcher" ? (previewPeekOpen && selectedPrompt ? "peek" : "launcher") : "workspace");
  $: settingsButtonLabel = settings.buttonLabel;
  $: settingsSavedTitle = settings.savedTitle;
  $: settingsSavedHint = settings.savedHint;
  $: if (selectedIndex >= filteredPrompts.length) selectedIndex = Math.max(filteredPrompts.length - 1, 0);
  $: applyAppearanceSettings(themeChoice, densityChoice);

  onMount(() => {
    document.documentElement.lang = locale;

    void bootstrap();
    let unlistenLauncher: (() => void) | null = null;
    let unlistenOpenSettings: (() => void) | null = null;
    void onLauncherShortcut(() => openDefaultView()).then((unlisten) => {
      unlistenLauncher = unlisten;
    });
    void onOpenSettingsRequest(() => openSettings()).then((unlisten) => {
      unlistenOpenSettings = unlisten;
    });

    const onKeydown = (event: KeyboardEvent) => {
      if (recordingShortcutId) {
        captureShortcut(event);
        return;
      }

      if (matchesShortcut(event, shortcutBindings.focusSearch)) {
        event.preventDefault();
        showLauncherView();
      }

      if ((event.metaKey || event.ctrlKey) && event.key === ",") {
        event.preventDefault();
        openSettings();
      }

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "n") {
        event.preventDefault();
        startNewPrompt();
      }

      if (matchesShortcut(event, shortcutBindings.editSelected)) {
        event.preventDefault();
        if (selectedPrompt) startEdit(selectedPrompt);
      }

      if (event.key === "Escape") {
        if (previewPeekOpen) {
          event.preventDefault();
          closePreviewPeek();
        } else if (query) {
          query = "";
          selectedIndex = 0;
        } else if (view !== "launcher") {
          showLauncherView();
        } else {
          status = translate("launcherDismissed");
        }
      }

      if (view !== "launcher") return;

      if (shouldTogglePreview(event)) {
        event.preventDefault();
        togglePreviewPeek();
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, filteredPrompts.length - 1);
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
      }

      if (matchesShortcut(event, shortcutBindings.copySelected) && selectedPrompt) {
        event.preventDefault();
        copySelected();
      }

      if (matchesShortcut(event, shortcutBindings.pasteSelected) && selectedPrompt) {
        event.preventDefault();
        pasteSelected();
      }

      const jumpIndex = jumpResultIndex(event);
      if (matchesJumpShortcut(event) && jumpIndex !== null) {
        const prompt = filteredPrompts[jumpIndex];
        if (prompt) {
          event.preventDefault();
          void quickUsePrompt(prompt, jumpIndex);
        }
      }
    };

    window.addEventListener("keydown", onKeydown);
    return () => {
      window.removeEventListener("keydown", onKeydown);
      unlistenLauncher?.();
      unlistenOpenSettings?.();
    };
  });

  function translate(key: TranslationKey, values?: TranslationValues): string {
    return t(locale, key, values);
  }

  function pasteSettingsForPlatform(paste: typeof settingsCopy.en.paste): typeof settingsCopy.en.paste {
    if (desktopPlatform === "windows") {
      return locale === "zh-CN"
        ? {
            ...paste,
            heading: "粘贴行为",
            description: "控制 PromptPop 如何复制片段，并在 Windows 上通过剪贴板和 Ctrl+V 尝试自动粘贴。",
            accessibilityTitle: "Windows 自动粘贴",
            accessibilityDescription: "使用剪贴板并向上一个活跃窗口发送 Ctrl+V",
            required: "可用",
            clipboardDescription: "目标应用不接收自动粘贴时仍可手动粘贴",
            granted: "可用",
            clipboardNote:
              "剪贴板兜底始终可用。Windows 自动粘贴会先隐藏 PromptPop，再尝试回到上一个窗口发送 Ctrl+V。",
            helpTitle: "Windows 行为说明",
            helpDescription:
              "Windows 不需要 macOS 辅助功能权限。管理员窗口、UAC 安全桌面或部分受保护应用可能拒绝模拟按键；这种情况下片段仍会保留在剪贴板。",
            lastChecked: "无需单独授权"
          }
        : {
            ...paste,
            heading: "Paste Behavior",
            description:
              "Control how PromptPop copies snippets and attempts Windows auto paste through the clipboard plus Ctrl+V.",
            accessibilityTitle: "Windows auto paste",
            accessibilityDescription: "Uses the clipboard and sends Ctrl+V to the previous active window",
            required: "Available",
            clipboardDescription: "Still works when the target app does not accept auto paste",
            granted: "Available",
            clipboardNote:
              "Clipboard fallback is always available. On Windows, auto paste hides PromptPop, returns to the previous window, then sends Ctrl+V.",
            helpTitle: "Windows behavior",
            helpDescription:
              "Windows does not require macOS Accessibility permission. Admin windows, the UAC secure desktop, and some protected apps may reject simulated keys; the snippet remains on the clipboard when that happens.",
            lastChecked: "No separate permission required"
          };
    }

    if (desktopPlatform === "linux") {
      return locale === "zh-CN"
        ? {
            ...paste,
            heading: "粘贴行为",
            description: "控制 PromptPop 如何复制片段；当前 Linux 桌面以剪贴板兜底为主。",
            accessibilityTitle: "Linux 自动粘贴",
            accessibilityDescription: "当前未启用跨桌面自动粘贴，建议使用剪贴板手动粘贴",
            required: "未启用",
            clipboardDescription: "复制后可在目标应用中手动粘贴",
            granted: "可用",
            clipboardNote: "剪贴板兜底始终可用。Linux 自动粘贴会在后续按桌面环境分别适配。",
            helpTitle: "Linux 行为说明",
            helpDescription: "不同桌面环境对模拟按键支持差异较大；当前版本不显示 macOS 权限，也不假装执行权限检查。",
            lastChecked: "剪贴板可用"
          }
        : {
            ...paste,
            heading: "Paste Behavior",
            description: "Control how PromptPop copies snippets; this Linux build currently favors clipboard fallback.",
            accessibilityTitle: "Linux auto paste",
            accessibilityDescription: "Cross-desktop auto paste is not enabled yet; paste manually from the clipboard",
            required: "Not enabled",
            clipboardDescription: "Copy first, then paste manually in the target app",
            granted: "Available",
            clipboardNote: "Clipboard fallback is always available. Linux auto paste can be adapted per desktop environment later.",
            helpTitle: "Linux behavior",
            helpDescription:
              "Simulated key support varies across desktop environments, so this version avoids showing macOS permissions or pretending to check them.",
            lastChecked: "Clipboard available"
          };
    }

    return paste;
  }

  function generalPermissionForPlatform(general: typeof settingsCopy.en.general) {
    if (desktopPlatform === "windows") {
      return locale === "zh-CN"
        ? {
            title: "Windows 自动粘贴",
            description: "自动粘贴使用剪贴板和 Ctrl+V；受保护窗口不接收时仍可手动粘贴。"
          }
        : {
            title: "Windows auto paste",
            description: "Auto paste uses the clipboard plus Ctrl+V; protected windows can still be pasted into manually."
          };
    }

    if (desktopPlatform === "linux") {
      return locale === "zh-CN"
        ? {
            title: "剪贴板粘贴",
            description: "当前 Linux 版本以剪贴板为兜底，不显示 macOS 辅助功能权限。"
          }
        : {
            title: "Clipboard paste",
            description: "This Linux build uses clipboard fallback and does not show macOS Accessibility permission."
          };
    }

    return {
      title: general.accessibilityTitle,
      description: general.accessibilityDescription
    };
  }

  function launchDescriptionForPlatform(fallback: string): string {
    if (desktopPlatform === "windows") {
      return locale === "zh-CN"
        ? "登录 Windows 后自动启动 PromptPop。"
        : "Start PromptPop automatically after signing in to Windows.";
    }

    if (desktopPlatform === "linux") {
      return locale === "zh-CN"
        ? "当前 Linux 版本暂不启用系统级开机启动。"
        : "System launch at login is not enabled for this Linux build yet.";
    }

    return fallback;
  }

  function localFirstDescriptionForPlatform(fallback: string): string {
    if (desktopPlatform === "windows") {
      return locale === "zh-CN"
        ? "这些偏好会保存在本机数据库中，不会同步到云端。快捷键、自动粘贴和开机启动都使用 Windows 本机能力。"
        : "These preferences are stored in the local database and are not synced to the cloud. Shortcuts, auto paste, and launch at login use native Windows behavior.";
    }

    if (desktopPlatform === "linux") {
      return locale === "zh-CN"
        ? "这些偏好会保存在本机数据库中，不会同步到云端。当前 Linux 版本以本机快捷键和剪贴板兜底为主。"
        : "These preferences are stored in the local database and are not synced to the cloud. This Linux build favors native shortcuts and clipboard fallback.";
    }

    return fallback;
  }

  function shouldTogglePreview(event: KeyboardEvent): boolean {
    if (event.code !== "Space" || event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return false;
    if (!selectedPrompt || !showPreviewPane) return false;
    const target = event.target;
    if (!(target instanceof HTMLElement)) return false;
    if (target instanceof HTMLTextAreaElement || target.isContentEditable) return false;
    if (target instanceof HTMLInputElement && target !== searchInput) return false;
    if (target === searchInput && query.trim()) return false;
    return true;
  }

  function syncWindowMode(mode: WindowMode) {
    applyWindowMode(mode);
  }

  function applyWindowMode(mode: WindowMode, force = false) {
    if (!force && mode === appliedWindowMode && mode === requestedWindowMode) return;
    requestedWindowMode = mode;
    const requestId = ++windowModeRequestId;

    windowModeQueue = windowModeQueue
      .catch(() => undefined)
      .then(async () => {
        if (requestId !== windowModeRequestId) return;
        await configureWindowMode(mode);
        if (requestId === windowModeRequestId) {
          appliedWindowMode = mode;
        }
      })
      .catch(() => {
        if (requestId === windowModeRequestId) {
          requestedWindowMode = appliedWindowMode;
        }
      });
  }

  function translateError(error: unknown, fallbackKey: TranslationKey): string {
    if (!(error instanceof Error)) return translate(fallbackKey);
    if (error.message === "Clipboard write permission denied") return translate("clipboardPermissionDenied");
    if (error.message === "Prompt not found") return translate("promptNotFound");
    return error.message || translate(fallbackKey);
  }

  async function bootstrap() {
    await loadPersistedSettings();
    await refreshEnvironment();
    await refresh();
    openDefaultView();
  }

  async function loadPersistedSettings() {
    try {
      const loaded = await loadSettings({ locale });
      applySettingsState(loaded);
      saveLocale(loaded.locale);
      document.documentElement.lang = loaded.locale;
    } catch (error) {
      status = translateError(error, "loadPromptsFailed");
    }
  }

  function applySettingsState(next: AppSettings) {
    locale = next.locale;
    themeChoice = next.theme;
    defaultViewChoice = next.defaultView;
    densityChoice = next.density;
    launchAtLogin = next.launchAtLogin;
    autoPasteAfterSelection = next.autoPasteAfterSelection;
    localBackup = next.localBackup;
    showPreviewPane = next.showPreviewPane;
    showUsageCount = next.showUsageCount;
    showTagsInLauncher = next.showTagsInLauncher;
    developerMode = next.developerMode;
    shortcutBindings = normalizeShortcutBindings({ ...defaultSettings.shortcuts, ...next.shortcuts });
  }

  function currentSettings(): AppSettings {
    return {
      locale,
      theme: themeChoice,
      defaultView: defaultViewChoice,
      density: densityChoice,
      launchAtLogin,
      autoPasteAfterSelection,
      localBackup,
      showPreviewPane,
      showUsageCount,
      showTagsInLauncher,
      developerMode,
      shortcuts: { ...shortcutBindings }
    };
  }

  async function persistSetting(key: string, value: string, message?: string) {
    try {
      await setSetting(key, value);
      markSettingsSaved(message);
    } catch (error) {
      status = error instanceof Error ? error.message : settings.savedMessage;
    }
  }

  async function persistAllSettings(message?: string) {
    try {
      await saveSettings(currentSettings());
      markSettingsSaved(message);
    } catch (error) {
      status = error instanceof Error ? error.message : settings.savedMessage;
    }
  }

  async function refreshEnvironment() {
    try {
      appEnvironment = await getAppEnvironment();
      launchAtLogin = appEnvironment.launchAtLogin;
    } catch {
      appEnvironment = null;
    }
  }

  function openDefaultView() {
    if (defaultViewChoice === "library") {
      if (selectedPrompt) {
        startEdit(selectedPrompt);
      } else {
        view = "library";
      }
    } else {
      showLauncherView();
    }
  }

  function showLauncherView() {
    previewPeekOpen = false;
    view = "launcher";
    applyWindowMode("launcher", true);
    tick().then(() => searchInput?.focus());
  }

  function applyAppearanceSettings(theme: ThemeChoice, density: DensityChoice) {
    if (typeof document === "undefined") return;
    document.documentElement.dataset.theme = theme;
    document.documentElement.dataset.density = density;
  }

  function promptListMeta(prompt: Prompt) {
    const alias = prompt.alias ? `/${prompt.alias.replace(/^\/+/, "")}` : "";
    const tagNames = showTagsInLauncher ? prompt.tags.map((tag) => `#${tag.name}`).join(" · ") : "";
    const debugId = developerMode ? `id:${prompt.id.slice(0, 8)}` : "";
    return [alias, tagNames, debugId].filter(Boolean).join(" · ") || text.noMetadata;
  }

  function normalizeShortcutBindings(bindings: typeof shortcutBindings) {
    return {
      ...bindings,
      jumpResult: normalizeJumpPrefix(bindings.jumpResult)
    };
  }

  function normalizeJumpPrefix(shortcut: string): string {
    const modifiers = shortcut
      .split("+")
      .filter(isShortcutModifier);
    return modifiers.length ? modifiers.join("+") : defaultSettings.shortcuts.jumpResult;
  }

  function shortcutLabels(shortcut: string): string[] {
    const isJumpPrefix = !shortcut.split("+").some((part) => !isShortcutModifier(part));
    const labels = shortcut.split("+").filter(Boolean).map((part) => shortcutKeyLabel(part, desktopPlatform));
    return isJumpPrefix ? [...labels, "1...9"] : labels;
  }

  function shortcutRecordingHint(id: ShortcutId): string {
    const hint = id === "jumpResult" ? settings.shortcuts.jumpRecordingHint : settings.shortcuts.recordingHint;
    return hint.replace("{modifier}", shortcutModifierExample(desktopPlatform));
  }

  function shortcutText(shortcut: string): string {
    return shortcutLabels(shortcut).join(" ");
  }

  function resultJumpShortcut(prefix: string, index: number): string {
    const digit = Math.min(index + 1, 9);
    return `${normalizeJumpPrefix(prefix)}+Digit${digit}`;
  }

  function shortcutPrefixLabels(shortcut: string): string[] {
    return shortcutLabels(normalizeJumpPrefix(shortcut)).filter((label) => label !== "1...9");
  }

  function normalizeEventKey(event: KeyboardEvent): string | null {
    if (event.code.startsWith("Key") || event.code.startsWith("Digit")) return event.code;
    if (event.code === "Space") return "Space";
    if (event.key === "Enter") return "Enter";
    if (event.key === "Escape") return "Escape";
    if (/^F\\d{1,2}$/.test(event.key)) return event.key;
    if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Tab", "Backspace", "Delete"].includes(event.key)) {
      return event.key;
    }
    return event.key.length === 1 ? event.key.toUpperCase() : null;
  }

  function eventShortcut(event: KeyboardEvent): string | null {
    const key = normalizeEventKey(event);
    if (!key || ["Meta", "Control", "Alt", "Shift"].includes(key)) return null;
    const parts = [];
    if (event.metaKey) parts.push("Meta");
    if (event.ctrlKey) parts.push("Control");
    if (event.altKey) parts.push("Alt");
    if (event.shiftKey) parts.push("Shift");
    parts.push(key);
    return parts.join("+");
  }

  function eventModifierPrefix(event: KeyboardEvent): string {
    const parts = [];
    if (event.metaKey || event.key === "Meta") parts.push("Meta");
    if (event.ctrlKey || event.key === "Control") parts.push("Control");
    if (event.altKey || event.key === "Alt") parts.push("Alt");
    if (event.shiftKey || event.key === "Shift") parts.push("Shift");
    return parts.join("+");
  }

  function matchesShortcut(event: KeyboardEvent, shortcut: string): boolean {
    return eventShortcut(event) === shortcut;
  }

  function matchesJumpShortcut(event: KeyboardEvent): boolean {
    return eventModifierPrefix(event) === normalizeJumpPrefix(shortcutBindings.jumpResult) && jumpResultIndex(event) !== null;
  }

  function jumpResultIndex(event: KeyboardEvent): number | null {
    const codeMatch = event.code.match(/^Digit([1-9])$/);
    const keyMatch = event.key.match(/^[1-9]$/);
    const number = codeMatch ? Number(codeMatch[1]) : keyMatch ? Number(event.key) : null;
    return number ? number - 1 : null;
  }

  function captureShortcut(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      recordingShortcutId = null;
      return;
    }

    const id = recordingShortcutId;
    if (!id) return;

    if (id === "jumpResult") {
      const prefix = eventModifierPrefix(event);
      if (!prefix) return;
      recordingShortcutId = null;
      void updateShortcut(id, prefix);
      return;
    }

    const shortcut = eventShortcut(event);
    if (!shortcut) return;
    recordingShortcutId = null;
    void updateShortcut(id, shortcut);
  }

  async function updateShortcut(id: ShortcutId, shortcut: string) {
    const normalized = id === "jumpResult" ? normalizeJumpPrefix(shortcut) : shortcut;
    const conflict = shortcutConflict(id, normalized);
    if (conflict) {
      shortcutNotice = `${settings.shortcuts.conflictPrefix} ${conflict}. ${settings.shortcuts.conflictHint}`;
      status = shortcutNotice;
      return;
    }

    shortcutNotice = "";
    const previous = shortcutBindings[id];
    shortcutBindings = { ...shortcutBindings, [id]: normalized };
    try {
      if (id === "globalLauncher") {
        const registered = await registerLauncherShortcut(normalized);
        shortcutBindings = { ...shortcutBindings, [id]: registered };
        await setSetting(`shortcuts.${id}`, registered);
      } else {
        await setSetting(`shortcuts.${id}`, normalized);
      }
      markSettingsSaved(settings.savedMessage);
    } catch (error) {
      shortcutBindings = { ...shortcutBindings, [id]: previous };
      status = error instanceof Error ? `${settings.shortcuts.unavailableHint} ${error.message}` : settings.shortcuts.unavailableHint;
    }
  }

  function shortcutConflict(id: ShortcutId, candidate = shortcutBindings[id]): string | null {
    const normalizedCandidate = id === "jumpResult" ? normalizeJumpPrefix(candidate) : candidate;

    for (const row of shortcutRowMeta) {
      if (row.id === id) continue;
      const other = shortcutBindings[row.id];
      if (row.id === "jumpResult") {
        if (digitShortcutUsesPrefix(normalizedCandidate, normalizeJumpPrefix(other))) {
          return settings.shortcuts.rows[row.id].title;
        }
      } else if (id === "jumpResult") {
        if (digitShortcutUsesPrefix(other, normalizedCandidate)) {
          return settings.shortcuts.rows[row.id].title;
        }
      } else if (other === normalizedCandidate) {
        return settings.shortcuts.rows[row.id].title;
      }
    }

    return null;
  }

  function digitShortcutUsesPrefix(shortcut: string, prefix: string): boolean {
    return shortcut.startsWith(`${prefix}+Digit`) && /^Digit[1-9]$/.test(shortcut.slice(prefix.length + 1));
  }

  function getFilteredPrompts(
    source: Prompt[],
    activeFilter: Filter,
    search: string,
    activeLocale: Locale
  ): Prompt[] {
    const needle = search.trim().toLowerCase();
    const filtered = source.filter((prompt) => {
      if (activeFilter === "favorites" && !prompt.isFavorite) return false;
      if (activeFilter === "recent" && !prompt.lastUsedAt) return false;
      if (!needle) return true;
      return [
        prompt.title,
        prompt.alias ?? "",
        prompt.body,
        prompt.notes ?? "",
        prompt.tags.map((tag) => tag.name).join(" ")
      ]
        .join(" ")
        .toLowerCase()
        .includes(needle);
    });

    return filtered.sort((a, b) => {
      if (activeFilter === "recent") {
        return Date.parse(b.lastUsedAt ?? "") - Date.parse(a.lastUsedAt ?? "");
      }
      if (b.isFavorite !== a.isFavorite) return Number(b.isFavorite) - Number(a.isFavorite);
      return b.usageCount - a.usageCount || a.title.localeCompare(b.title, activeLocale);
    });
  }

  function getSearchSummary(matchCount: number, totalCount: number, search: string) {
    if (!search.trim()) return `${totalCount} ${text.appAriaPromptResults.toLowerCase()}`;
    return `${matchCount} / ${totalCount}`;
  }

  function extractVariables(body: string): string[] {
    return Array.from(new Set(Array.from(body.matchAll(/\{\{\s*([^}]+?)\s*\}\}/g), (match) => match[1].trim())));
  }

  function promptMeta(prompt: Prompt) {
    const tagNames = prompt.tags.map((tag) => tag.name).join(", ");
    const alias = prompt.alias ? `${prompt.alias} / ` : "";
    return `${alias}${tagNames || text.noMetadata}`;
  }

  function promptSnippet(prompt: Prompt) {
    return prompt.body.replace(/\s+/g, " ").trim();
  }

  async function refresh() {
    loading = true;
    try {
      [prompts, tags] = await Promise.all([listPrompts(), listTags()]);
      selectedId = selectedId ?? prompts[0]?.id ?? null;
      status = translate("promptsLoaded", { count: prompts.length });
    } catch (error) {
      status = translateError(error, "loadPromptsFailed");
    } finally {
      loading = false;
    }
  }

  function setLocale(next: Locale) {
    locale = next;
    saveLocale(next);
    document.documentElement.lang = next;
    status = translate("languageChanged");
    void persistSetting("locale", next, t(next, "languageChanged"));
  }

  function setFilter(next: Filter) {
    filter = next;
    selectedIndex = 0;
    tick().then(() => searchInput?.focus());
  }

  function openSettings(section: SettingSection = "general") {
    previewPeekOpen = false;
    view = "settings";
    activeSettingsSection = section;
    status = settings.localReady;
  }

  function markSettingsSaved(message?: string) {
    resetSettingsArmed = false;
    status = message ?? settings.savedMessage;
  }

  function selectPrompt(prompt: Prompt, index: number) {
    selectedIndex = index;
    selectedId = prompt.id;
  }

  function togglePreviewPeek() {
    if (!selectedPrompt || !showPreviewPane) return;
    previewPeekOpen = !previewPeekOpen;
    applyWindowMode(previewPeekOpen ? "peek" : "launcher", true);
    status = previewPeekOpen ? translate("previewOpened", { title: selectedPrompt.title }) : translate("previewClosed");
  }

  function closePreviewPeek() {
    previewPeekOpen = false;
    applyWindowMode("launcher", true);
    status = translate("previewClosed");
  }

  async function copySelected() {
    if (!selectedPrompt) return;
    await copyPromptById(selectedPrompt.id);
  }

  async function pasteSelected() {
    if (!selectedPrompt) return;
    await pastePromptById(selectedPrompt.id);
  }

  async function pastePromptById(id: string) {
    try {
      const prompt = await pastePrompt(id);
      patchPrompt(prompt);
      status = translate("pastedPrompt", { title: prompt.title });
      return prompt;
    } catch (error) {
      status = translateError(error, "pastePromptFailed");
      return null;
    }
  }

  async function copyPromptById(id: string) {
    try {
      const prompt = await copyPrompt(id);
      patchPrompt(prompt);
      status = translate("copiedPrompt", { title: prompt.title });
      return prompt;
    } catch (error) {
      status = translateError(error, "copyPromptFailed");
      return null;
    }
  }

  async function quickUsePrompt(prompt: Prompt, index: number) {
    selectedIndex = index;
    selectedId = prompt.id;
    const usedPrompt = autoPasteAfterSelection
      ? await pastePromptById(prompt.id)
      : await copyPromptById(prompt.id);
    if (usedPrompt) await hideLauncher().catch(() => undefined);
  }

  function patchPrompt(prompt: Prompt) {
    prompts = prompts.map((item) => (item.id === prompt.id ? prompt : item));
    selectedId = prompt.id;
  }

  function openLibrary() {
    if (selectedPrompt) {
      startEdit(selectedPrompt);
    } else {
      startNewPrompt();
    }
  }

  function startNewPrompt() {
    previewPeekOpen = false;
    view = "library";
    editingId = null;
    draft = { ...emptyDraft };
    tagText = "";
  }

  function startEdit(prompt: Prompt) {
    previewPeekOpen = false;
    view = "library";
    selectedId = prompt.id;
    editingId = prompt.id;
    draft = {
      title: prompt.title,
      body: prompt.body,
      alias: prompt.alias ?? "",
      notes: prompt.notes ?? "",
      isFavorite: prompt.isFavorite,
      tags: prompt.tags.map((tag) => tag.name)
    };
    tagText = draft.tags.join(", ");
  }

  async function saveDraft() {
    if (!draft.title.trim() || !draft.body.trim()) {
      status = translate("titleAndBodyRequired");
      return;
    }

    saving = true;
    const input = {
      ...draft,
      title: draft.title.trim(),
      body: draft.body.trim(),
      alias: draft.alias?.trim() || null,
      notes: draft.notes?.trim() || null,
      tags: tagText
        .split(",")
        .map((tag) => tag.trim())
        .filter(Boolean)
    };

    try {
      const prompt = editingId
        ? await updatePrompt({ ...input, id: editingId })
        : await createPrompt(input);
      await refresh();
      selectedId = prompt.id;
      status = translate(editingId ? "updatedPrompt" : "createdPrompt", { title: prompt.title });
      editingId = prompt.id;
    } catch (error) {
      status = translateError(error, "savePromptFailed");
    } finally {
      saving = false;
    }
  }

  async function removePrompt(prompt: Prompt) {
    try {
      await deletePrompt(prompt.id);
      prompts = prompts.filter((item) => item.id !== prompt.id);
      selectedId = prompts[0]?.id ?? null;
      status = translate("deletedPrompt", { title: prompt.title });
      startNewPrompt();
    } catch (error) {
      status = translateError(error, "savePromptFailed");
    }
  }

  async function setThemeChoice(next: ThemeChoice) {
    themeChoice = next;
    await persistSetting("theme", next);
  }

  async function setDefaultViewChoice(next: DefaultViewChoice) {
    defaultViewChoice = next;
    await persistSetting("defaultView", next);
  }

  async function setDensityChoice(next: DensityChoice) {
    densityChoice = next;
    await persistSetting("density", next);
  }

  async function toggleLaunchAtLogin() {
    const next = !launchAtLogin;
    const previous = launchAtLogin;
    launchAtLogin = next;
    try {
      launchAtLogin = await setLaunchAtLogin(next);
      await refreshEnvironment();
      markSettingsSaved();
    } catch (error) {
      launchAtLogin = previous;
      status = error instanceof Error ? error.message : settings.savedMessage;
    }
  }

  async function toggleBooleanSetting(key: keyof Pick<
    AppSettings,
    | "autoPasteAfterSelection"
    | "localBackup"
    | "showPreviewPane"
    | "showUsageCount"
    | "showTagsInLauncher"
    | "developerMode"
  >) {
    const next = !currentSettings()[key];
    if (key === "autoPasteAfterSelection") autoPasteAfterSelection = next;
    if (key === "localBackup") localBackup = next;
    if (key === "showPreviewPane") showPreviewPane = next;
    if (key === "showUsageCount") showUsageCount = next;
    if (key === "showTagsInLauncher") showTagsInLauncher = next;
    if (key === "developerMode") developerMode = next;
    await persistSetting(key, String(next));
  }

  async function openTarget(target: "data" | "logs" | "exports" | "backups" | "accessibility") {
    try {
      await openSettingsTarget(target);
      markSettingsSaved(
        target === "accessibility"
          ? settings.statuses.openSystemSettingsRequested
          : settings.statuses.openFolderRequested
      );
    } catch (error) {
      status = error instanceof Error ? error.message : settings.savedMessage;
    }
  }

  async function runTestPaste() {
    try {
      await testPasteAutomation();
      markSettingsSaved(settings.statuses.testPasteQueued);
    } catch (error) {
      status = translateError(error, "pastePromptFailed");
    }
  }

  function jsonExportPayload() {
    return JSON.stringify(
      {
        exportedAt: new Date().toISOString(),
        app: "PromptPop",
        prompts,
        tags
      },
      null,
      2
    );
  }

  function markdownExportPayload() {
    return prompts
      .map((prompt) => {
        const tagsLine = prompt.tags.map((tag) => tag.name).join(", ");
        return [
          `# ${prompt.title}`,
          "",
          `Alias: ${prompt.alias ?? ""}`,
          `Favorite: ${prompt.isFavorite ? "yes" : "no"}`,
          `Tags: ${tagsLine}`,
          "",
          prompt.body,
          "",
          prompt.notes ? `Notes: ${prompt.notes}` : ""
        ]
          .filter((line, index, source) => line || source[index - 1] !== "")
          .join("\n");
      })
      .join("\n\n---\n\n");
  }

  async function exportPrompts(format: "json" | "markdown") {
    try {
      const contents = format === "json" ? jsonExportPayload() : markdownExportPayload();
      const saved = await saveExport(format, contents);
      lastExportPath = saved.path;
      await refreshEnvironment();
      markSettingsSaved(
        format === "json"
          ? settings.statuses.jsonExportRequested
          : settings.statuses.markdownExportRequested
      );
    } catch (error) {
      status = error instanceof Error ? error.message : settings.savedMessage;
    }
  }

  async function runBackup() {
    try {
      const saved: SavedFile = await backupDatabase();
      lastBackupAt = new Date().toISOString();
      lastExportPath = saved.path;
      await refreshEnvironment();
      markSettingsSaved(settings.statuses.backupRequested);
    } catch (error) {
      status = error instanceof Error ? error.message : settings.savedMessage;
    }
  }

  async function restoreDefaultSnippets() {
    try {
      if (localBackup) await runBackup();
      const restoredCount = await restoreStarterSnippets(locale);
      await refresh();
      markSettingsSaved(
        restoredCount > 0
          ? `${settings.statuses.starterSnippetsRestored}: ${restoredCount}`
          : settings.statuses.starterSnippetsAlreadyPresent
      );
    } catch (error) {
      status = error instanceof Error ? error.message : settings.savedMessage;
    }
  }

  function openImportPicker() {
    importInput?.click();
  }

  async function importPromptFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    try {
      if (localBackup) await runBackup();
      const content = await file.text();
      const imported = parseImportFile(content, file.name);
      for (const prompt of imported) {
        await createPrompt(prompt);
      }
      await refresh();
      markSettingsSaved(settings.statuses.importRequested);
    } catch (error) {
      status = error instanceof Error ? error.message : settings.savedMessage;
    } finally {
      input.value = "";
    }
  }

  function parseImportFile(content: string, filename: string): PromptInput[] {
    if (filename.toLowerCase().endsWith(".json")) {
      const parsed = JSON.parse(content);
      const source = Array.isArray(parsed) ? parsed : parsed.prompts;
      if (!Array.isArray(source)) throw new Error("JSON export must contain a prompts array");
      return source.map((item: Partial<Prompt>) => ({
        title: String(item.title ?? "").trim(),
        body: String(item.body ?? "").trim(),
        alias: item.alias ?? null,
        notes: item.notes ?? null,
        isFavorite: Boolean(item.isFavorite),
        tags: Array.isArray(item.tags)
          ? item.tags.map((tag) => (typeof tag === "string" ? tag : tag.name)).filter(Boolean)
          : []
      })).filter((item) => item.title && item.body);
    }

    return content
      .split(/\n---\n/g)
      .map((block) => {
        const title = block.match(/^#\\s+(.+)$/m)?.[1]?.trim() ?? "";
        const alias = block.match(/^Alias:\\s*(.*)$/m)?.[1]?.trim() ?? "";
        const tagLine = block.match(/^Tags:\\s*(.*)$/m)?.[1]?.trim() ?? "";
        const body = block
          .replace(/^#\\s+.+$/m, "")
          .replace(/^Alias:.*$/m, "")
          .replace(/^Favorite:.*$/m, "")
          .replace(/^Tags:.*$/m, "")
          .replace(/^Notes:.*$/m, "")
          .trim();
        return {
          title,
          body,
          alias: alias || null,
          notes: null,
          isFavorite: false,
          tags: tagLine.split(",").map((tag) => tag.trim()).filter(Boolean)
        };
      })
      .filter((item) => item.title && item.body);
  }

  async function copyDiagnostics() {
    const diagnostics = {
      appVersion: appEnvironment?.appVersion ?? "0.1.0",
      tauriVersion: appEnvironment?.tauriVersion ?? "unknown",
      databasePath: appEnvironment?.databasePath ?? "unknown",
      prompts: prompts.length,
      tags: tags.length,
      settings: currentSettings()
    };

    try {
      await navigator.clipboard.writeText(JSON.stringify(diagnostics, null, 2));
      markSettingsSaved(settings.statuses.diagnosticsCopied);
    } catch {
      status = translate("clipboardPermissionDenied");
    }
  }

  async function resetSettings() {
    if (!resetSettingsArmed) {
      resetSettingsArmed = true;
      status = settings.statuses.resetConfirmationRequired;
      return;
    }

    applySettingsState({ ...defaultSettings, locale });
    saveLocale(locale);
    await persistAllSettings(settings.savedMessage);
    await registerLauncherShortcut(defaultSettings.shortcuts.globalLauncher).catch(() => undefined);
    await setLaunchAtLogin(defaultSettings.launchAtLogin).catch(() => undefined);
    await refreshEnvironment();
    resetSettingsArmed = false;
  }

  function formatDate(value: string | null, activeLocale: Locale) {
    if (!value) return t(activeLocale, "never");
    return new Intl.DateTimeFormat(activeLocale, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit"
    }).format(new Date(value));
  }
</script>

<main
  class:library-mode={view === "library"}
  class:settings-mode={view === "settings"}
  class:comfortable-density={densityChoice === "comfortable"}
  class:developer-mode={developerMode}
>
  {#if view === "launcher"}
    <section class="window launcher-window" class:peek-mode={previewPeekOpen && !!selectedPrompt} aria-label={text.appAriaPromptLauncher}>
      <div class="launcher-stage">
        <section class="launcher-pane">
          <header class="launcher-topbar">
            <label class="search-box">
              <span class="search-glyph">S</span>
              <input
                bind:this={searchInput}
                bind:value={query}
                type="search"
                placeholder={text.searchPlaceholder}
                autocomplete="off"
                on:input={() => {
                  selectedIndex = 0;
                  previewPeekOpen = false;
                }}
              />
            </label>
            <span class="local-pill"><span class="status-dot"></span>{text.localOnly}</span>
            <button class="launcher-icon-button" type="button" title={text.library} aria-label={text.library} on:click={openLibrary}>
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4.5 19.5h15" />
                <path d="M6 4.5h3.5v15H6z" />
                <path d="M11 4.5h3.5v15H11z" />
                <path d="M16.2 5.2l2.8-.7 3.4 14.2-2.8.7z" />
              </svg>
            </button>
            <button class="launcher-icon-button" type="button" title={settingsButtonLabel} aria-label={settingsButtonLabel} on:click={() => openSettings()}>
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M12 15.5A3.5 3.5 0 1 0 12 8a3.5 3.5 0 0 0 0 7.5z" />
                <path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 0 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-1.8-.3 1.6 1.6 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.2a1.6 1.6 0 0 0-1-1.5 1.6 1.6 0 0 0-1.8.3l-.1.1a2 2 0 0 1-2.8-2.8l.1-.1A1.6 1.6 0 0 0 4.6 15a1.6 1.6 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.2a1.6 1.6 0 0 0 1.5-1 1.6 1.6 0 0 0-.3-1.8l-.1-.1A2 2 0 0 1 7.1 4.3l.1.1A1.6 1.6 0 0 0 9 4.7a1.6 1.6 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.2a1.6 1.6 0 0 0 1 1.5 1.6 1.6 0 0 0 1.8-.3l.1-.1a2 2 0 0 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8 1.6 1.6 0 0 0 1.5 1h.2a2 2 0 0 1 0 4h-.2a1.6 1.6 0 0 0-1.4 1z" />
              </svg>
            </button>
          </header>

          <nav class="filter-row compact" aria-label={text.filters}>
            <button class:active={filter === "all"} type="button" on:click={() => setFilter("all")}>
              {text.all}
            </button>
            <button class:active={filter === "favorites"} type="button" on:click={() => setFilter("favorites")}>
              {text.favorites}
            </button>
            <button class:active={filter === "recent"} type="button" on:click={() => setFilter("recent")}>
              {text.recent}
            </button>
            <button class="filter-plus" type="button" title={text.newPrompt} aria-label={text.newPrompt} on:click={startNewPrompt}>+</button>
          </nav>

          {#if loading}
            <div class="empty-panel compact">{text.loadingPrompts}</div>
          {:else if filteredPrompts.length === 0}
            <div class="empty-panel">
              <span class="empty-badge">{text.noPromptsFound}</span>
              <h2>{query ? `${text.noPromptsFound}: ${query}` : text.noPromptsFound}</h2>
              <p>{text.noPromptsHint}</p>
              <div class="empty-actions">
                <button class="primary-button" type="button" on:click={startNewPrompt}>{text.newPrompt}</button>
                <button class="quiet-button" type="button" on:click={() => { query = ""; filter = "all"; }}>
                  {text.close}
                </button>
              </div>
            </div>
          {:else}
            <ol class="results-list compact" aria-label={text.appAriaPromptResults}>
              {#each filteredPrompts as prompt, index}
                <li>
                  <button
                    type="button"
                    class:selected={index === selectedIndex}
                    on:click={() => selectPrompt(prompt, index)}
                    on:dblclick={() => copyPromptById(prompt.id)}
                  >
                    <span class="result-copy">
                      <span class="result-title">
                        <strong>{prompt.title}</strong>
                      </span>
                      <span class="result-snippet"><span>{text.body}:</span> {promptSnippet(prompt)}</span>
                      <span class="result-meta">{promptListMeta(prompt)}</span>
                    </span>
                    <span class="result-side">
                      {#if index === selectedIndex && showUsageCount && prompt.lastUsedAt}
                        <small>{formatDate(prompt.lastUsedAt, locale)}</small>
                      {/if}
                      <kbd>{shortcutText(resultJumpShortcut(shortcutBindings.jumpResult, index))}</kbd>
                    </span>
                  </button>
                </li>
              {/each}
            </ol>
          {/if}

          <footer class="commandbar">
            <span><span class="status-dot"></span>{searchSummary}</span>
            <button class="command-action" type="button" on:click={copySelected} disabled={!selectedPrompt}>
              <kbd>{shortcutText(shortcutBindings.copySelected)}</kbd>{text.copyShortcut}
            </button>
            <button class="command-action" type="button" on:click={togglePreviewPeek} disabled={!selectedPrompt || !showPreviewPane} aria-pressed={previewPeekOpen}>
              <kbd>Space</kbd>{text.previewShortcut}
            </button>
            <button class="command-action" type="button" on:click={pasteSelected} disabled={!selectedPrompt}>
              <kbd>{shortcutText(shortcutBindings.pasteSelected)}</kbd>{text.pasteShortcut}
            </button>
          </footer>
        </section>

        {#if previewPeekOpen && selectedPrompt}
          <aside class="preview-peek" aria-label={text.appAriaSelectedPreview}>
            <header class="peek-header">
              <strong><kbd>Space</kbd>{text.preview}</strong>
              <button class="peek-close" type="button" on:click={closePreviewPeek}>Esc {text.close}</button>
            </header>
            <section class="peek-title">
              <h2>{selectedPrompt.title}</h2>
              <p>{promptMeta(selectedPrompt)}</p>
            </section>
            <section class="prompt-card peek-body">
              <div class="card-title">
                <strong>{text.previewBody}</strong>
                <span>{selectedPrompt.body.length} chars</span>
              </div>
              <pre>{selectedPrompt.body}</pre>
            </section>
            <section class="variables-panel compact">
              <strong>{selectedVariables.length ? "VARIABLES" : text.notes}</strong>
              {#if selectedVariables.length}
                <div class="variable-list">
                  {#each selectedVariables as variable}
                    <code>{"{{"}{variable}{"}}"}</code>
                  {/each}
                </div>
              {:else}
                <p>{selectedPrompt.notes || text.noMetadata}</p>
              {/if}
            </section>
            <footer class="peek-actions">
              <button class="primary-button" type="button" on:click={copySelected}>{shortcutText(shortcutBindings.copySelected)} {text.copyShortcut}</button>
              <button class="quiet-button" type="button" on:click={pasteSelected}>{shortcutText(shortcutBindings.pasteSelected)} {text.pasteShortcut}</button>
            </footer>
          </aside>
        {/if}
      </div>
    </section>
  {:else if view === "settings"}
    <section class="settings-screen" aria-label={settings.appAria}>
      <aside class="settings-sidebar">
        <div class="settings-brand">
          <h1>{settings.sidebarTitle}</h1>
          <p>{settings.sidebarSubtitle}</p>
        </div>

        <label class="settings-search">
          <span>S</span>
          <input bind:value={settingsQuery} type="search" placeholder={settings.searchPlaceholder} />
        </label>

        <nav class="settings-nav" aria-label={settings.navAria}>
          {#each settingsSections as item}
            <button
              type="button"
              class:active={activeSettingsSection === item.id}
              on:click={() => (activeSettingsSection = item.id)}
            >
              <span class="settings-rail"></span>
              <span class="settings-icon">{item.icon}</span>
              <span>{item.label}</span>
            </button>
          {/each}
        </nav>

        <div class="settings-spacer"></div>

        <div class="settings-status-card">
          <span class="status-dot"></span>
          <div>
            <strong>{settingsSavedTitle}</strong>
            <span>{settingsSavedHint}</span>
          </div>
        </div>
      </aside>

      <section class="settings-content">
        {#if activeSettingsSection === "general"}
          <header class="settings-page-header">
            <div>
              <h2>{settings.general.heading}</h2>
              <p>{settings.general.description}</p>
            </div>
            <span class="save-pill">{settings.savePill}</span>
          </header>

          <div class="settings-layout">
            <div class="settings-list">
              <article class="settings-row">
                <div>
                  <strong>{settings.general.languageTitle}</strong>
                  <span>{settings.general.languageDescription}</span>
                </div>
                <div class="segmented-control">
                  <button class:active={locale === "en"} type="button" on:click={() => setLocale("en")}>{settings.options.english}</button>
                  <button class:active={locale === "zh-CN"} type="button" on:click={() => setLocale("zh-CN")}>{settings.options.chinese}</button>
                </div>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.general.themeTitle}</strong>
                  <span>{settings.general.themeDescription}</span>
                </div>
                <div class="segmented-control wide">
                  <button class:active={themeChoice === "system"} type="button" on:click={() => setThemeChoice("system")}>{settings.options.system}</button>
                  <button class:active={themeChoice === "light"} type="button" on:click={() => setThemeChoice("light")}>{settings.options.light}</button>
                  <button class:active={themeChoice === "dark"} type="button" on:click={() => setThemeChoice("dark")}>{settings.options.dark}</button>
                </div>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.general.launchTitle}</strong>
                  <span>{platformLaunchDescription}</span>
                </div>
                <button
                  class="toggle-switch"
                  class:enabled={launchAtLogin}
                  type="button"
                  aria-label="Toggle launch at login"
                  aria-pressed={launchAtLogin}
                  on:click={toggleLaunchAtLogin}
                >
                  <span></span>
                </button>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.general.defaultViewTitle}</strong>
                  <span>{settings.general.defaultViewDescription}</span>
                </div>
                <div class="segmented-control wide">
                  <button class:active={defaultViewChoice === "launcher"} type="button" on:click={() => setDefaultViewChoice("launcher")}>{settings.options.launcher}</button>
                  <button class:active={defaultViewChoice === "library"} type="button" on:click={() => setDefaultViewChoice("library")}>{settings.options.library}</button>
                </div>
              </article>
            </div>

            <aside class="settings-context">
              <article>
                <h3>{settings.general.localFirstTitle}</h3>
                <p>{platformLocalFirstDescription}</p>
              </article>
              <article class="highlight-card">
                <strong>{settings.general.sqliteTitle}</strong>
                <span>{settings.general.sqliteDescription}</span>
              </article>
              <article class="shortcut-card">
                <span>{settings.general.shortcutTitle}</span>
                <strong>
                  {settings.general.launcherShortcut}
                  {#each shortcutLabels(shortcutBindings.globalLauncher) as key}
                    <kbd>{key}</kbd>
                  {/each}
                </strong>
                <strong>
                  {settings.general.focusSearchShortcut}
                  {#each shortcutLabels(shortcutBindings.focusSearch) as key}
                    <kbd>{key}</kbd>
                  {/each}
                </strong>
              </article>
              <article>
                <h3>{platformGeneralPermission.title}</h3>
                <p>{platformGeneralPermission.description}</p>
              </article>
            </aside>
          </div>
        {:else if activeSettingsSection === "shortcuts"}
          <header class="settings-page-header">
            <div>
              <h2>{settings.shortcuts.heading}</h2>
              <p>{settings.shortcuts.description}</p>
            </div>
            <span class="save-pill">{settings.savePill}</span>
          </header>

          {#if recordingShortcutId}
            <div class="shortcut-recorder">
              <div>
                <strong>{settings.shortcuts.recordingPrefix} {settings.shortcuts.rows[recordingShortcutId].title}</strong>
                <span>{shortcutRecordingHint(recordingShortcutId)}</span>
              </div>
              <div class="shortcut-keys">
                {#each recordingShortcutId === "jumpResult" ? shortcutPrefixLabels(shortcutBindings.jumpResult) : shortcutLabels(shortcutBindings[recordingShortcutId]) as key}
                  <kbd>{key}</kbd>
                {/each}
                <span class="recording-dot"></span>
              </div>
            </div>
          {/if}

          {#if shortcutNotice}
            <p class="shortcut-notice">{shortcutNotice}</p>
          {/if}

          <div class="shortcut-table">
            {#each shortcutRows as row}
              <article class="shortcut-row" class:focused={row.active} class:recording={row.recording}>
                <div>
                  <strong>{row.title}</strong>
                  <span>{row.description}</span>
                </div>
                <div class="shortcut-actions">
                  {#each row.keys as key}
                    <kbd>{key}</kbd>
                  {/each}
                  <button class="quiet-button tiny-button" type="button" on:click={() => { recordingShortcutId = row.id; shortcutNotice = ""; status = settings.statuses.shortcutEditReady; }}>{settings.options.edit}</button>
                </div>
                {#if row.conflict}
                  <p class="shortcut-conflict">{settings.shortcuts.conflictPrefix} {row.conflict}. {settings.shortcuts.conflictHint}</p>
                {/if}
              </article>
            {/each}
          </div>
        {:else if activeSettingsSection === "paste"}
          <header class="settings-page-header">
            <div>
              <span class="eyebrow">{platformPasteSettings.eyebrow}</span>
              <h2>{platformPasteSettings.heading}</h2>
              <p>{platformPasteSettings.description}</p>
            </div>
          </header>

          <div class="permission-strip">
            <article>
              <span class="permission-icon">A</span>
              <div>
                <strong>{platformPasteSettings.accessibilityTitle}</strong>
                <span>{platformPasteSettings.accessibilityDescription}</span>
              </div>
              <em class:granted={pasteAutomationReady}>{pasteAutomationReady ? platformPasteSettings.granted : platformPasteSettings.required}</em>
            </article>
            <article>
              <span class="permission-icon success">C</span>
              <div>
                <strong>{platformPasteSettings.clipboardTitle}</strong>
                <span>{platformPasteSettings.clipboardDescription}</span>
              </div>
              <em class="granted">{platformPasteSettings.granted}</em>
            </article>
          </div>

          <section class="settings-list standalone">
            <article class="settings-row">
              <div>
                <strong>{platformPasteSettings.autoPasteTitle}</strong>
                <span>{platformPasteSettings.autoPasteDescription}</span>
              </div>
              <button
                class="toggle-switch"
                class:enabled={autoPasteAfterSelection}
                type="button"
                aria-label="Toggle auto paste after selection"
                aria-pressed={autoPasteAfterSelection}
                on:click={() => toggleBooleanSetting("autoPasteAfterSelection")}
              >
                <span></span>
              </button>
            </article>
            <p class="settings-note">{platformPasteSettings.clipboardNote}</p>
          </section>

          <article class="permission-help">
            <strong>{platformPasteSettings.helpTitle}</strong>
            <span>{platformPasteSettings.helpDescription}</span>
          </article>

          <div class="settings-actions-row">
            {#if isMacOS}
              <button class="primary-button" type="button" on:click={() => openTarget("accessibility")}>{settings.options.openSystemSettings}</button>
            {/if}
            <button class="quiet-button" type="button" on:click={runTestPaste}>{settings.options.testPaste}</button>
            <span>{pasteAutomationStatus}</span>
          </div>
        {:else if activeSettingsSection === "data"}
          <header class="settings-page-header">
            <div>
              <h2>{settings.data.heading}</h2>
              <p>{settings.data.description}</p>
            </div>
            <span class="save-pill warning">{settings.localOnly}</span>
          </header>

          <div class="settings-layout">
            <div class="settings-list">
              <article class="settings-row">
                <div>
                  <strong>{settings.data.databaseTitle}</strong>
                  <span class="path-pill">{appEnvironment?.databasePath ?? "localStorage://PromptPop"}</span>
                </div>
                <button class="primary-button" type="button" on:click={() => openTarget("data")}>{settings.options.openFolder}</button>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.data.exportTitle}</strong>
                  <span>{settings.data.exportDescription}</span>
                </div>
                <div class="settings-inline-actions">
                  <button class="quiet-button" type="button" on:click={() => exportPrompts("json")}>JSON</button>
                  <button class="quiet-button" type="button" on:click={() => exportPrompts("markdown")}>Markdown</button>
                </div>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.data.importTitle}</strong>
                  <span>{settings.data.importDescription}</span>
                </div>
                <button class="quiet-button" type="button" on:click={openImportPicker}>{settings.options.import}</button>
                <input
                  bind:this={importInput}
                  class="visually-hidden"
                  type="file"
                  accept=".json,.md,.markdown,application/json,text/markdown,text/plain"
                  on:change={importPromptFile}
                />
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.data.starterTitle}</strong>
                  <span>{settings.data.starterDescription}</span>
                </div>
                <button class="quiet-button" type="button" on:click={restoreDefaultSnippets}>{settings.options.restoreSnippets}</button>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.data.backupTitle}</strong>
                  <span>{settings.data.backupDescription}</span>
                </div>
                <div class="settings-inline-actions">
                  <button
                    class="toggle-switch"
                    class:enabled={localBackup}
                    type="button"
                    aria-label="Toggle local backup"
                    aria-pressed={localBackup}
                    on:click={() => toggleBooleanSetting("localBackup")}
                  >
                    <span></span>
                  </button>
                  <button class="primary-button" type="button" on:click={runBackup}>{settings.options.backUpNow}</button>
                </div>
              </article>
            </div>

            <aside class="settings-context">
              <article>
                <h3>{settings.data.storageTitle}</h3>
                <p>{settings.data.storageDescription}</p>
              </article>
              <article class="metric-card">
                <span>{settings.data.prompts}</span>
                <strong>{prompts.length}</strong>
                <span>{settings.data.tags}</span>
                <strong>{tags.length}</strong>
                <span>{settings.data.lastBackup}</span>
                <strong>{formatDate(lastBackupAt, locale)}</strong>
              </article>
              <article>
                <h3>{settings.data.readyTitle}</h3>
                <p>{lastExportPath ?? settings.data.readyDescription}</p>
              </article>
              <article>
                <h3>{settings.data.backupEnabledTitle}</h3>
                <p>{settings.data.backupEnabledDescription}</p>
              </article>
            </aside>
          </div>
        {:else if activeSettingsSection === "appearance"}
          <header class="settings-page-header">
            <div>
              <h2>{settings.appearance.heading}</h2>
              <p>{settings.appearance.description}</p>
            </div>
            <span class="save-pill">{settings.savePill}</span>
          </header>

          <div class="settings-layout">
            <div class="settings-list">
              <article class="settings-row">
                <div>
                  <strong>{settings.appearance.densityTitle}</strong>
                  <span>{settings.appearance.densityDescription}</span>
                </div>
                <div class="segmented-control wide">
                  <button class:active={densityChoice === "compact"} type="button" on:click={() => setDensityChoice("compact")}>{settings.options.compact}</button>
                  <button class:active={densityChoice === "comfortable"} type="button" on:click={() => setDensityChoice("comfortable")}>{settings.options.comfortable}</button>
                </div>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.appearance.previewTitle}</strong>
                  <span>{settings.appearance.previewDescription}</span>
                </div>
                <button
                  class="toggle-switch"
                  class:enabled={showPreviewPane}
                  type="button"
                  aria-label="Toggle preview pane"
                  aria-pressed={showPreviewPane}
                  on:click={() => toggleBooleanSetting("showPreviewPane")}
                >
                  <span></span>
                </button>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.appearance.usageTitle}</strong>
                  <span>{settings.appearance.usageDescription}</span>
                </div>
                <button
                  class="toggle-switch"
                  class:enabled={showUsageCount}
                  type="button"
                  aria-label="Toggle usage count"
                  aria-pressed={showUsageCount}
                  on:click={() => toggleBooleanSetting("showUsageCount")}
                >
                  <span></span>
                </button>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.appearance.tagsTitle}</strong>
                  <span>{settings.appearance.tagsDescription}</span>
                </div>
                <button
                  class="toggle-switch"
                  class:enabled={showTagsInLauncher}
                  type="button"
                  aria-label="Toggle launcher tags"
                  aria-pressed={showTagsInLauncher}
                  on:click={() => toggleBooleanSetting("showTagsInLauncher")}
                >
                  <span></span>
                </button>
              </article>

              <article class="settings-row tall-row">
                <div>
                  <strong>{settings.appearance.uiFontTitle}</strong>
                  <span>{settings.appearance.uiFontDescription}</span>
                </div>
                <div class="font-preview">
                  <strong>Geist</strong>
                  <span>{settings.appearance.fontPreviewLabel}</span>
                </div>
              </article>

              <article class="settings-row tall-row">
                <div>
                  <strong>{settings.appearance.promptFontTitle}</strong>
                  <span>{settings.appearance.promptFontDescription}</span>
                </div>
                <div class="font-preview mono">
                  <strong>{settings.appearance.promptPreviewSample}</strong>
                  <span>{settings.appearance.promptPreviewNote}</span>
                </div>
              </article>
            </div>

            <aside class="settings-context">
              <article>
                <h3>{settings.appearance.behaviorTitle}</h3>
                <p>{settings.appearance.behaviorDescription}</p>
              </article>
              <article>
                <h3>{settings.appearance.themeDirectionTitle}</h3>
                <div class="theme-preview">
                  <span class="theme-swatch light">{settings.options.light}</span>
                  <span class="theme-swatch dark">{settings.options.dark}</span>
                </div>
                <p>{settings.appearance.themeDirectionDescription}</p>
              </article>
              <article class="highlight-card">
                <h3>{settings.appearance.narrowTitle}</h3>
                <p>{settings.appearance.narrowDescription}</p>
              </article>
              <button class="quiet-button full-width" type="button" on:click={() => persistAllSettings(settings.statuses.appearanceApplied)}>{settings.options.applyAppearance}</button>
            </aside>
          </div>
        {:else}
          <header class="settings-page-header">
            <div>
              <h2>{settings.advanced.heading}</h2>
              <p>{settings.advanced.description}</p>
            </div>
            <span class="save-pill muted">{settings.advanced.appPill}</span>
          </header>

          <div class="settings-layout">
            <div class="settings-list">
              <article class="settings-row">
                <div>
                  <strong>{settings.advanced.developerTitle}</strong>
                  <span>{settings.advanced.developerDescription}</span>
                </div>
                <button
                  class="toggle-switch"
                  class:enabled={developerMode}
                  type="button"
                  aria-label="Toggle developer mode"
                  aria-pressed={developerMode}
                  on:click={() => toggleBooleanSetting("developerMode")}
                >
                  <span></span>
                </button>
              </article>

              <article class="settings-row">
                <div>
                  <strong>{settings.advanced.logsTitle}</strong>
                  <span>{settings.advanced.logsDescription}</span>
                </div>
                <button class="quiet-button" type="button" on:click={() => openTarget("logs")}>{settings.options.open}</button>
              </article>

              <article class="diagnostics-card">
                <div>
                  <strong>{settings.advanced.diagnosticsTitle}</strong>
                  <span>{settings.advanced.appVersion}</span>
                  <span>{settings.advanced.databaseRevision}</span>
                  <span>{settings.advanced.tauriVersion}</span>
                  <span>{settings.advanced.osPermission}</span>
                </div>
                <div>
                  <button class="quiet-button tiny-button" type="button" on:click={copyDiagnostics}>{settings.options.copyReport}</button>
                  <strong>{appEnvironment?.appVersion ?? "0.1.0"}</strong>
                  <strong>{prompts.length + tags.length}</strong>
                  <strong>{appEnvironment?.tauriVersion ?? "browser"}</strong>
                  <strong class:danger-text={!pasteAutomationReady}>{pasteAutomationReady ? platformPasteSettings.granted : settings.advanced.pending}</strong>
                </div>
              </article>

              <article class="settings-row danger-row">
                <div>
                  <strong>{settings.advanced.resetTitle}</strong>
                  <span>{settings.advanced.resetDescription}</span>
                </div>
                <button class="danger-button" type="button" on:click={resetSettings}>{settings.options.reset}</button>
              </article>

              <div class="danger-confirmation">{settings.advanced.dangerConfirmation}</div>
            </div>

            <aside class="settings-context">
              <article>
                <h3>{settings.advanced.principleTitle}</h3>
                <p>{settings.advanced.principleDescription}</p>
              </article>
              <article class="metric-card">
                <span>{settings.advanced.systemHealth}</span>
                <strong>OK</strong>
                <span>{settings.advanced.database}</span>
                <strong>{settings.advanced.active}</strong>
                <span>{settings.advanced.shortcutHook}</span>
                <strong>{shortcutLabels(shortcutBindings.globalLauncher).join(" ")}</strong>
              </article>
              <article>
                <h3>{settings.advanced.logs}</h3>
                <p>{appEnvironment?.logsDir ?? "~/Library/Logs/PromptPop"}</p>
              </article>
              <article class="highlight-card">
                <strong>{settings.advanced.noRestartTitle}</strong>
                <span>{settings.advanced.noRestartDescription}</span>
              </article>
            </aside>
          </div>
        {/if}
      </section>
    </section>
  {:else}
    <section class="library-screen" aria-label={text.appAriaPromptLibrary}>
      <header class="library-header">
        <div>
          <h1>PromptPop {text.library}</h1>
          <p>{text.appAriaPromptLibrary}</p>
        </div>
        <div class="library-actions">
          <span class="db-pill">SQLite local / {prompts.length}</span>
          <button class="quiet-button" type="button" on:click={showLauncherView}>{text.launcher}</button>
          <button class="quiet-button" type="button" on:click={() => openSettings()}>{settingsButtonLabel}</button>
          <button class="primary-button" type="button" on:click={startNewPrompt}>+ {text.newPrompt}</button>
        </div>
      </header>

      <div class="library-workspace">
        <aside class="library-list">
          <div class="section-title">
            <div>
              <h2>{text.appAriaPromptResults}</h2>
              <span>{prompts.length} total</span>
            </div>
          </div>

          <label class="library-search">
            <span>S</span>
            <input bind:value={query} placeholder={text.searchPlaceholder} />
          </label>

          <nav class="filter-row compact" aria-label={text.filters}>
            <button class:active={filter === "all"} type="button" on:click={() => setFilter("all")}>{text.all}</button>
            <button class:active={filter === "favorites"} type="button" on:click={() => setFilter("favorites")}>{text.favorites}</button>
            <button class:active={filter === "recent"} type="button" on:click={() => setFilter("recent")}>{text.recent}</button>
          </nav>

          <div class="library-rows">
            {#if filteredPrompts.length === 0}
              <div class="empty-list">{text.noPromptsHint}</div>
            {:else}
              {#each filteredPrompts as prompt, index}
                <button
                  type="button"
                  class:active={editingId === prompt.id}
                  on:click={() => { selectPrompt(prompt, index); startEdit(prompt); }}
                >
                  <strong>{prompt.title}</strong>
                  <span>{promptMeta(prompt)}</span>
                  <small>{prompt.tags.map((tag) => `#${tag.name}`).join(" ")}</small>
                </button>
              {/each}
            {/if}
          </div>
        </aside>

        <form class="editor-panel" on:submit|preventDefault={saveDraft}>
          <header class="editor-header">
            <div>
              <h2>{editingId ? text.editPrompt : text.newPrompt}</h2>
              <p>{editingId && libraryPrompt ? promptMeta(libraryPrompt) : text.noMetadata}</p>
            </div>
            <div class="editor-actions">
              {#if editingId && libraryPrompt}
                <button class="quiet-button" type="button" on:click={copySelected}>{text.copy}</button>
                <button class="danger-button" type="button" on:click={() => removePrompt(libraryPrompt)}>{text.delete}</button>
              {/if}
              <button class="primary-button" disabled={saving} type="submit">{saving ? text.saving : text.savePrompt}</button>
            </div>
          </header>

          <div class="feedback-row">
            <span><span class="status-dot"></span>{status}</span>
            <span>{text.tags}: {tagText || text.none}</span>
          </div>

          <div class="field-grid">
            <label>
              {text.title}
              <input bind:value={draft.title} placeholder={text.titlePlaceholder} />
            </label>

            <label>
              {text.alias}
              <input bind:value={draft.alias} placeholder={text.aliasPlaceholder} />
            </label>
          </div>

          <label>
            {text.body}
            <textarea bind:value={draft.body} rows="11" placeholder={text.bodyPlaceholder}></textarea>
          </label>

          <label>
            {text.tags}
            <input bind:value={tagText} placeholder={text.tagsPlaceholder} list="known-tags" />
            <datalist id="known-tags">
              {#each tags as tag}
                <option value={tag.name}></option>
              {/each}
            </datalist>
          </label>

          <label>
            {text.notes}
            <textarea bind:value={draft.notes} rows="3" placeholder={text.notesPlaceholder}></textarea>
          </label>

          <div class="editor-options">
            <label class="toggle-card">
              <input bind:checked={draft.isFavorite} type="checkbox" />
              <span>{text.favoritePrompt}</span>
            </label>
            <div class="metadata-card">
              <strong>Local SQLite metadata</strong>
              <span>id: {editingId || "new"}</span>
              <span>{editingId && libraryPrompt ? `updated: ${formatDate(libraryPrompt.updatedAt, locale)}` : text.noMetadata}</span>
              <span>{editingId && libraryPrompt ? `last: ${formatDate(libraryPrompt.lastUsedAt, locale)}` : text.never}</span>
            </div>
          </div>
        </form>
      </div>

      <footer class="library-footer">
        <span>{text.noPromptsHint}</span>
        <span><span class="status-dot"></span>{status}</span>
      </footer>
    </section>
  {/if}
</main>
