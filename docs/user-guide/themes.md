# Make a theme

`assets/themes/theme.json.example` lists each setting that a theme file can
set. Each setting has its default value (the dark theme).

1. Copy `theme.json.example` to a new name that ends in `.json`, in the same
   folder. The app lists `.json` files only, so it never lists the example.
2. Change `descriptor.name`. Set `base` to `dark` or `light`.
3. Change the values that you want. Delete the settings that you do not
   want to change: a setting that the file omits keeps the value of `base`.
4. Select the theme in the theme switcher.

The loader refuses a file by name when a colour is not `#rrggbb` or
`#rrggbbaa`, when a setting name is not known, or when the text is not
readable against its ground. A geometry value outside its range is
clamped to the range.

## Colours (`colors`)

| Setting | What it changes |
|---|---|
| `ink` | Window and panel ground |
| `ink_alt` | Ribbon, browser and header ground |
| `well` | Input fields and empty recesses |
| `text` | Body text |
| `text_dim` | Secondary text |
| `text_faint` | Disabled and hint text |
| `border` | Lines between areas |
| `accent` | Selection and hover accent |
| `accent_hover` | Accent when the pointer is over it |
| `field_bg` | Input field ground |
| `field_hover` | Input field ground under the pointer |
| `field_active` | Input field ground while it is edited |
| `selection_bg` | Text selection and open list highlight |
| `selection_stroke` | Outline of a selection |
| `viewport` | Ground of the 3D view |
| `ribbon_ink` | Ink of a plain ribbon glyph |

## Geometry (`geometry`, in points)

| Setting | What it changes | Range |
|---|---|---|
| `radius_widget` | Corner radius of controls and ribbon cells | 0 to 12 |
| `radius_window` | Corner radius of windows | 0 to 24 |
| `radius_menu` | Corner radius of menus and lists | 0 to 24 |
| `item_spacing_x`, `item_spacing_y` | Gap between items | 0 to 32 |
| `button_padding_x`, `button_padding_y` | Button padding | 0 to 32 |
| `control_height` | Minimum height of an input field | 12 to 64 |
| `row_height` | Height of a browser tree row | 12 to 64 |
| `cell_h` | Height of a ribbon cell | 24 to 96 |
| `cell_w` | Largest width of a ribbon cell | 24 to 96 |
| `glyph_px` | Edge of a ribbon glyph | 8 to 64 |
| `font_label` | Caption under a ribbon glyph | 6 to 32 |
| `font_caption` | Ribbon group caption | 6 to 32 |
| `font_row` | Browser row label | 6 to 32 |
| `font_row_suffix` | Browser row suffix | 6 to 32 |
| `ribbon_tab_h` | Height of the ribbon tab row | 12 to 48 |
| `ribbon_cell_top_gap` | Gap from the tab row to the cells | 0 to 24 |
| `ribbon_caption_gap` | Gap from the cell box to the caption row | 0 to 24 |
| `ribbon_caption_row_h` | Height of the caption row | 12 to 48 |
| `ribbon_bottom_gap` | Gap below the caption row | 0 to 24 |
