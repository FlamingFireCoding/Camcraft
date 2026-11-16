## #2002 - Selected tool settings and a separate tool catalog

- **Show a selected tool's editable settings in Properties.** Geometry,
  tool kind, material and cutting data, tool number, holder dimensions,
  and body/profile/cutter controls describe one document tool. They belong
  beside its browser selection. The existing summary and Edit button add
  an unnecessary step before reaching those settings. A summary-only page
  and a separate window for every existing-tool edit were dropped because
  neither meets the requested browser-to-Properties workflow.

- **Give catalog operations their own dock tab.** Catalog rows and their
  Add buttons, New tool, library import/export, import mode, and Import
  tool holders describe library management rather than the selected tool.
  The Tools ribbon button and browser View > Tool library… open that
  surface without replacing the selected tool's Properties. The tab is
  closable and may float like the other context tabs. Keeping these controls under
  every selected tool was dropped because it hides the tool's settings
  among unrelated actions. The former page has no control literally named
  “Support features”; holder/body/profile controls are tool settings and
  stay with the tool. The per-tool Catalog metadata header becomes
  Tool information: part number, vendor, coating, helix and notes describe
  the selected tool rather than the library browser.

- **Reuse the editor and its commands.** Properties and the creation
  window share `tool_settings::contents`, including Apply/Add. Properties
  stacks the fields, preview and body/profile controls vertically for its
  narrow width. The legacy editor and creation window retain their wide
  arrangement. Tool
  geometry, cutting data and numbers keep their existing command history
  paths. The New tool action retains a creation window because there is
  no document tool to select yet. Duplicating field widgets or introducing
  direct world writes was dropped to avoid divergent validation and undo
  behavior. The setup Tool list's Edit action also targets Properties.
  Selecting a tool closes any open tool-editor draft so the shared fields
  have one active editor. A legacy window opened directly suppresses the
  Properties fields until that window closes.

- **Open and focus tabs idempotently through layout requests.**
  `LayoutRequest::ToolLibrary` opens or focuses `Tab::Tools`, while
  `Show(Properties)` brings the selected tool's settings into view even
  after Properties was closed or another tab became active. Repeating the
  request leaves the tab open. Adding a `Panel::Tools` presence field and
  changing another State constructor was dropped to avoid a second
  constructor overlap with #1798. These requests are transient view state;
  they add neither persisted project data nor command-history entries.
  Source inspection found no MCP tool or script verb that opens the
  catalog by page name. Their library import/export commands keep the same
  behavior and need no navigation change.

- **Stack on #2001 and preserve its nozzle editing behavior.** That PR
  changes the same tool-editor function. Starting from its head keeps the
  waterjet fields when the editor is shared with Properties. A second
  implementation from main would obscure that overlap and require the
  same integration later.

- **Pin structure instead of pixels.** AccessKit labels and roles check
  that browser selection exposes editable tool settings and that Tools
  opens the catalog independently. Dock behavior follows the existing
  closable panel pattern. A pixel snapshot refresh was dropped because
  this issue changes navigation and ownership rather than a release
  baseline. Reversing the split later would require moving the routing
  and these structural assertions; it needs no project-file migration.

- **Keep catalog filters in temporary UI state.** `Option<ToolKind>` selects
  one of the kinds present in the catalog; `None` selects All types. A
  `String` holds the name filter. A row must match both the selected kind
  and the lowercase name substring. An empty result shows a short message.
  This meets the requested catalog search without changing the project or
  command history. Persisted filters and undoable filter commands were
  dropped because these controls only change which catalog rows are shown.
  Listing only catalog kinds avoids choices with no catalog entries.
  Changing the filter rules later needs UI and structural-test changes,
  but no project migration.

- **Preserve main's waterjet Apply refusal in the shared form.** The merge
  moves the `torch_refusal()` check into `tool_settings::contents`, where
  it disables Apply for an invalid waterjet recipe in both Properties and
  the editor window. Main's check and the form extraction are compatible.
  Taking either side alone was dropped because it would lose the refusal
  or the shared settings surface. The Add tool path stays unchanged.
  A later form split must retain this check on both Apply paths and update
  the structural tests that pin the refusal.
