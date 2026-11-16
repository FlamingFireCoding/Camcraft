## #1453 - Explicit tap hand and refusal before posting

- **Store hand directly on the tool as `ThreadHand::Right` or `Left`.**
  A missing field means right-hand, which preserves every existing tap.
  Unknown or malformed values are refused by name. Signed pitch or RPM
  was dropped because those values already have positive-value laws and
  must not encode two meanings. Catalog bookkeeping was dropped because
  its fields explicitly do not affect machining. Changing this choice
  later requires a tool-format migration and updates to the consumers.

- **Refuse left-hand taps in the shared posting preflight.** The refusal
  names the operation and tool before records or NC output are emitted.
  It covers all post choices, including the eight built-in posts that
  implement tapping: Fanuc, Haas, Siemens 840D, Heidenhain, Mazak,
  Tormach PathPilot, Doosan DNM and Fanuc Lathe. Implementing dialect
  cycles in this change was dropped because each needs its own cycle,
  spindle and modal-state contract. Future support must carry the hand
  into the post stream and prove the cycle for each supported dialect
  before relaxing the shared refusal. Right-hand output stays unchanged.

- **Preserve the hand through catalog import, tool JSON, project files,
  tool libraries, editor drafts and undoable tool commands.** The tap
  editor states that left-hand posting is unavailable. Keeping hand only
  in the editor or operation was dropped because another read or edit
  could lose it. Omitting the right-hand JSON field preserves legacy tool
  definitions. New behavior can use the same field without another UI
  state model or a new command.

- **Write project version 62 for saved projects. Use library version 2
  only for exports containing a left-hand tap.** Older readers ignore unknown tool fields, so an
  additive field alone could turn a saved left-hand tap into right-hand.
  The version barriers make those readers refuse the file instead.
  Right-hand-only libraries retain version 1; new readers accept both
  library versions and default older tools to right-hand. Reusing the
  old versions for left-hand data was dropped because it permits silent
  loss of machining meaning. Future format changes must retain these
  barriers or provide an equally explicit compatibility rule.

- **Stack on PR #2001.** It changes shared tool conversion functions,
  including the tool editor and persistence paths. An independent branch
  from main was dropped because it would duplicate those overlapping
  edits. The parent must merge first. A second posting dependency was
  avoided by placing the refusal in the existing preflight rather than
  the record-building function changed by PR #1681. No dialect scripts,
  NC/CL goldens, dependencies or process files change for this issue.
