## #2042 - Packages ship the examples directory

**Decided.** Each package carries `examples/` whole, beside `assets/`. It is a
sibling of `assets/` in the zip, installer, tar.gz, FreeBSD tarball, snap,
AppImage (`usr/bin/examples`), Flatpak (`/app/bin/examples`), deb and rpm
(`/usr/lib/camcraft/examples`), AUR (`/usr/lib/camcraft/examples`) and in
`Contents/Resources/examples` of the macOS app.

**Why.** The app already finds `assets/` beside the executable. The same
staged tree feeds every package, so one rule fits all. No system needs a
shared data directory (`/usr/share`) for this. The example `.camproj` files
embed their model (the `facets` array), so they hold no path to `models/`.
No example project needed an edit. `models/first-part.obj` ships as a file
that the user can import.

**Dropped.** A shared `/usr/share/camcraft/examples` directory: it needs a
second path rule for each Linux package type. A File menu entry to open the
examples: filed as a follow-up issue, not part of this fix.

**Cost to change.** Low: move one copy line in each packager.
