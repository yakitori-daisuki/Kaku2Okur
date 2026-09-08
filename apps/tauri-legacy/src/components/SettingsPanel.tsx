import type { AppSettings, BackgroundMode, SendBackgroundMode } from "../domain/types";

interface SettingsPanelProps {
  settings: AppSettings;
  onChange: (patch: Partial<AppSettings>) => void;
}

const BACKGROUNDS: Array<{ value: BackgroundMode; label: string; swatch: string }> = [
  { value: "system", label: "System", swatch: "linear-gradient(135deg, #fff 50%, #111 50%)" },
  { value: "white", label: "White", swatch: "#fff" },
  { value: "black", label: "Black", swatch: "#000" },
  { value: "custom", label: "Custom", swatch: "custom" },
];

const SEND_BACKGROUNDS: Array<{ value: SendBackgroundMode; label: string }> = [
  { value: "match", label: "Match canvas" },
  { value: "white", label: "White" },
  { value: "black", label: "Black" },
  { value: "custom", label: "Custom" },
];

export function SettingsPanel({ settings, onChange }: SettingsPanelProps) {
  return (
    <aside className="settings-panel" aria-label="Settings">
      <section>
        <h2>Canvas</h2>
        <div className="background-options">
          {BACKGROUNDS.map((option) => (
            <button
              type="button"
              key={option.value}
              className={settings.backgroundMode === option.value ? "is-selected" : ""}
              onClick={() => onChange({ backgroundMode: option.value })}
            >
              {option.value === "custom" ? (
                <input
                  type="color"
                  value={settings.customBackground}
                  aria-label="Custom canvas background"
                  onChange={(event) =>
                    onChange({ backgroundMode: "custom", customBackground: event.target.value })
                  }
                  onClick={(event) => event.stopPropagation()}
                />
              ) : (
                <span className="background-swatch" style={{ background: option.swatch }} />
              )}
              <span>{option.label}</span>
            </button>
          ))}
        </div>
      </section>

      <section>
        <label htmlFor="send-background">Send background</label>
        <select
          id="send-background"
          value={settings.sendBackgroundMode}
          onChange={(event) =>
            onChange({ sendBackgroundMode: event.target.value as SendBackgroundMode })
          }
        >
          {SEND_BACKGROUNDS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        {settings.sendBackgroundMode === "custom" && (
          <input
            className="wide-color-input"
            type="color"
            value={settings.customSendBackground}
            aria-label="Custom send background"
            onChange={(event) => onChange({ customSendBackground: event.target.value })}
          />
        )}
      </section>

      <section>
        <label htmlFor="invocation">Invocation</label>
        <select
          id="invocation"
          value={settings.invocationGesture}
          onChange={(event) =>
            onChange({
              invocationGesture: event.target.value as AppSettings["invocationGesture"],
            })
          }
        >
          <option value="dual-modifier">Both Command / Ctrl</option>
          <option value="conventional">Cmd / Ctrl + Shift + Space</option>
        </select>
      </section>

      <section className="toggle-row">
        <label htmlFor="trackpad-default">Open in Trackpad Sketch</label>
        <input
          id="trackpad-default"
          type="checkbox"
          checked={settings.trackpadDefault}
          onChange={(event) => onChange({ trackpadDefault: event.target.checked })}
        />
      </section>
    </aside>
  );
}
