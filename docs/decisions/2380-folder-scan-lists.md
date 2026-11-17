## #2380 - The machine and post lists follow their folders

**Decided.**
- The registry tables (`BUILTINS`, `MACHINES`) stay. They map a name (which
  documents store) to a file. They no longer decide what a list shows.
- A new scan (`posting_scan.rs`) reads the folder names (`.cps`, `.mch`) at
  the first use and at each Refresh. It never opens a file.
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

**Cost to change.** Low: one module, and callers that read `*_scan()`.
