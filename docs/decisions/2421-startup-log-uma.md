## #2421 - Keep desktop startup diagnostics without a terminal

**Decided.** The desktop initializer appends plain-text diagnostics to
`camcraft.log` in the existing platform configuration directory. It uses a
temporary `camcraft` directory when no configuration directory resolves.
Each completed record calls `write_all` and `sync_data`. The file keeps the
default INFO-level diagnostics independently of terminal verbosity. The
CLI keeps its existing stderr-only initializer.

The logger preserves the previous panic hook and records panic messages
with forced backtraces. Returned startup failures also get forced
backtraces. Failure dialogs and terminal messages name the log location
and the backend flags to try. GPU records include the adapter name,
vendor, device, backend, driver and driver information. Enumeration and
candidate records reach the log before device creation and the test
frame; the selected-adapter record follows successful bring-up.

**Why.** A desktop launch may have no terminal, and `process::exit` or a
driver hang can prevent shutdown cleanup. Synchronous append gives each
startup record a chance to reach disk immediately and retains evidence
from the previous run. Terminal filtering must not hide the GPU path or
failure that a report needs. Reusing the configuration resolver and the
existing tracing dependencies keeps this change small.

**Considered and dropped.** An asynchronous writer that flushes only at
shutdown cannot preserve these records during a hard lock. Truncating on
startup would erase the previous run. File logging for the CLI would add
filesystem side effects to commands such as help. New logging dependencies
are unnecessary. Software graphics mode, changes to automatic backend
fallback and project-file command-line handling need separate work.

**Cost to change later.** Retention limits or rotation can replace the
append policy inside the desktop logger without changing project files or
CAM commands. A different log destination requires updating the path
resolver, failure messages and subprocess fixtures. Headless tests cover
persistence and failure handling; Windows dialogs, physical GPU failures
and survival of a full system lock remain unexercised.
