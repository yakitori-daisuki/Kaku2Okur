import { Hand, X } from "lucide-react";

interface TrackpadSuggestionProps {
  active: boolean;
  onActivate: () => void;
  onDismiss: () => void;
}

export function TrackpadSuggestion({
  active,
  onActivate,
  onDismiss,
}: TrackpadSuggestionProps) {
  return (
    <div className={`trackpad-suggestion${active ? " is-active" : ""}`}>
      <button type="button" className="trackpad-action" onClick={onActivate}>
        <Hand aria-hidden="true" />
        <span>Trackpad Sketch</span>
        <kbd>T</kbd>
      </button>
      <button
        type="button"
        className="suggestion-close"
        aria-label="Dismiss Trackpad Sketch suggestion"
        title="Dismiss"
        onClick={onDismiss}
      >
        <X aria-hidden="true" />
      </button>
    </div>
  );
}

