# App Icon Source

Original vector artwork for Kaku2Okur. These shapes are not SF Symbols.

## Selected Envelope Concept

Open `Kaku2Okur-Envelope.icon` in Apple Icon Composer for the selected envelope,
sketch card, and pencil direction. See [envelope-design.md](envelope-design.md)
for editable layers and validation. The previews are `envelope-preview.png` and
`envelope-preview-dark.png`; `envelope-reference.png` preserves the selected
concept sheet. The macOS app build uses this document. The old `Kaku2Okur.icon`
is retained as a previous design, not as the active build input.

## Previous Design

- Canvas: 1024 by 1024, macOS.
- Background: white in the default appearance, graphite in dark appearance.
- `Kaku2Okur.icon` is the editable Icon Composer document. Its `Assets` directory contains the original SVG layers at shared canvas coordinates.
- Keep the ink trail below the pencil and send arrow.
- Use restrained glass depth on the blue foreground; preserve the legibility of the ink trail at small sizes.
- In dark appearance, change the ink trail to a light neutral.

## Build

`bash script/build_icon.sh` compiles `Kaku2Okur-Envelope.icon` with Xcode 26's `actool`, producing `target/app-icon/Assets.car`, `Kaku2Okur-Envelope.icns`, and the icon Info.plist keys. The macOS build script embeds both the asset catalog and ICNS fallback (as `Kaku2Okur.icns`) before signing. `CFBundleIconName` selects `Kaku2Okur-Envelope` in the catalog. The asset catalog retains appearance-specific rendering.

The artwork was authored as an Icon Composer document and previewed with Apple's bundled `ictool` CLI. The Icon Composer GUI's first-launch license agreement was not accepted or modified. The application folder copy is not changed by these scripts.

To export a preview with Icon Composer's bundled CLI:

```sh
"/path/to/Xcode.app/Contents/Applications/Icon Composer.app/Contents/Executables/ictool" \
  apps/native/assets/app-icon/Kaku2Okur.icon --export-image \
  --output-file target/app-icon/preview.png --platform macOS \
  --rendition Default --width 512 --height 512 --scale 1
```

Use `--rendition Dark` for the dark variant. The ink trail is an outlined path, allowing appearance-specific fills without closing its open center. Regenerate that outline with `swift script/outline_icon.swift apps/native/assets/app-icon/Kaku2Okur.icon/Assets/ink-trail.svg`.

Icon Composer exports are intended for the macOS build. Other platforms can render the original SVG artwork independently.
