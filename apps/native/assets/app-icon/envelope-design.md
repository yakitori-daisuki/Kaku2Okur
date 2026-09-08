# Envelope Icon

`Kaku2Okur-Envelope.icon` is the editable Apple Icon Composer document for the
selected third concept (blue envelope, sketch card, yellow pencil). The macOS app
build uses this document. The previous `Kaku2Okur.icon` is retained separately.

The selected raster concept has been reconstructed as scalable SVG artwork,
not embedded as a flattened screenshot. The document contains four groups and
six editable layers: open flap, paper, landscape, side folds, front fold, pencil.
All SVGs share a 1024 by 1024 canvas. Icon Composer controls the background,
appearance, shadows, and layer effects. Edit SVG sources to change path geometry.
The document lists groups and layers front-to-back (the pencil is first).

Open `Kaku2Okur-Envelope.icon` with Apple's Icon Composer (bundled with Xcode).
The document has default and dark backgrounds; the artwork stays opaque so the
pencil and paper do not wash out. The yellow pencil includes a dark graphite tip
and separate wood color. Preview exports are in `envelope-preview.png` and
`envelope-preview-dark.png`.

This is an editable reconstruction of the selected direction, not a pixel-exact
conversion of the generated raster's paper texture and lighting.

## Validation

Apple's bundled `ictool` successfully exported both default and dark previews.
Xcode `actool` successfully compiled the document into `Assets.car` and
`Kaku2Okur-Envelope.icns` under `target/envelope-icon`, without warnings or errors.
The Icon Composer GUI was not used for this validation; its first-launch license
agreement has not been accepted on the user's behalf.

```sh
mkdir -p target/envelope-icon
xcrun actool apps/native/assets/app-icon/Kaku2Okur-Envelope.icon \
  --compile target/envelope-icon --platform macosx \
  --minimum-deployment-target 14.0 --target-device mac \
  --app-icon Kaku2Okur-Envelope --standalone-icon-behavior all \
  --output-partial-info-plist target/envelope-icon/icon-info.plist \
  --warnings --errors
```
