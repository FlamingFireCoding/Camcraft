## #3675 - A recording build writes its JSON to a file, not to a pipe

**Cause.** The gate's per-crate prebuild (`debug build: <crate> (--no-run)`)
records its test executables from cargo's JSON on stdout. `direct::cargo`
read that stdout through a pipe, in a reader thread, and joined the thread
after cargo exited. A process that the build starts and that outlives the
build inherits cargo's stdout. The sccache server is such a process: the
first `rustc` wrapper call starts it when no server runs. On Windows the
server keeps an inherited copy of the write end of the pipe, so the reader
got no end of file until the server stopped (its idle timeout, 10 min by
default). During that join no watchdog runs: the watchdog stops when cargo
exits. So the gate waited after a green build, and the wait depended on
the state of the sccache server, not on cargo. The issue named a cargo
"rmeta handoff". This worker did not see the reported wedge on its own
machine (cargo 1.100.0, a shared sccache server that already ran). The
pipe hold is the one wait of the prebuild path that is shown (test below)
and that depends on the sccache state. It is the probable cause, not a
confirmed one. A second, private sccache server (other port) to start the
server cold failed its compiles ("connection was forcibly closed"), so
that path was not run end to end.

Readings (this machine, cargo 1.100.0, Windows 11):

- The pinning test with the old `direct.rs`: the fake cargo exits at once
  and leaves a descendant that sleeps 120 s on its stdout. The step took
  120.03 s. The 60 s watchdog did not fire.
- The same test with the fix: the step ends at once (11 xtask watchdog and
  direct tests pass in 7 s).

**Decided.** The JSON goes to a file in the temporary directory (one name for
each process and build). The watchdog counts the file size from the open
handle as output, as it counted the pipe bytes. When cargo exits, the gate
reads the file and deletes it. A descendant that still holds the file only
delays the deletion. It cannot hold the gate.

**Dropped.**

- A bounded join of the reader thread (wait N seconds after the exit). It
  keeps a thread blocked on the pipe and a time constant to tune. The file
  needs neither.
- Start the sccache server before the gate (`sccache --start-server`). It
  fixes only the sccache case. Any other long-lived descendant (a build
  script that starts a daemon) still holds the pipe.
- A fallback from a wedged prebuild to the per-binary path. With the cause
  removed, the prebuild no longer waits after cargo exits. A real silent
  cargo still ends as a STALL row that names the step (#3303).

**Cost to change later.** Small: one function, `direct::cargo`. The
temporary file costs one write and one read of the JSON (a few MB at most).
