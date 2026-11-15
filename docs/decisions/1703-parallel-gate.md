## #1703 - Run independent release-gate lanes together

**Decided:** Prebuild the debug test targets and the release heavy targets before
running release-gate suites. Run independent suites in waves of at most four,
mixing debug and release steps. Keep app, kernel, UI, build and clippy steps alone.
Keep invoking Cargo for each suite after the prebuilds, with the existing working
directories, libtest arguments and separate numbered logs.

**Why:** This preserves the existing suite selection and diagnostics while
overlapping test execution. App and kernel contain import watchdogs; keeping their
whole suites alone avoids a fragile list of individual timed tests. UI suites can
open windows and must also run alone. Debug and release use different profile
directories. A temporary probe with four prebuilt two-second test runs measured
8.086 seconds serially and 2.052 seconds concurrently. Cargo briefly held package
and build locks, then released them before test execution. This supports using the
existing Cargo commands rather than writing an artifact parser and test launcher.

The gate reports each lane's duration, total elapsed time and the sum of lane wall
times. The sum is an estimate of serial cost under the current load, not a measured
serial release benchmark. The full measured comparison is recorded below.

**Also decided:** Compare corresponding removed and added Cargo.lock lines after
normalizing CRLF line endings. Permit unchanged lines and version-line changes;
continue refusing changed names, checksums and other content.

**Considered and dropped:** Running every suite concurrently risks watchdog and
window failures. An unbounded worker pool risks exhausting machine resources.
Splitting individual app/kernel binaries can improve utilization later but needs
a reliable inventory of timed suites. A custom Cargo artifact parser duplicates
Cargo's target selection. Splitting work between uma and m3 adds coordination,
artifact transfer and failure handling beyond the optional multi-host request.

**Cost to change later:** The wave planner and its tests own concurrency and
isolation policy. Finer isolation or a queue that immediately fills freed slots
can change that module without changing suite commands. Multi-host execution
would additionally need a runner protocol and combined checkout/log validation.


### Full release-gate comparison (2026-11-15 UTC, PR #1790 review)

**Revision and machine:** Both measured runs used the unchanged code and tracked
working tree at `31dc65818dbb341c6ba199979a07e87891f84a21`, on `uma`: Intel Core
i7-1360P, 12 cores / 16 logical CPUs, 61 GiB RAM, x86_64 Linux
`7.0.0-43-generic` (Ubuntu), `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
No GitHub runner or second machine was used. Both runs retained the plan's
Cargo `-j 6` and libtest `--test-threads=6` settings.

**Cache and order:** First run the complete parallel release gate as a warmup:
5086.11 seconds, all 169 steps attempted, three failed steps. Its new worktree
initially had no product build artifacts; the existing shared sccache was enabled.
Then measure serial, followed by parallel, in that same worktree. Retain both
`target/debug` and `target/release` and the shared sccache between runs. The
inherited `RUSTC_WRAPPER` selected sccache and `SCCACHE_CACHE_SIZE` was `40G`;
`CARGO_TARGET_DIR` was unset. No cache was cleared and no source, manifest,
compiler setting or dependency changed. OS page caches, temperature and background
machine load were not controlled. These are one pair of warm-cache measurements,
not a cold-build comparison or a statistical speedup claim.

| Full run | Elapsed wall time (`/usr/bin/time -p`, real) | Steps attempted | Failed steps | Exit |
| --- | ---: | ---: | ---: | ---: |
| Serial command replay | 3982.32 s (66.37 min) | 169 | 3 | 1 |
| Parallel release gate | 3851.03 s (64.18 min) | 169 | 3 | 1 |

The measured parallel run took 131.29 seconds less (3.30%, 1.034x). This compares
actual full-run wall times, not summed lane times. The standalone app and kernel
barriers and the long `shared_face_bodies` heavy test remain part of both runs.
The latter alone took 1473.21 seconds in the serial run.

**Results and limits:** Both full measured runs completed every planned command,
including both prebuilds, all debug suites, every release heavy command, clippy and
both UI snapshot commands. Both had the same three failed steps:

- Clippy refused existing dead code in `crates/ui/src/egui_app/app_icon.rs`:
  `PNG_256` at line 12 and `taskbar_icon` at line 40, under `-D warnings`.
- The UI snapshot step failed `egui_app::tests::egui_shell_snapshot`,
  `egui_app::tests::egui_shell_snapshot_light` and
  `egui_app::tests::tool_editor_snapshot`.
- The GPU documentation snapshot step failed
  `docs_images_snapshot_match_the_rendered_steps`.

The other 166 commands passed in each measured run. No snapshots were updated.
These are timings of full gates with identical failures, not passing release-gate
verdicts; they do not establish the time for a fully green release. Fixing those
unrelated UI findings was outside this review request.

**Commands and serial method:** The release runner has no serial switch. Replay
every exact Cargo command printed by its full dry-run sequentially with the
printed working directory, using an external Python harness; do not change the
source or revision to disable concurrency. The dry-run contains 169 commands.
The harness below was used for the serial measurement; its per-step logs and JSON
retain each command, exit status and elapsed time. It stops on a failed prebuild,
as the parallel gate does, but neither measured prebuild failed.

From the worktree root, before editing this documentation:

```sh
mkdir -p gate-logs/pr1790-benchmark
cargo xtask gate release --dry-run > /tmp/pr1790-release-plan.txt 2>&1
/usr/bin/time -p -o gate-logs/pr1790-benchmark/warmup.time cargo xtask gate release > gate-logs/pr1790-benchmark/warmup.log 2>&1
```

Save this harness as `gate-logs/pr1790-benchmark/serial.py`:

```python
import os, pathlib, re, shlex, subprocess, time, json
root = pathlib.Path.cwd()
plan = pathlib.Path('/tmp/pr1790-release-plan.txt').read_text()
commands = []
for line in plan.splitlines():
    m = re.fullmatch(r'     \(cwd ([^)]+)\) cargo (.+)', line)
    if m:
        cwd = root / 'crates/ui' if m[1] == 'ui' else root
        commands.append((cwd, shlex.split(m[2])))
assert len(commands) == 169, len(commands)
before = subprocess.check_output(['git', 'rev-parse', 'HEAD'])
start = time.monotonic()
results = []
for i, (cwd, args) in enumerate(commands, 1):
    t = time.monotonic()
    logfile = root / 'gate-logs/pr1790-benchmark' / f'serial-{i:03}.log'
    print(f'{i}/169 cwd={cwd.relative_to(root)} cargo {shlex.join(args)}', flush=True)
    with logfile.open('w') as log:
        code = subprocess.run(['cargo', *args], cwd=cwd, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT).returncode
    result = dict(step=i, args=args, cwd=str(cwd.relative_to(root)), seconds=time.monotonic()-t, code=code)
    results.append(result)
    print(result, flush=True)
    if '--no-run' in args and code:
        break
elapsed = time.monotonic()-start
assert before == subprocess.check_output(['git', 'rev-parse', 'HEAD'])
summary = dict(head=before.decode().strip(), elapsed_seconds=elapsed, results=results)
(root / 'gate-logs/pr1790-benchmark/serial.json').write_text(json.dumps(summary, indent=2))
print(f'wall {elapsed:.3f}s; {len(results)} steps; {sum(r["code"] != 0 for r in results)} failed', flush=True)
raise SystemExit(any(r['code'] for r in results))
```

Then run the two measured commands, in order, retaining their logs:

```sh
/usr/bin/time -p -o gate-logs/pr1790-benchmark/serial.time python3 gate-logs/pr1790-benchmark/serial.py > gate-logs/pr1790-benchmark/serial.log 2>&1
/usr/bin/time -p -o gate-logs/pr1790-benchmark/parallel.time cargo xtask gate release > gate-logs/pr1790-benchmark/parallel.log 2>&1
```

**Coverage check:** Compare the `(cwd, Cargo arguments)` multisets from
`serial.json` and the measured parallel output: exactly 169 commands in each,
with equal multisets. Normalize workspace-root cwd to `.` and UI cwd to
`crates/ui`; sort the strings `cwd + "\t" + arguments joined by spaces`, join
with newlines and add one final newline. The SHA-256 of that canonical manifest
was `d9277549caf847a2bdc052ef605c58ab973ef606c6581e52aac31d8fb5f0212b`.
HEAD and the tracked working tree stayed unchanged during the runs; the parallel
gate did not report VOID. Raw evidence is retained locally in
`/tmp/camcraft-pr1790-benchmark`, including the warmup and parallel per-step gate
logs, all serial per-step logs, the dry-run plan, harness and measured summaries.
The commands, method and results above are self-contained for re-running the
comparison without those local files.
