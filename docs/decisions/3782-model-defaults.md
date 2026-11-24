## #3782 - Select installed local models and maintain cloud defaults

**What was decided.** An empty local model field selects an installed model automatically. The transport reads `GET /api/tags`. It checks each candidate with `POST /api/show` and selects the first model with the `tools` capability. All local presets use this selection rule. Each preset keeps its existing fallback mode. A model name that the user enters stays explicit.

If no installed model supports tools, the diagnostic gives `ollama pull qwen3:8b` and links to <https://ollama.com/library>.

Cloud defaults live in one table in the app layer. The defaults are `claude-sonnet-5-5`, `gpt-6.2-sol` for both OpenAI endpoints, and `gemini-4-hydrogen`. These IDs were checked against the official model pages on 2026-11-24:

- <https://platform.claude.com/docs/en/models/overview>
- <https://developers.openai.com/api/docs/models/gpt-6.2-sol>
- <https://aistudio.google.com/docs/models/gemini-4-hydrogen>

Each row in the shared `CLOUD_DEFAULTS` table includes its retirement status. An offline test rejects a default when its row marks it as retired. The table and its status fields need maintenance when providers change model availability.

**Why.** An installed model can work on the first Send without a model name that the user must enter. The capability probe excludes models that cannot call tools. One table prevents separate defaults from drifting between callers. Recorded responses keep the tests independent of installed models, provider accounts, and network access.

**What was dropped.** The app does not download a model automatically. A download can consume substantial disk space and bandwidth. The app does not silently replace an explicit model name. That would hide the user's choice. A fixed local model name was dropped because the selected machine might not have that model. Live cloud checks were dropped because they require network access and can require API keys.

**Cost to change later.** A different automatic selection policy changes the candidate selection and its recorded-response tests. Provider changes require an update to the shared defaults table or its retirement status fields. A provider wire-format change can also require transport work. These changes do not require a project format migration.
