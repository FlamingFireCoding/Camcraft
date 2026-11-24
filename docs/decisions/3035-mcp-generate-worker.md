## #3035 - the stdio MCP server generates on a worker thread

**What was decided.** `camcraft-cli mcp` (the stdio server, `mcp::serve`)
runs each `generate` call on a worker thread. A reader thread moves the
input lines to the session thread. The session thread keeps the world and
answers every other call while the worker runs, for example `get_job` and
`save_project`. The generate answer goes out when its result lands. Answers
can therefore come out of request order, as JSON-RPC permits.

- The worker gets only the staged job (`StagedGeneration`, #2257). The world
  reads stay on the session thread.
- The result lands through the History, as in the synchronous tool, only if
  `still_current` is true. If the operation or its cached path changed
  while the worker ran, the call is refused by name: "discarded: the
  operation changed while it generated - generate it again". This is the
  same law as the GUI's generation worker.
- `notifications/cancelled` with the generate's `requestId` drops the
  generate. No answer goes out for that id, as the MCP specification says,
  and its result never lands.
- One generate at a time. A second `generate` while one runs is refused by
  name. `open_project` is refused while a generate runs, because it
  replaces the world that the result lands in.
- At the end of input, a running generate still gets its answer before the
  loop ends. A piped script keeps its answers.
- The live GUI bridge (`answer_live`) and the in-process `Server::handle`
  stay synchronous. Their callers answer one line at a time.

**Why.** The issue's agent lost its job: a generate over 900 s blocked the
`save_project` and `get_job` calls behind it, and there was no cancel.
The `StagedGeneration` seam already exists for the GUI, so the change is
in the MCP layer only. No generator changes, and no posted output changes.

**Dropped.**
- A cancel point inside the generators. A cancelled worker runs to its end,
  and the session drops its result. A real stop needs a cancel flag in each
  generator (adaptive, waterline, 5-axis): that is the open run-time part
  of #3035.
- A `cancel_generate` tool. `notifications/cancelled` is the protocol's own
  form, and clients send it at their timeout.
- Refusing every write while a generate runs. `still_current` and the
  operation stamp already keep a stale result from landing as current.

**Not done here (stays open on #3035).** The run time of Rough12, Rest6 and
Waterline6 on the mould core. The staging of a rest operation still
simulates the operations ahead of it on the session thread, so that part
still blocks other calls.

**Cost to change later.** Small. The lane is one module,
`crates/app/src/mcp/background.rs`. A generator cancel flag can be passed
through the staged job later without a change to the protocol.
