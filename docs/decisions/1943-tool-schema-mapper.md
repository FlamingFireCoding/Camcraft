## #1943 - Map the live MCP tools to provider formats

**Decided:** Read `mcp::tools::list()` for each request. Map this table to
Anthropic Messages, OpenAI Chat Completions, OpenAI Responses, and Gemini
`generateContent`. Keep the mapper and the call/result adapters independent
of network transport and tool execution.

Set OpenAI `strict` to `false`. This preserves optional MCP arguments and
schemas with typed `additionalProperties`. For Gemini, change type arrays
to `anyOf` and remove `$schema` from schema locations. Keep literal values
in `enum` and `default` unchanged.

Preserve call IDs and MCP errors in provider results. The caller keeps the
original response for reasoning items and Gemini thought signatures.

Accept one local JSON proposal. Validate its tool name and arguments against
the live MCP schema. Keep its fields private in `PendingCall`. Require
explicit user confirmation for each tool outside an explicit read-only
allowlist. Require confirmation for `detect_pockets`, because its optional
`create` argument can add an operation.

Select prompt JSON mode when native tool calling is off and prompt JSON is
available. Otherwise, select read-only Q&A mode. Its context contains the
project data and an empty tools list.

**Why:** One live table keeps provider schemas current when MCP tools change.
The adapters prepare calls without changing the existing command and safety
checks. Named validation errors prevent invalid proposals from reaching
confirmation. New tools require confirmation until their read-only behaviour
is explicitly accepted.

The validator implements the schema vocabulary that the live table uses.
It refuses unsupported validation keywords by name. It applies literal
constraints: `set_work_material` currently permits a null type but has an
enum without null. This change preserves that schema and refuses null.

**Dropped:** Copied provider tool tables would drift from MCP. OpenAI
`strict: true` would require schema changes and reject typed additional
properties. A new validation dependency is unnecessary for the current
schema vocabulary. Network transport, provider sessions, and a confirmation
UI are outside this offline mapper change.

**Cost to change later:** Add a validator branch and a pin for each new
validation keyword. Extend the read-only allowlist only after checking the
tool handler. A future transport must present the exact pending proposal,
pass the user's confirmation, preserve response history, and dispatch
through MCP. Correct the material enum in a separate change if its schema
must permit null.

Public format references:

- [Anthropic tool calls and results](https://platform.claude.com/docs/en/agents-and-tools/tool-use/handle-tool-calls)
- [OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling)
- [Gemini FunctionDeclaration](https://ai.google.dev/api/caching#FunctionDeclaration)
- [JSON Schema validation](https://json-schema.org/draft/2020-12/json-schema-validation)
