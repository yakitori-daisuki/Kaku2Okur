# Use a shared Tauri core with native platform adapters

**Status:** Superseded by [ADR 0002](0002-use-rust-native-slint-shell.md)

Kaku2Okur will be a Tauri 2 desktop application with a shared Rust core and a shared TypeScript canvas UI. The shared layers own Canvas Sessions, Canvas Objects, command history, settings, rendering policy, and the delivery state machine; narrow native adapters own global Invocation Gestures, non-activating window behavior, Delivery Target restoration, clipboard access, paste dispatch, and device-specific trackpad input. This keeps the product genuinely portable to Windows while allowing the macOS-first release to use the native behavior its low-friction workflow requires; building the whole app in Swift would make the later Windows version a rewrite, while forcing all system integration through the lowest common denominator would weaken the defining interaction.

## Consequences

- macOS ships first and Windows follows against the same core contracts.
- Native adapters expose capabilities instead of pretending every platform has equivalent trackpad or focus APIs.
- Platform code may use Swift/Objective-C on macOS and Win32 interop on Windows, but it may not own the Sketch model or product workflow.
- Linux remains possible at the core boundary but is not an initial delivery commitment.
