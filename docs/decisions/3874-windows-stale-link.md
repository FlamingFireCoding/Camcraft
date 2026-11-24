# #3874 - Windows UI link stale file check and token proof

On Windows, the UI link discovery file (`<pid>.json`) stores the TCP loopback port and token. If a window crashes, a stale port could be bound by another process. Prior to this change, the CLI sent the token to any listening socket without requiring proof of token ownership, and read unbounded answer lines.

## Decision

1. **Mutual Token Proof Handshake:**
   - The CLI generates a fresh nonce for each session.
   - The CLI sends `hello_line` carrying `token_proof(token, nonce, "client")` and `nonce` without sending the raw token in plaintext.
   - The window verifies the client proof and replies with `token_proof(token, nonce, "server")`.
   - The CLI rejects any peer that fails to return the expected token proof, sending zero tool calls to unverified servers.

2. **PID Liveness Check:**
   - Before attempting connection, `link::entries` verifies that `entry.pid` is alive via `pid_alive`.
   - On Windows, `pid_alive` probes the process handle via `OpenProcess`. If the process has exited, the stale `.json` file is removed immediately.

3. **Bounded Answer Reading:**
   - `link_client::call` bounds read buffer lines to 16 MiB (`MAX_LINE`) preventing memory exhaustion from malformed server streams.
