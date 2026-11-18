## #2358 - The Setup page lists the user's machines

**What was decided.**

- The Machine page's "Library" list reads the Machines pane's catalogue
  (`machines_pane::refresh`, `library::enumerate_with`) instead of
  `posting::MACHINES` alone. It shows the user's own `.mch` files first,
  then a separator, then the shipped machines. A file the document has
  mounted from elsewhere ("Load other") is a file entry too, so the combo
  names it instead of "other".
- A file entry reads "name (file name)". A refused file adds
  "- refused: <first refusal>" and its row is disabled.
- A shipped pick still queues `LoadBuiltinMachine` by name; a file pick
  queues `UseCatalogueMachine(MachineSource::File(..))`, the Machines
  pane's undoable command. No command or project field is new.
- Opening the list re-reads the catalogue, so a file saved outside the app
  shows up without a remount.
- "Load other" opens its file dialog in the user machines folder when the
  folder exists (`machines_pane::dialog_start`), else at the dialog's own
  default.
- The test shell (`ShellUi::blank`) sets no user machines folder, so no UI
  test reads the tester's real folder.

**Why.** A machine saved from the Machines pane could be found again only
through View > Machines. A user who made a machine most likely wants to
use it, so it comes first. Reusing the pane's catalogue and command keeps
one source for the list and one undoable door for the mount.

**Considered and dropped.**

- Shipped machines first, as `library::enumerate` returns them. Dropped:
  the user's own machine is the likely pick.
- Routing shipped picks through `UseCatalogueMachine` too. Dropped: it
  changes a working path for no gain in this issue.
- Reading the folder on each frame. Dropped: frame cost; the catalogue is
  cached on the pane and re-read on remount or when the list opens.

**Cost to change later.** The order and labels live in
`machine_page::library_row` and `library_label`; the folder lookup in
`machines_pane::user_dir` and `dialog_start`. A different order or label
is a local change to those and to `tests/setup_user_machines.rs`.
