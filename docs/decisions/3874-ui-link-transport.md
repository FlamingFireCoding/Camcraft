## #3874 - the UI link transport: a verified peer, not a disclosed token

**What was decided.** The CLI verifies the peer by transport, before
the token crosses. On Windows the window listens on a named pipe
(`\\.\pipe\camcraft-ui-link-<pid>`) with a security descriptor that
admits the window's own user only; the CLI asks the kernel which
process serves the pipe (`GetNamedPipeServerProcessId`) and refuses a
peer that is not the process the discovery file names, before it sends
one byte. On Unix the link keeps the Unix domain socket in the user's
0700 folder. The token stays as the client-side proof that the CLI read
the discovery file; it no longer proves anything about the window. A
`"tcp"` entry is refused by name on every platform, with a diagnostic
that names the reason.

**Why.** The review of PR #3971 showed the token-proof exchange cannot
work on an open port: the hello sends the token to the peer before the
peer proves anything, so a false server that never read the discovery
file still receives the token and can answer with a computed proof.
The stale-file threat of #3874 (a crashed window, a port or pipe name
taken by another process) is not fixed by any exchange that discloses
the secret first. Only the transport can verify the peer: the pipe's
user-only ACL keeps other local users off the pipe, and the
kernel-reported server process id tells the CLI who serves it. The
residual window is pid reuse by a same-user process; a same-user
process can read the discovery file anyway, so it is outside the
threat model of #3874 (a different local user).

**What was considered and dropped.**
- The nonce/proof hello of the first PR cut: dropped, per the review;
  it disclosed the token to the very peer it tried to check.
- Loopback TCP with a better exchange: dropped; no exchange fixes the
  disclosure, and the reviewer named the pipe as the sanctioned route.
- The `interprocess` crate: not taken; it is not in `Cargo.lock` and a
  new crate needs the maintainer's decision. The named pipe is written
  against `windows-sys`, which the PR already introduces; the feature
  list grows (`Win32_System_Pipes`, `Win32_System_IO`,
  `Win32_Storage_FileSystem`, `Win32_Security`,
  `Win32_Security_Authorization`). The maintainer still has to confirm
  these dependency declarations (`libc` on Unix for the pid check,
  `windows-sys` on Windows).
- A pipe read or write without overlapped I/O: dropped; the
  synchronous pipe API has no read or write deadline, and the idle law
  and the hello deadline need one. Every overlapped wait is bounded and
  cancelled on timeout.

**Known limit, recorded.** A named pipe cannot close one direction
only, so the `--ui mcp` bridge's end-of-stdin does not signal the
window the way a socket half-close does. The window's idle drop (5
minutes) or the broken pipe when the CLI process exits ends the
session instead. A tool answer already written is never lost: the
window writes each answer before it reads the next request.

**Cost to change later.** The discovery file keeps its fields; only
the `transport` value changes (`"pipe"`, a name derived from the pid).
An old CLI reading a pipe entry refuses it by transport name; a new
CLI reading an old TCP entry refuses it with the risk-naming
diagnostic. The three other review blockers ride the same cut: one
elapsed deadline over the whole hello exchange, a byte cap that ends
the link at a line that never ends, and a bridge bound that reports a
truncated session as a failure.
