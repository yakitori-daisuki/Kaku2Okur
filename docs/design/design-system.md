# Kaku2Okur Design System

The production reference is [kaku2okur-concept.png](./kaku2okur-concept.png). The UI is a compact editor surface, not a document window or a marketing page.

## Tokens

- Canvas light: `#ffffff`
- Canvas dark: `#111315`
- Chrome light: `rgba(250, 251, 252, 0.96)`
- Chrome dark: `rgba(31, 34, 38, 0.96)`
- Primary text: `#1f2328` / `#f4f6f8`
- Muted text: `#66707a` / `#aab2bb`
- Border: `#d8dde3` / `#3a4047`
- Selection: `#1769ff` / `#55bdf0`
- Fallback: `#df5b4f`
- Radius: 6px for controls, 8px for the window
- Toolbar control: 36px square, 18px icon, 1.75px stroke
- UI font: system UI, 13px controls, 12px secondary status

## Layout

- One canvas fills the window.
- One floating toolbar is centered at the top with no nested containers.
- The settings control sits at the far edge without creating a sidebar.
- Trackpad Sketch appears as one temporary suggestion at the bottom edge.
- Tooltips carry control names; persistent instructional copy is omitted.

## Motion

- Toolbar and suggestion enter in 140ms using opacity and a 4px translation.
- Tool changes do not resize or shift the toolbar.
- Reduced-motion users receive immediate state changes.

