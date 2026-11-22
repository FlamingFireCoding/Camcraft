## #3509 - Save a project through a temp file and a rename

**What was decided.**

- `project::save` writes through `machine::store::write_atomic`: a temp
  file in the same folder, flushed, then renamed over the target. The
  window, the MCP tool `save_project` and the script call `cam.save` all
  use `project::save`, so one change covers them.
- `project::save` keeps two refusals of the plain write. A folder that
  does not exist is refused by name, because the shared writer creates
  missing folders. A read-only project file is refused by name, because
  on Linux and macOS a rename replaces a read-only file.
- `machine::store::temp_path_for` gives the temp path. `write_atomic`
  uses it, and the tests use it to make a save fail. A path that names
  no file is now refused by name before a folder is made. Before, the
  temp file took the name "machine" and the rename failed.

**Why.** A plain `std::fs::write` truncates the old file first. A write
that stops (a full disk, a lost network share, a stopped app) left a
damaged project. Machine files, preferences, recent files and view
settings already used `write_atomic`.

**Considered and dropped.**

- Move `write_atomic` to a new module: it already has five callers in
  this crate, and a move gives no change in behaviour.
- Let the shared writer create a missing folder for a project, as it
  does for preferences: a typo in an agent's or script's path would make
  folders. That is a different change.
- Fail a save in the tests with read-only folders or open file handles:
  they act differently on Windows and Unix. A folder in the place of the
  temp file fails the same way on all systems.
- Merge the copy of the same steps in `fixture_library/store.rs`: a
  different change.

**Known limits.**

- The new file gets the permissions, owner and ACL of a new file in that
  folder, not those of the old file. Machine files and preferences act
  the same way.
- A project path that is a symbolic link becomes a plain file.
- The folder is not flushed after the rename. Power loss directly after a
  save can keep the old file. It cannot leave a part-written file.
- On Windows, the rename fails when a program holds the old file open
  without delete sharing. The save then reports the error and keeps the
  old file.

**Cost to change later.** Low. The checks are in
`project::wire::write_project`, and the four tests in
`crates/app/tests/project_roundtrip.rs` pin them.
