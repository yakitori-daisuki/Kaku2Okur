import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { SettingsPanel } from "./components/SettingsPanel";
import { SketchCanvas, type SketchCanvasHandle } from "./components/SketchCanvas";
import { Toolbar } from "./components/Toolbar";
import { TrackpadSuggestion } from "./components/TrackpadSuggestion";
import { renderSceneToDataUrl } from "./domain/render";
import type { CanvasObject, Tool, TrackpadTouch } from "./domain/types";
import { useSceneHistory } from "./hooks/useSceneHistory";
import { useSettings } from "./hooks/useSettings";
import {
  deliverImage,
  discardSession,
  isTauriRuntime,
  loadCurrentScene,
  setInvocationGesture,
  syncScene,
} from "./lib/bridge";

const SUGGESTION_KEY = "kaku2okur.trackpad-suggestion-dismissed";

export default function App() {
  const scene = useSceneHistory();
  const {
    settings,
    updateSettings,
    canvasBackground,
    sendBackground,
    darkCanvas,
  } = useSettings();
  const canvasRef = useRef<SketchCanvasHandle>(null);
  const [tool, setTool] = useState<Tool>("pen");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [stroke, setStroke] = useState(darkCanvas ? "#ffffff" : "#15181c");
  const [strokeWidth, setStrokeWidth] = useState(3);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [widthOpen, setWidthOpen] = useState(false);
  const [sending, setSending] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [trackpadMode, setTrackpadMode] = useState(settings.trackpadDefault);
  const [showTrackpadSuggestion, setShowTrackpadSuggestion] = useState(
    () => localStorage.getItem(SUGGESTION_KEY) !== "true",
  );

  useEffect(() => {
    if (darkCanvas && stroke === "#15181c") setStroke("#ffffff");
    if (!darkCanvas && stroke === "#ffffff") setStroke("#15181c");
  }, [darkCanvas, stroke]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 1800);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    if (!isTauriRuntime()) return;
    const unlisten = listen("session-opened", async () => {
      const recovery = await loadCurrentScene();
      scene.reset(recovery);
      setSelectedId(null);
      setTool("pen");
      setTrackpadMode(settings.trackpadDefault);
    });
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, [scene.reset, settings.trackpadDefault]);

  useEffect(() => {
    if (!isTauriRuntime() || !trackpadMode) return;
    const unlisten = listen<TrackpadTouch>("trackpad-touch", (event) => {
      canvasRef.current?.handleTrackpadTouch(event.payload);
    });
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, [trackpadMode]);

  useEffect(() => {
    void syncScene(scene.objects);
  }, [scene.objects]);

  const commit = useCallback(
    (objects: CanvasObject[]) => {
      scene.commit(objects);
    },
    [scene.commit],
  );

  const send = useCallback(
    async (objects: CanvasObject[] = scene.objects) => {
      if (sending || objects.length === 0) return;
      const dataUrl = renderSceneToDataUrl(objects, sendBackground);
      if (!dataUrl) return;

      setSending(true);
      try {
        const outcome = await deliverImage(dataUrl);
        if (outcome.status === "clipboardFallback") {
          setToast("Copied to clipboard");
        } else if (outcome.status === "preview" && outcome.message) {
          setToast(outcome.message);
        }
        if (outcome.status === "dispatched") {
          scene.reset();
          setSelectedId(null);
        }
      } catch {
        setToast("Copied to clipboard");
      } finally {
        setSending(false);
      }
    },
    [scene.objects, scene.reset, sendBackground, sending],
  );

  const discard = useCallback(async () => {
    scene.reset();
    setSelectedId(null);
    await discardSession();
  }, [scene.reset]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const editingText = target?.tagName === "TEXTAREA" || target?.tagName === "INPUT";
      if (editingText) return;

      const command = event.metaKey || event.ctrlKey;
      if (command && event.key.toLowerCase() === "z") {
        event.preventDefault();
        if (event.shiftKey) scene.redo();
        else scene.undo();
        setSelectedId(null);
        return;
      }
      if (command && event.key.toLowerCase() === "y") {
        event.preventDefault();
        scene.redo();
        setSelectedId(null);
        return;
      }
      if (event.key === "Enter") {
        event.preventDefault();
        void send();
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        if (canvasRef.current?.cancelActiveEdit()) return;
        if (selectedId) setSelectedId(null);
        else void discard();
        return;
      }
      if ((event.key === "Delete" || event.key === "Backspace") && selectedId) {
        event.preventDefault();
        commit(scene.objects.filter((object) => object.id !== selectedId));
        setSelectedId(null);
        return;
      }
      if (event.key.toLowerCase() === "t") {
        event.preventDefault();
        setTrackpadMode((active) => !active);
      }
    };

    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, [commit, discard, scene.objects, scene.redo, scene.undo, selectedId, send]);

  const handleSettingsChange = (patch: Partial<typeof settings>) => {
    updateSettings(patch);
    if (patch.invocationGesture) void setInvocationGesture(patch.invocationGesture);
    if (patch.trackpadDefault !== undefined) setTrackpadMode(patch.trackpadDefault);
  };

  return (
    <main
      className={`app-shell${trackpadMode ? " is-trackpad-mode" : ""}`}
      data-theme={darkCanvas ? "dark" : "light"}
    >
      <SketchCanvas
        ref={canvasRef}
        objects={scene.objects}
        tool={tool}
        stroke={stroke}
        strokeWidth={strokeWidth}
        background={canvasBackground}
        darkCanvas={darkCanvas}
        selectedId={selectedId}
        onSelectionChange={setSelectedId}
        onCommit={commit}
        onSend={(objects) => void send(objects)}
      />

      <Toolbar
        tool={tool}
        color={stroke}
        strokeWidth={strokeWidth}
        canUndo={scene.canUndo}
        canRedo={scene.canRedo}
        sending={sending}
        settingsOpen={settingsOpen}
        paletteOpen={paletteOpen}
        widthOpen={widthOpen}
        onToolChange={(next) => {
          setTool(next);
          setSelectedId(null);
          setSettingsOpen(false);
        }}
        onColorChange={(color) => {
          setStroke(color);
          setPaletteOpen(false);
        }}
        onStrokeWidthChange={setStrokeWidth}
        onUndo={() => {
          scene.undo();
          setSelectedId(null);
        }}
        onRedo={() => {
          scene.redo();
          setSelectedId(null);
        }}
        onSend={() => void send()}
        onToggleSettings={() => {
          setSettingsOpen((open) => !open);
          setPaletteOpen(false);
          setWidthOpen(false);
        }}
        onTogglePalette={() => {
          setPaletteOpen((open) => !open);
          setSettingsOpen(false);
          setWidthOpen(false);
        }}
        onToggleWidth={() => {
          setWidthOpen((open) => !open);
          setSettingsOpen(false);
          setPaletteOpen(false);
        }}
      />

      {settingsOpen && (
        <SettingsPanel settings={settings} onChange={handleSettingsChange} />
      )}

      {showTrackpadSuggestion && (
        <TrackpadSuggestion
          active={trackpadMode}
          onActivate={() => setTrackpadMode((active) => !active)}
          onDismiss={() => {
            localStorage.setItem(SUGGESTION_KEY, "true");
            setShowTrackpadSuggestion(false);
          }}
        />
      )}

      {toast && <div className="status-toast">{toast}</div>}
    </main>
  );
}
