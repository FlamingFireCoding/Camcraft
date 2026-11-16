## #1946 - In-app AI chat dock panel, background worker, and offline UI tests

**Decided:** Add a closed-by-default `Tab::AiChat` / `Panel::AiChat` dock pane
opened from the View door beside `Tab::Properties` (`layout/mod.rs`), switching
`LayoutRequest::Toggle` to `show(dock, tab_of(p))` so newly opened tabs beside
Properties activate immediately. Keep panel state (`AiChatState`) isolated
from the CAM document on UI `State`, reset/shut it down on New, successful Open,
and `Shell::shutdown`, and pass `ai_proxy.send_event(())` via `State::set_ai_wake`
so background events wake the `Wait` event loop.

Run each chat turn on a background `std::thread` (with a `wasm32` stub) driven by
a synchronous `ChatTransport` boundary (`DefaultTransport` enforces offline policy
until the live HTTP transport from #1945 is plugged in). Execute MCP tool calls from
the worker over `camcraft_app::mcp::Inbox::channel()` (`State::ai_mcp_tx` /
`State::ai_mcp_inbox`), drained alongside the TCP `mcp` inbox in `mcp_host::pump`
via a shared `answer_request` helper so every tool execution runs on the UI main
thread inside `State` and updates `doc_version` / undo `History` consistently.

Enforce safety gates in UI/worker code:
- `Offline only` refuses `EndpointKind::Cloud` presets in both `AiChatState::submit`
  and `check_offline_policy` before any transport call.
- `FallbackMode::Native` pauses sensitive file/G-code tools (`post`, `open_project`,
  `save_project`, `import_model`, `import_tool_library`, `export_tool_library`) behind
  an inline Allow/Deny prompt; `FallbackMode::PromptJson` gates every tool outside
  the read-only allowlist behind Allow/Deny after schema validation; `FallbackMode::ReadOnly`
  displays a banner and disables tool parsing and MCP dispatch completely.
- `Attach selection` defaults to `false` and captures `SelectionSnapshot` only
  when explicitly checked. Disable `Attach screenshot` with the visible message
  “Viewport screenshot capture is unavailable.” until actual viewport capture
  is wired; a solid-color fallback does not represent the model or toolpath.
- Bind `Undo Turn` to globally unique History command identities and the document
  identity, in addition to its depth span. Replacement at the same stack depth
  must refuse undo just like a later edit.
- Give each submitted turn a fresh inbox. Stop the worker and discard its inbox
  on New or successful Open, invalidate prior turn undo records, and reject
  queued calls when the active turn is cancelled before UI dispatch.

**Why:** Routing in-app worker tool calls through `Inbox::channel()` avoids a
parallel command interpreter or off-thread `World` mutation and keeps document
changes atomic and undoable. Separating `ChatTransport` lets offline UI and
safety tests exercise View-door layout, MCP round-trips, cancellation, confirmation
prompts, fallback banners, opt-in attachments, and Turn Undo deterministically in
`egui_kittest` without network I/O.

**Dropped:** Spawning commands off-thread or duplicating TCP JSON-RPC over loopback
for in-app chat would add socket framing overhead and race with UI state. Allowing
`Undo Turn` to pop across later manual edits would silently undo the user's own
work. Depth alone was rejected because ordinary Undo followed by a manual edit
can replace the AI entry at the same depth. Reusing the inbox was rejected
because old queued calls could reach a replacement document or a later turn.
Sending a solid-color PNG was rejected because valid image encoding does not
prove viewport capture. Storing chat history
inside `.cam` project files was omitted so conversations never pollute part geometry
or git/fixture state.

**Cost to change later:** Wiring live HTTP providers (#1945) implements
`ChatTransport` and swaps it via `AiChatState::set_transport` without changing
panel layout, `Inbox::channel()` dispatch, or safety gates. Changing the native
confirmation set or read-only allowlist edits `requires_confirmation` /
`is_read_only_tool` in `crates/ui/src/egui_app/ai_chat/transport.rs`.

Actual screenshot attachment will require a viewport capture callback and tests
that inspect model/toolpath image content before the control can be enabled.
