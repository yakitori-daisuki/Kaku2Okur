# Kaku2Okur

Kaku2Okur is a desktop tool for quickly expressing a visual idea and delivering it to the input destination the user was already working in. It treats a drawing as a short-lived message, not as a document to manage.

## Implementation Priorities

Lightweight resident operation takes priority over optimizing for the fastest prototype. The active application is a single Rust-native process with a Slint software-rendered UI and tiny-skia renderer. It must not require a WebView, JavaScript runtime, Flutter engine, network service, or account to complete the core workflow.

The Sketch model and Send workflow are platform-neutral. AppKit/Objective-C and Win32 adapters may own window, input, focus, clipboard, and paste behavior, but platform code must not fork the product model. Platform-only features are capability-gated instead of simulated.

## Language

**Canvas Session**:
One drawing interaction from invocation until send or discard. A Canvas Session has one Sketch and one Delivery Target.
_Avoid_: Note, file, board

**Sketch**:
The visual message being composed during a Canvas Session. A Sketch contains Canvas Objects and a Background.
_Avoid_: Screenshot, memo, image

**Canvas Object**:
An editable element in a Sketch, such as a pen stroke, arrow, rectangle, ellipse, or text label. "Shape" refers only to geometric Canvas Objects.
_Avoid_: Item, layer

**Selection**:
The single Canvas Object currently chosen for editing. Selection is an editing state, not a kind of Canvas Object or drawing tool.
_Avoid_: Active layer, picked shape

**Background**:
The surface behind the Canvas Objects and part of the visual meaning of a Sketch. Its appearance may follow the system or a user preference.
_Avoid_: Theme

**Invocation Gesture**:
A system-wide user gesture that starts or toggles a Canvas Session. When a session is visible, invoking temporarily hides it without discarding the sketch; invoking again resumes it. It is broader than a conventional keyboard shortcut because it may be a double tap or a simultaneous modifier-key press.
_Avoid_: Hotkey, launch shortcut

**Delivery Target**:
The text or image input destination the user was working in immediately before invoking Kaku2Okur. A Delivery Target may belong to an AI service or any other application that accepts images.
_Avoid_: AI app, selected area, active window

**Send**:
The act of finishing the current Sketch and asking Kaku2Okur to deliver it to the Delivery Target. Sending is distinct from saving or exporting a document.
_Avoid_: Save, upload

**Cancel Edit**:
The act of reverting the current unfinished interaction while keeping the Canvas Session and its previously committed Canvas Objects open.
_Avoid_: Discard, close

**Discard**:
The act of ending a Canvas Session without sending its Sketch. Discard applies to the whole Canvas Session, not merely the current edit or Selection.
_Avoid_: Cancel edit, close

**Paste Dispatch**:
The operating system accepting Kaku2Okur's request to paste a Send Result into the Delivery Target. It does not prove that the destination application accepted or rendered the result.
_Avoid_: Paste success

**Send Result**:
The flattened visual representation produced from a Sketch for delivery. It is cropped to the meaningful drawn area rather than representing the entire working canvas.
_Avoid_: Canvas file, screenshot

**Clipboard Fallback**:
A Send Result left on the system clipboard when paste dispatch cannot be performed. It lets the user recover the result without repeating the Sketch.
_Avoid_: Paste, export

**Recovery Sketch**:
The single, temporary Sketch retained only so an interrupted Send can be retried during the current app run. It is not a saved document or an entry in a drawing history.
_Avoid_: Draft, autosave, history item

**Trackpad Sketch Mode**:
An optional input mode in which the trackpad surface maps to the canvas and one-finger movement operates the selected Canvas tool. Availability depends on the capabilities of the current platform and device.
_Avoid_: Tablet mode, touch mode

**Trackpad Hotspot**:
One of eight configurable regions at the four corners and four edge midpoints of the trackpad. A Trackpad Hotspot reserves its region only when it has an assigned action and can activate on touch-down, long press, or physical click.
_Avoid_: Gesture button, trackpad key

**Hold to Tidy**:
An optional pen behavior that replaces a recognizable unfinished stroke with a precise geometric Canvas Object when the pointer stays still at the end of that stroke. It currently recognizes straight lines, rectangles, and ellipses.
_Avoid_: AI correction, autocorrect

## Flagged Ambiguities

**Paste success**:
There is no universal signal proving that another application accepted an image. The product therefore distinguishes Paste Dispatch from acceptance by the Delivery Target.

**Cursor**:
Use **pointer** for the on-screen pointing device and **text insertion point** for the caret inside a Delivery Target.

**Screenshot**:
A screenshot may later be used as source material for a Sketch, but a blank Sketch is the primary workflow and must not be called a screenshot.

## Example Dialogue

> **Developer:** Does every Canvas Session target ChatGPT?
>
> **Product:** No. It targets the Delivery Target that was in use before the Invocation Gesture. ChatGPT, Claude, Gemini, and a regular image-capable editor are all valid examples.
>
> **Developer:** Can we report that Send succeeded after the paste shortcut was issued?
>
> **Product:** Report Paste Dispatch only. If dispatch cannot happen, preserve the Send Result through Clipboard Fallback. The destination accepting the image is not universally observable.
>
> **Developer:** Is a clipboard screenshot the starting point?
>
> **Product:** No. A Canvas Session starts with a blank Sketch. Screenshot editing is supplementary.
