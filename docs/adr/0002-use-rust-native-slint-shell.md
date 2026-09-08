# Use a Rust-native Slint shell

**Status:** Accepted

**Date:** 2026-08-29

## Context

Kaku2Okur is intended to stay resident all day and appear instantly. Its defining workflow needs global input, native window management, clipboard ownership, focus restoration, and synthetic paste on both macOS and Windows. A browser-backed shell was useful for proving the interaction, but its resident memory and process overhead conflict with lightweight operation. Flutter removes the browser engine but still embeds a comparatively large application engine and would split the drawing and platform layers across Dart and native code.

## Decision

Build the active app as one Rust executable using:

- Slint with the winit software renderer for the retained UI.
- tiny-skia for deterministic canvas rendering and cropped Send Results.
- A shared Rust model for Canvas Objects, history, selection, settings, and delivery.
- Narrow Objective-C/AppKit and Win32 adapters for platform behavior.
- Memory-mapped system fonts rather than eagerly decoding CJK font files.

No WebView, JavaScript runtime, Flutter engine, account layer, or service-specific AI integration belongs in the resident process. Platform-only capabilities such as raw macOS trackpad input are explicit and may be unavailable elsewhere.

## Consequences

- macOS and Windows share the behavior that matters while retaining native control over focus and input.
- The software renderer keeps idle GPU and framework overhead low; complex animation is deliberately not a product priority.
- UI changes use Slint rather than the larger React ecosystem.
- Slint attribution is exposed through the top-level tray menu using `AboutSlint`.
- The Tauri implementation remains preserved under `apps/tauri-legacy` as a reference, not as an active build target.

## Measured baseline

On the development Mac, the release app measured about 18 MB physical footprint on a cold hidden launch and about 64 MB with the canvas visible. After the first display, the software window backend retains its reusable surfaces, so a hidden process remains near the visible baseline. The staged app bundle measured about 9.8 MB. These are comparison baselines, not hard guarantees across operating systems and hardware.
