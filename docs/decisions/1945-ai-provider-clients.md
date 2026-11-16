## #1945 - AI provider HTTP clients, keychain keys and offline-only mode

**What was decided.** A new `camcraft_app::ai` module holds the provider
transport: `endpoint` (URL parse, loopback test, the offline and https laws),
`presets` (Anthropic, OpenAI Chat, OpenAI Responses, Gemini over https;
Ollama, Unsloth Studio, LM Studio and llama-server on loopback), `wire` (the
request path and body per provider, echoing tool-call envelopes so reasoning
items and Gemini thought signatures survive), `probe` (Ollama `/api/show`
capabilities), `secret` (an opaque `ApiKey`, environment first, then the
keychain), and, native only, `http` (the client) and `keychain`. The AI Chat
dock uses it through `transport_http::HttpTransport`; the browser build keeps
the offline-only default transport and links no socket or keychain code.
`prefs.json` gains `aiOfflineOnly` (written only when on), and the dock's
"Offline only" switch is saved there.

**New third-party crates (need the maintainer's decision).** `ureq 2.12`
(default features off, `tls`: rustls + ring + webpki-roots) for HTTP(S), and
`keyring 4.2` (macOS Keychain, Windows Credential Manager, Secret Service
over the zbus 5 already in the lock) for the keys. `url 2.5` was already in
the lock and is now a direct dependency. Nothing in the workspace spoke TLS
or reached an OS credential store, so neither could be done with existing
crates; hand-writing TLS is not an option.

**Why these.** ureq is blocking (the chat worker is a plain thread; no async
runtime enters the tree) and its 2.x line reads no proxy from the
environment by default, so a loopback call cannot be routed off the machine
by an `HTTP_PROXY`. rustls with ring avoids OpenSSL on Linux and the
aws-lc-rs CMake build. keyring 4's Linux store is pure Rust over zbus, so no
`libdbus` build package is needed on the release runners.

**Offline-only is enforced in the client, twice.** `endpoint::admit` refuses
a non-loopback host before anything resolves; the agent's resolver then
refuses any resolved address that is not loopback (a hosts-file `localhost`
pointing elsewhere). No redirect is followed: a local server could otherwise
bounce a call to a remote host. Plain `http://` off the machine is refused
whether or not a key is sent - cloud providers are https only.

**Unsloth Studio and llama-server bases.** The issue names
`http://127.0.0.1:8888` for Unsloth Studio; the base is
`http://127.0.0.1:8888/v1`, the OpenAI-compatible mount every other local
runtime here uses. llama-server's documented default port is 8080; its label
says to start it with `--jinja` for tool calls.

**A model without tools refuses, it does not silently degrade.** When an
Ollama model in the native-tools preset lists no `tools` capability, the
step is refused by name, pointing at the Prompt-JSON and Q&A presets. Falling
back inside the transport would send a request without the prompt-JSON
context (the transport has no project snapshot) and the reply would not parse.

**Considered and dropped.** `reqwest` (pulls tokio and hyper); `native-tls`
(OpenSSL headers on Linux); keyring's `dbus-secret-service` store (vendored
libdbus C build); storing keys in `prefs.json` encrypted (the issue forbids
any key there); an MCP tool to set a key (an agent should never handle a
user's secret).

**Cost to change later.** Low for presets and wire shapes (data and pure
functions with loopback tests); swapping the HTTP crate touches only
`ai/http.rs`.
