## #1901 - App icons in the user guide

**What was decided.** Add a global Zola `icon` component using `get_url`
and an inline CSS class. Start each operation section with its app icon and show
control and browser icons beside the guide's control names. Copy all 93
PNG files named by the app's icon table and its licence notice into the
site. Link the notice from the footer. Use the site tests to check byte
parity, table names, operation mappings, rendered URLs and notice links.

**Why.** Users can match an operation or control in the guide to the app.
The deployment pins Zola 0.23.6. Zola 0.23 removed shortcodes in favour of
components, so calls use `{{ <icon name="op-face" /> }}` and the existing
`site/templates/shortcodes/icon.html` path contains a global component
definition. This follows Zola's current content rendering mechanism:
[Zola content rendering documentation](https://www.getzola.org/documentation/content/overview/).
`get_url` preserves a deployed site's path prefix. Black icons need light
ink under the site's dark colour scheme. Table and byte checks make a
renamed or changed app icon fail a test. Operation icons share their
existing picture line so the guide pages stay within the file-size law.

**Considered and dropped.** A symlink to `assets/ui` depends on Windows
checkout settings. A release workflow copy step changes process files and
does not help a standalone local site build. Inline SVGs duplicate the
app's raster asset pipeline. The previous attempt's Rust parity test
duplicates the site tests and makes a documentation-only check compile
the UI, so this patch keeps the pins in `site/tests`.

**Cost to change later.** Low. Refresh copied PNGs and the notice when the
app changes; the site test names missing or stale files. A future shared
asset build can replace those copies without changing guide components.
