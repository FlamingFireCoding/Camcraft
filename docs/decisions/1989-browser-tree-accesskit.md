## #1989 - AccessKit labels for painted browser tree rows and context menu

**Decided:** Label each painted browser row with `WidgetType::Button` and
`"Browser: {label}"`, each eye toggle with `WidgetType::Button` and
`"{Hide|Show} {label}"`, and each context-menu row in `tree_menu` with its
rendered caption (`WidgetType::Button` with live state `*runnable && !is_current`
for actionable rows, `WidgetType::Label` with `Sense::hover()` for section
headers). Clamp the browser tree width in `browser_pane::browser` to the
visible clip edge (`(ui.clip_rect().right() - ui.cursor().min.x).max(0.0)`) so
eye toggles stay inside the pane clip rect. Filter `"Browser: ..."` nodes out
of `setups_declutter::shows`. Remove `BROWSER_TREE_TOP` and `BROWSER_EYE_X`
from `crates/ui/src/egui_app/tests.rs` and drive all six browser behaviour
tests through AccessKit queries (deriving drag coordinates from the target
row's node rect).

**Why:** Bare row labels (`"Machine"`, `"Post"`, `"Tool list"`, `"Setup1"`)
collided with page-level `query_by_label` and `query_all_by_label` assertions
because the browser pane stays visible alongside the active property page.
`"Browser: {label}"` keeps browser rows distinct while still searchable by label.
In `browser_pane::browser`, the top button strip inside `egui_dock`'s
`ScrollArea` pushed `ui.available_width()` to `240.8` past the tab's right clip
edge (`228.0`), placing the `Stock` eye center (`228.8`) outside
`interact_rect` (`219.8..228.0`) so `get_by_label("Hide Stock").click()`
missed; clamping the tree rect width to the visible clip edge keeps the eye
inside `clip_rect` and hit-testable at its AccessKit center.

**Considered and dropped:** Using bare `row.label` on browser rows caused nine
existing unit tests (`machine_page`, `every_field_lives_on_exactly_one_section`,
`every_registry_kind_gets_its_tab_bar`, `setup_tree`, and `setups_declutter`)
to see duplicate or unexpected labels from the persistent browser pane.
Switching `ShellTabs::scroll_bars` for `Tab::Browser` instead of clamping the
tree rect was dropped because it would not by itself prevent `ui.available_width()`
from expanding past `clip_rect.right()` under wide content. Clicking eye toggles
by hardcoded `er.left()` coordinates instead of clamping the tree rect was
dropped because it leaves the eye clipped in the UI and breaks AccessKit
`.click()`.

**Cost to change later:** Browser row queries in UI tests prefix `"Browser: "`;
changing the prefix only requires updating those test queries and the prefix
filter in `setups_declutter::shows`.
