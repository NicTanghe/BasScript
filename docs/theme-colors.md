# Theme colors

Open Settings → Theme to preview changes directly in the editor. Dark uses dark
panels, fields and buttons with light interface text. The active file and selected
controls share one accent color.

- **Colors**: app, top menu, explorer, plain editor, processed pane, paper and selection backgrounds.
- **UI**: panels and popups, input fields, button normal/hover/pressed colors, active controls and borders.
- **Text**: interface/plain text, muted labels, and screenplay/Markdown text colors.
- **Links**: link colors by entity type.
- **Glass**: native transparency switches.
- **Themes**: built-in presets and named themes.

OK, Save and Save As New use the same normal, hover and pressed colors as the
toolbar buttons. Interface text colors also apply to explorer names, theme
labels, metadata fields, settings and popups.

Paper is always opaque. Processed-background glass affects the surrounding pane,
including the gaps between pages. The paper picker omits alpha, and loading or
saving a translucent paper color sets its alpha to 1.

Changes save to `settings/theme.ron`; named themes save under `settings/themes/`.
The added fields use the same `(red, green, blue, alpha)` values as other colors:

| Setting | RON field |
| --- | --- |
| Plain editor | `plain_background` |
| Panels / popups | `panel_background` |
| Input fields | `input_background` |
| Buttons | `button_background` |
| Button hover | `button_hover` |
| Button pressed | `button_pressed` |
| Active controls / file | `active_background` |
| Borders | `border_color` |

Existing theme files continue to load. Missing interface colors use a dark or
light palette based on the saved app background; explicitly saved colors are
preserved. Changes and preset switches update existing controls immediately.
