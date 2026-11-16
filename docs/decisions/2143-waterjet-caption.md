## #2143 - Waterjet short caption

**What was decided:** the door's short caption for `Kind::Waterjet` is
"Waterjet". `short_caption` has one explicit arm per kind and no wildcard.

**Why:** the match is exhaustive, so the missing arm breaks the build of
`camcraft-ui` on main. The tool settings pane already names the kind
"Waterjet nozzle", so the word matches the UI.

**Considered and dropped:** a wildcard arm (the issue forbids it). A
ribbon mode row in `MACHINE_MODES` for Waterjet (no built-in waterjet
machine ships yet, so no row is due; `caption()` keeps its "Machine"
fallback until one ships).

**Cost to change later:** one string in `short_caption`, and one assert
in `every_machine_kind_has_a_short_caption`.
