import type { ShortcutBindings, ShortcutId } from "./types";

export type DesktopPlatform = "macos" | "windows" | "linux";

const shortcutIds: ShortcutId[] = [
  "globalLauncher",
  "focusSearch",
  "copySelected",
  "pasteSelected",
  "editSelected",
  "jumpResult"
];

const macShortcutDefaults: ShortcutBindings = {
  globalLauncher: "Alt+Space",
  focusSearch: "Meta+K",
  copySelected: "Enter",
  pasteSelected: "Meta+Enter",
  editSelected: "Meta+KeyE",
  jumpResult: "Meta"
};

const windowsShortcutDefaults: ShortcutBindings = {
  globalLauncher: "Alt+Space",
  focusSearch: "Control+K",
  copySelected: "Enter",
  pasteSelected: "Control+Enter",
  editSelected: "Control+KeyE",
  jumpResult: "Control"
};

export function detectDesktopPlatform(): DesktopPlatform {
  if (typeof navigator === "undefined") return "linux";

  const nav = navigator as Navigator & { userAgentData?: { platform?: string } };
  const platform = `${nav.userAgentData?.platform ?? ""} ${nav.platform ?? ""}`.toLowerCase();

  if (platform.includes("mac")) return "macos";
  if (platform.includes("win")) return "windows";
  return "linux";
}

export function defaultShortcutBindings(platform: DesktopPlatform = detectDesktopPlatform()): ShortcutBindings {
  return platform === "macos" ? { ...macShortcutDefaults } : { ...windowsShortcutDefaults };
}

export function migrateShortcutBindingsForPlatform(
  bindings: ShortcutBindings,
  platform: DesktopPlatform = detectDesktopPlatform()
): ShortcutBindings {
  const defaults = defaultShortcutBindings(platform);
  if (platform === "macos") return { ...bindings };

  const migrated = { ...bindings };
  for (const id of shortcutIds) {
    if (migrated[id] === macShortcutDefaults[id]) {
      migrated[id] = defaults[id];
    }
  }
  return migrated;
}

export function isShortcutModifier(part: string): boolean {
  return ["Meta", "Control", "Alt", "Shift"].includes(part);
}

export function shortcutModifierExample(platform: DesktopPlatform = detectDesktopPlatform()): string {
  return platform === "macos" ? "Cmd" : "Ctrl";
}

export function shortcutKeyLabel(part: string, platform: DesktopPlatform = detectDesktopPlatform()): string {
  if (part === "Meta") return platform === "macos" ? "Cmd" : "Win";
  if (part === "Alt") return platform === "macos" ? "Option" : "Alt";
  if (part === "Control") return "Ctrl";
  if (part.startsWith("Key")) return part.replace("Key", "");
  if (part.startsWith("Digit")) return part.replace("Digit", "");
  return part;
}
