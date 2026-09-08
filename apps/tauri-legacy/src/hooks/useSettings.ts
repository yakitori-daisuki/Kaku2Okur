import { useEffect, useMemo, useState } from "react";
import type { AppSettings } from "../domain/types";

const STORAGE_KEY = "kaku2okur.settings.v1";

const DEFAULT_SETTINGS: AppSettings = {
  backgroundMode: "system",
  customBackground: "#ffffff",
  sendBackgroundMode: "match",
  customSendBackground: "#ffffff",
  invocationGesture: "dual-modifier",
  trackpadDefault: false,
};

function isDarkColor(color: string): boolean {
  const value = color.replace("#", "");
  if (!/^[0-9a-f]{6}$/i.test(value)) return false;
  const red = Number.parseInt(value.slice(0, 2), 16);
  const green = Number.parseInt(value.slice(2, 4), 16);
  const blue = Number.parseInt(value.slice(4, 6), 16);
  return (red * 0.2126 + green * 0.7152 + blue * 0.0722) / 255 < 0.46;
}

function loadSettings(): AppSettings {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? { ...DEFAULT_SETTINGS, ...JSON.parse(stored) } : DEFAULT_SETTINGS;
  } catch {
    return DEFAULT_SETTINGS;
  }
}

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(loadSettings);
  const [systemDark, setSystemDark] = useState(
    () => window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false,
  );

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = (event: MediaQueryListEvent) => setSystemDark(event.matches);
    media.addEventListener("change", handleChange);
    return () => media.removeEventListener("change", handleChange);
  }, []);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }, [settings]);

  const canvasBackground = useMemo(() => {
    switch (settings.backgroundMode) {
      case "white":
        return "#ffffff";
      case "black":
        return "#000000";
      case "custom":
        return settings.customBackground;
      case "system":
        return systemDark ? "#111315" : "#ffffff";
    }
  }, [settings.backgroundMode, settings.customBackground, systemDark]);

  const sendBackground = useMemo(() => {
    switch (settings.sendBackgroundMode) {
      case "match":
        return canvasBackground;
      case "white":
        return "#ffffff";
      case "black":
        return "#000000";
      case "custom":
        return settings.customSendBackground;
    }
  }, [canvasBackground, settings.customSendBackground, settings.sendBackgroundMode]);

  const updateSettings = (patch: Partial<AppSettings>) => {
    setSettings((current) => ({ ...current, ...patch }));
  };

  return {
    settings,
    updateSettings,
    canvasBackground,
    sendBackground,
    darkCanvas: isDarkColor(canvasBackground),
  };
}
