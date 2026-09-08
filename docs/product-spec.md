# Kaku2Okur Product Spec

## Product Promise

Invoke a blank canvas from anywhere, draw the idea that is difficult to put into words, press Enter, and continue in the original input destination with the drawing attached.

The primary workflow must remain useful for ChatGPT, Claude, Gemini, and other image-capable inputs without requiring service-specific integrations.

## Core Workflow

1. The user places the text insertion point in the intended Delivery Target.
2. The user performs the configurable Invocation Gesture.
3. A compact Canvas Session appears on the display containing the pointer.
4. The user draws with pen, eraser, arrow, rectangle, ellipse, or text.
5. Enter sends. Escape discards. Clicking outside the canvas does not close it.
6. Kaku2Okur dispatches paste to the previous Delivery Target and closes the canvas.
7. If paste dispatch cannot be performed, the Send Result remains on the clipboard and a small fallback notice appears.

Successful dispatch is silent. Kaku2Okur must not claim that the destination accepted the image because generic desktop APIs cannot prove that reliably.

## Canvas

- Start each Canvas Session with a blank canvas.
- Store strokes and shapes as editable objects during the session.
- Initial tools: pen, eraser, arrow, rectangle, ellipse, and text.
- Include a selection tool alongside the creation tools.
- Selection supports one Canvas Object at a time, with move, resize, and delete actions.
- Multiple selection and rotation are deferred from the initial tool set.
- Support undo with Command/Ctrl+Z and redo with Command/Ctrl+Shift+Z or Command/Ctrl+Y.
- Typing a printable character or starting IME composition on the neutral canvas immediately opens the text editor at the latest pointer position, before conversion is confirmed. The same input control retains focus, preedit, candidate selection, and committed text throughout the transition. Finishing the text returns to the previously selected tool.
- While editing text, Enter belongs to the native input method for conversion confirmation or inserts a line break; it never sends the Sketch. Shift+Enter also inserts a line break.
- Command+Enter on macOS (Ctrl+Enter on Windows), Done, or clicking outside the text editor commits the text without sending. The finish-input shortcut never sends, including when repeated after the editor closes. Enter sends again after returning to the canvas; the explicit Send button remains available during editing.
- With the pen, text, or selection tool active, drag a placed text label directly to adjust its position. A move is one undoable action, and Escape restores its starting position. Other creation tools and the eraser retain their chosen behavior.
- Hold to Tidy is enabled by default and can be disabled in Settings. Pausing at the end of a pen stroke converts a recognizable straight line, rectangle, or ellipse into a precise editable object; unrecognized strokes remain untouched.
- Escape cancels an unfinished edit first, clears Selection second, and discards the Canvas Session only from the neutral canvas state.
- Empty-canvas Send is a no-op and closes nothing accidentally.
- Render the Send Result around the meaningful content bounds with a small, consistent padding.
- Do not create a persistent Sketch history.
- Keep at most one Recovery Sketch in memory so an interrupted Send can be retried without redrawing.
- Clear the Recovery Sketch after successful dispatch, explicit discard, or app termination.

## Background

- Default canvas background follows the operating system appearance: white in light mode and black in dark mode.
- Canvas background preferences: follow system, white, black, or custom color.
- Initial Send Result uses the visible canvas background.
- Keep canvas appearance and send-background policy separate in the settings model so a later version can offer "always send on white" without changing drawing behavior.

## Window Behavior

- The first Canvas Session on a display appears centered and occupies approximately half its width and half its height.
- Moving or resizing stores a display-relative frame for that display.
- Reconnecting a known display restores its own frame.
- The blank upper window area and toolbar are draggable. A completed drag saves the frame immediately, while Send, Discard, and Close save it again as a fallback.
- Display keys use stable monitor identity where the operating system exposes it, rather than process-local monitor handles.
- An unavailable or unknown display falls back to a centered frame on the display containing the pointer.
- Display selection is a replaceable policy. The initial policy uses the pointer display; a future policy may use the Delivery Target when detection is reliable.
- The canvas does not close on outside click.
- Invocation raises the canvas, but it uses the normal window level and does not stay above every other application.
- The visible close control, Command/Ctrl+W, and the operating system close action discard the current Canvas Session and hide the window.
- On macOS, the window should avoid activating Kaku2Okur where possible so the prior application remains the Delivery Target. The implementation may still retain an in-memory target reference for reliable restoration.

## Invocation

- Invocation Gesture is configurable.
- Initial macOS default: simultaneous left and right Option keys.
- Initial Windows default: simultaneous left and right Alt keys.
- Available alternatives are simultaneous or double-tapped Command/Control, double-tapped Option/Alt, and Command/Control+Shift+Space.
- The configured Invocation Gesture toggles the canvas. While a Canvas Session is visible it temporarily hides the window, preserves the sketch, and restores focus to the Delivery Target. Invoking again resumes that sketch and captures the currently focused app as the Delivery Target.
- Escape and the window close control end and discard the Canvas Session; they are not temporary-hide actions.
- A keyboard lacking one of the required modifier keys must be detected during setup and offered a conventional chord.
- On macOS, global Invocation requires Input Monitoring access. The app preflights and requests that operating-system permission before starting its listener.
- If access is missing or the listener cannot start, show a focused permission dialog with Open Settings, Check Again, and Later actions instead of failing silently.
- macOS reads the current left/right modifier state through CoreGraphics; Windows reads it through `GetAsyncKeyState` without an additional permission prompt. Event tracking remains the fallback for platforms without a native state query.
- On macOS, invocation consumes a listen-only session event tap and reads side-specific modifier bits from each event. It does not overwrite event-time state with a later global key-state query or translate typed characters. A disabled event tap is re-enabled while permission remains granted.
- While waiting for Input Monitoring permission, detect a newly granted permission automatically and start listening without requiring Check Again.

## Trackpad Sketch Mode

- Trackpad Sketch Mode is an optional beta capability, initially implemented for supported macOS trackpads.
- The toolbar toggles the mode while the canvas is open; a setting can make it the default mode. Letter keys remain available for immediate text entry.
- The whole trackpad maps to the whole canvas.
- One-finger motion uses the currently selected canvas tool and two-finger gestures are ignored in the first version.
- The four corners and four edge midpoints are configurable Trackpad Hotspots.
- Each Trackpad Hotspot can use touch-down, long press, or physical click as its trigger.
- Available actions are tool selection, undo, redo, delete Selection, Send, or no action.
- A configured Trackpad Hotspot reserves its region for its trigger; an unconfigured hotspot remains part of the drawing surface.
- Leaving a long-press or click hotspot before activation cancels it, and a hotspot click is consumed before it reaches unrelated canvas UI.
- A subtle suggestion introduces the mode. "Later" dismisses it for the current app run; enabling Trackpad Sketch Mode by default suppresses future suggestions until that setting is turned off.
- Core drawing and sending must work on Windows even when equivalent raw trackpad input is unavailable.

## Delivery

- Delivery uses a temporary clipboard payload followed by a platform paste dispatch.
- Preserve the previous clipboard payload before attempting delivery.
- Restore it after dispatch is handed to the operating system.
- If target restoration or paste dispatch fails, leave the Send Result on the clipboard instead and show "Copied to clipboard" briefly.
- Do not show routine success notifications.
- Accessibility/input-control permissions are requested only when the user reaches a feature that needs them, with a clear retry path.

## Platform Scope

- macOS is the first shipping platform.
- Windows is the second supported platform and must share the same Sketch model, tools, settings semantics, and Send workflow.
- Platform-specific capabilities live behind explicit capability checks; feature parity is not faked when the operating system or hardware cannot provide an equivalent input signal.
- Linux is not an initial release target, but the shared core must not depend on macOS or Windows concepts.

## Supplementary Features

- Editing an image already on the clipboard may be added after the blank-canvas workflow is stable.
- Screenshot capture should integrate with existing capture tools or clipboard input before Kaku2Okur grows a full capture subsystem.
- User-defined shapes and local image generation may later create Canvas Objects through the same insertion boundary.

## Deferred Scope

- Accounts, cloud sync, or online storage.
- A persistent whiteboard library or document browser.
- Service-specific ChatGPT, Claude, or Gemini browser extensions.
- Full screenshot capture and scrolling capture.
- Guaranteed confirmation that a third-party destination accepted an image.
