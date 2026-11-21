## #1873 - Core cadmpeg runner, version check and temp-dir sandbox

### 1. No `cadmpeg formats --json` probe; the exit status of `check` and `convert` is the capability test

**What was decided.** The runner does not call `cadmpeg formats`. It runs
`--version`, then `check --json <input>`, then
`convert <input> -o <out> --to step --reject-lossy=export`. A build that
cannot read the input format or cannot write STEP fails one of these with
a nonzero exit status, and the import is refused with cadmpeg's own last
error line.

**Why.** The issue asks for a `cadmpeg formats --json` probe. cadmpeg
0.6.12 has no such option: `formats` takes no arguments and prints a text
table (the first worker on this issue found this; the CLI definition at
tag commit `9d7592c64` confirms it). cadmpeg's own source says that the
table is for a terminal reader and is not for a caller to parse. The
probe was only an early diagnostic. Safety does not depend on it: `check`
and `convert` give the verdict, and the STEP importer's guards run on the
result.

**Dropped.**
- Parsing the text table of `cadmpeg formats`. Its layout is not a
  contract, so a new cadmpeg could silently break the import.
- A JSON schema of our own that only the stub answers. It proves nothing
  about cadmpeg.
- Waiting for an upstream JSON contract (the first worker's
  recommendation). It blocks #1875, #1876 and #1877 on a third party. The
  probe can be added later without a change to the pipeline.

**Cost to change.** One more step in `cadmpeg::to_step` before `check`.

### 2. The check verdict is the exit status, and the JSON report is read as a second guard

**What was decided.** A nonzero exit status of `check` refuses. With exit
status 0, the runner still reads the JSON on standard output. It refuses
when a `findings` list at any depth has an entry with severity `error` or
`blocking`, and when the output is not JSON.

**Why.** cadmpeg's README states that `check` exits 1 on `error` and
`blocking` findings, so the exit status is the documented contract. The
report layout is versioned by cadmpeg and can move; the scan by key name
does not depend on the path of the list. A report that the runner cannot
read cannot clear a part (Prime Directive).

**Dropped.** A typed model of the report. Its schema is not ours and no
real report was available in this environment (cadmpeg is not installed).

### 3. The child writes to files in the private folder, not to pipes

**What was decided.** Standard output and standard error of the child go
to two files in the private folder. One loop looks at the child, the
clock and the file sizes every 15 ms. A child that passes the time limit,
prints more than 16 MiB, or grows the STEP file past the size limit is
stopped. The folder is removed by a `Drop` guard.

**Why.** With pipes, a reader thread blocks for as long as any process
holds the pipe open, also after the child is stopped. With files there is
no reader thread, and "bounded buffers" is a size that the loop reads. No
new crate is needed.

**Dropped.** Stopping the whole process group of the child. That needs
`libc` (`kill` on a negative pid) or `unsafe`; the issue forbids new
dependencies. Only the direct child is stopped. cadmpeg is one process.

### 4. Refusals use `CommandError::Needs`; an unusable binary uses `CommandError::Invalid`

**What was decided.** A missing program, an old version, a wrong stated
path, a failed `check`, a failed `convert`, a time-out and an oversized
output are `Needs`: plain text that the user can act on. A program whose
`--version` text states no version is `Invalid`. An unqualified native
extension keeps `CommandError::UnsupportedType` with the extension, as
IGES had.

**Why.** `Needs` shows the text with no "invalid parameters:" prefix
(#1105). `UnsupportedType` keeps the existing IGES tests and UI status
text unchanged.

### 5. The qualified table is empty, and the stated path is used alone

**What was decided.** `import::native::QUALIFIED` is empty, so no import
reaches the runner in a shipped build until #1875 adds a format. All
extensions that cadmpeg lists as inputs (`.prt` included) are refused by
name. When `cadmpeg_path` is set and is not a program file, the import
refuses; it does not fall back to `PATH`.

**Why.** The issue says to keep the table empty. A silent fallback would
run a different program than the one the user named.

**Also.** `ImportModel::within` reads the preferences file for the
stated path (`Settings::from_prefs`) because the command constructor has
no settings argument; #1876 owns the UI field. Until then the UI keeps a
hand-written `cadmpegPath` when it saves the preferences.

**Not done here.** The `wasm32` refusal text and the UI wording belong to
#1876. In this PR the runner module is compiled only off `wasm32`, and a
qualified extension on `wasm32` or in `ImportModel::from_bytes` refuses by
name.
