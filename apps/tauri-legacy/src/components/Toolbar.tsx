import {
  ArrowUpRight,
  Circle,
  Eraser,
  MousePointer2,
  Pencil,
  Pipette,
  Redo2,
  Send,
  Settings,
  Square,
  Type,
  Undo2,
} from "lucide-react";
import type { ComponentType, SVGProps } from "react";
import type { Tool } from "../domain/types";

type IconType = ComponentType<SVGProps<SVGSVGElement>>;

const TOOLS: Array<{ id: Tool; label: string; icon: IconType }> = [
  { id: "select", label: "Select", icon: MousePointer2 },
  { id: "pen", label: "Pen", icon: Pencil },
  { id: "eraser", label: "Eraser", icon: Eraser },
  { id: "arrow", label: "Arrow", icon: ArrowUpRight },
  { id: "rectangle", label: "Rectangle", icon: Square },
  { id: "ellipse", label: "Ellipse", icon: Circle },
  { id: "text", label: "Text", icon: Type },
];

const COLORS = ["#15181c", "#1769ff", "#df5b4f", "#18a46b", "#f0a000", "#ffffff"];

interface ToolButtonProps {
  label: string;
  icon: IconType;
  selected?: boolean;
  disabled?: boolean;
  onClick: () => void;
}

function ToolButton({ label, icon: Icon, selected, disabled, onClick }: ToolButtonProps) {
  return (
    <button
      type="button"
      className={`icon-button${selected ? " is-selected" : ""}`}
      aria-label={label}
      title={label}
      disabled={disabled}
      onClick={onClick}
    >
      <Icon aria-hidden="true" />
    </button>
  );
}

interface ToolbarProps {
  tool: Tool;
  color: string;
  strokeWidth: number;
  canUndo: boolean;
  canRedo: boolean;
  sending: boolean;
  settingsOpen: boolean;
  paletteOpen: boolean;
  widthOpen: boolean;
  onToolChange: (tool: Tool) => void;
  onColorChange: (color: string) => void;
  onStrokeWidthChange: (width: number) => void;
  onUndo: () => void;
  onRedo: () => void;
  onSend: () => void;
  onToggleSettings: () => void;
  onTogglePalette: () => void;
  onToggleWidth: () => void;
}

export function Toolbar({
  tool,
  color,
  strokeWidth,
  canUndo,
  canRedo,
  sending,
  settingsOpen,
  paletteOpen,
  widthOpen,
  onToolChange,
  onColorChange,
  onStrokeWidthChange,
  onUndo,
  onRedo,
  onSend,
  onToggleSettings,
  onTogglePalette,
  onToggleWidth,
}: ToolbarProps) {
  return (
    <>
      <div className="toolbar-wrap" data-tauri-drag-region>
        <div className="toolbar" role="toolbar" aria-label="Drawing tools">
          {TOOLS.map((item) => (
            <ToolButton
              key={item.id}
              label={item.label}
              icon={item.icon}
              selected={tool === item.id}
              onClick={() => onToolChange(item.id)}
            />
          ))}

          <span className="toolbar-divider" aria-hidden="true" />

          <ToolButton label="Undo" icon={Undo2} disabled={!canUndo} onClick={onUndo} />
          <ToolButton label="Redo" icon={Redo2} disabled={!canRedo} onClick={onRedo} />

          <div className="control-anchor">
            <button
              type="button"
              className="icon-button swatch-button"
              aria-label="Stroke color"
              title="Stroke color"
              onClick={onTogglePalette}
            >
              <span className="active-swatch" style={{ background: color }} />
            </button>
            {paletteOpen && (
              <div className="popover palette-popover" role="dialog" aria-label="Stroke color">
                {COLORS.map((value) => (
                  <button
                    type="button"
                    key={value}
                    className={`color-swatch${color === value ? " is-selected" : ""}`}
                    style={{ background: value }}
                    aria-label={`Use ${value}`}
                    onClick={() => onColorChange(value)}
                  />
                ))}
                <label className="custom-color" title="Custom color">
                  <Pipette aria-hidden="true" />
                  <input
                    type="color"
                    value={color}
                    aria-label="Custom stroke color"
                    onChange={(event) => onColorChange(event.target.value)}
                  />
                </label>
              </div>
            )}
          </div>

          <div className="control-anchor">
            <button
              type="button"
              className="width-button"
              aria-label="Stroke width"
              title="Stroke width"
              onClick={onToggleWidth}
            >
              <span style={{ height: Math.min(6, strokeWidth) }} />
            </button>
            {widthOpen && (
              <div className="popover width-popover" role="dialog" aria-label="Stroke width">
                <input
                  type="range"
                  min="1"
                  max="16"
                  step="1"
                  value={strokeWidth}
                  aria-label="Stroke width"
                  onChange={(event) => onStrokeWidthChange(Number(event.target.value))}
                />
                <output>{strokeWidth}</output>
              </div>
            )}
          </div>

          <ToolButton label="Send" icon={Send} disabled={sending} onClick={onSend} />
        </div>
      </div>

      <div className="settings-anchor">
        <ToolButton
          label="Settings"
          icon={Settings}
          selected={settingsOpen}
          onClick={onToggleSettings}
        />
      </div>
    </>
  );
}

