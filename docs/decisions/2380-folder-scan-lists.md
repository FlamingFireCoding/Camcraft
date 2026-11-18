## #2380 - The machine and post lists follow their folders

**Decided.**
- The registry tables (`BUILTINS`, `MACHINES`) stay. They map a name (which
  documents store) to a file. They no longer decide what a list shows.
- A new scan (`camcraft_post::data_scan::scan_folder`, beside
  `data_dir::list_files`) reads the folder names (`.cps`, `.mch`). It never
  opens a file. `app` re-exports it. The cache that binds it to the
  `BUILTINS` and `MACHINES` tables and to the process folders is in
  `posting_files.rs`, beside those tables: it reads the folders at the
  first use and at each Refresh (review of PR #2414).
- A list shows a table entry only if its file is in the folder. A file that
  the table does not name is listed as a file entry (`SelectPostFile`,
  `LoadMachineFile`, `MachineSource::File`): the commands that exist.
- A malformed file is listed, with its own refusals. It does not hide others.
  An unreadable folder lists nothing and logs the reason.
- A document that names a missing post or machine is refused by name
  as before (`builtin_post_text`, `builtin_machine_text`, #2319).
- The machine page, the machines pane, the post picker and the MCP
  machine-name list use the same scan. Refresh buttons call `posting::rescan`.

**Why.** The old lists came from the tables, so a removed file kept its
entry (then a file-not-found error) and an added file never showed.

**Dropped.**
- Naming a post or machine by a new library name for extra files: the file
  path is their identity, and no new command was needed.
- A scan for every frame: the scan is cached and read again only at Refresh.
- Dropping the text caches: a file that was read once and removed later still
  posts from its cached text until restart. A later issue can change this.

**Cost to change.** Low: one `camcraft-post` module, the cache in
`posting_files.rs`, and callers that read `*_scan()`.
