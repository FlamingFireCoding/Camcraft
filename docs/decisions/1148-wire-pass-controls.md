## #1148 - Edit Wire Contour passes and slug retention on the operation page

**What was decided.** The Wire Contour Passes tab edits the ordered skim
list, each skim's machine condition and reverse flag, the holding tabs and
the captured-piece declaration. The list supports add, remove and reorder.
Tab count and width remain explicitly stated optional values; positions
and the cut-off condition have their own controls. Every field edits the
existing operation draft. Apply sends `EditOperationParams`, so an edit
lands as one undoable command.

**Why.** These parameters already exist in the project file and command
door. The operation page must expose the same choices without inventing a
second validation path. Condition choices come from the mounted machine.
An unknown condition ID stays visible until the user changes it, allowing
the existing command refusal to name it. This work changes no generator
or validation rule.

**Considered and dropped.** Direct world writes would bypass command
validation and undo. Guessing a machine condition, tab count or tab width
would turn an unstated machining choice into an apparent user decision.
A new command would duplicate `EditOperationParams` without adding a new
operation or undo boundary. All three alternatives were dropped in favour
of the existing draft and Apply workflow.

**Cost to change later.** Control arrangement or editing behaviour can
change in the wire page's UI helpers. The stored parameter shape, command
door and generator need no migration for such a change.

**Review evidence (#1765).** Capture the actual egui operation page before
and after the change with the same populated wire draft. Store these review
images outside release snapshot goldens and link them from the PR body at
the pushed commit. This gives the reviewer stable visual evidence without
refreshing release snapshots or adding production behavior. A small PNG and
Markdown-reference pin detects missing, corrupt, or duplicated evidence.
