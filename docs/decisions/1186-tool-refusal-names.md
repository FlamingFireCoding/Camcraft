# #1186: a refused tool is named, with the rule it breaks

- **Chosen:** a new `Tool::why_not_sane() -> Option<String>` (`crates/tools/src/tool_sanity.rs`) returns the first rule a tool breaks, in plain words, such as "turning surface speed 0 m/min is outside 0..2000 m/min (it must be over 0)" or "corner radius 100 mm is more than half the 6 mm diameter". `Tool::is_sane` is now `why_not_sane().is_none()`, and `TurningData` gets the same pair.
  - Why: one source of truth. The words and the verdict cannot drift apart, because `is_sane` reads only the function that writes the words.
  - Dropped: a second, parallel `why` function beside the old boolean. Two copies of one law drift.
  - Moving the body also takes about 100 lines out of `tools.rs` (714 -> 614).
- **Verdict unchanged, proved:** `tool_sanity_tests::the_named_law_gives_the_old_verdict_everywhere` keeps the old `is_sane` and `TurningData::is_sane` bodies verbatim as an oracle. It sweeps every catalog tool, retyped as each of the 17 kinds, with each of the 14 numeric fields set to 13 edge values (0, negative, the 60/180-degree bounds, over 500, NaN, infinity), plus flute counts, orientation codes and turning data. The new law agrees with the old on every case.
- **Merge of main (2026-11-16):** main added the waterjet, electrode and thread-mill kinds, `pitch_min`, and the catalog and cutting-row refusals to `is_sane` after this branch started. Each new rule now has its words in `why_not_sane`, and the oracle holds main's `is_sane` body.
  - Chosen: the electrode and thread-mill rules are written out in `tool_sanity.rs` with their own words.
  - Dropped: a call to `electrode::refusal`. It does not refuse a Y size over 500 mm, which `electrode::is_sane` refuses, so the verdict would change.
- **Chosen:** the four places that refused a bad tool now say `tool '<name>': <rule>`:
  - the project loader (`project/validate.rs`, the issue's case, which printed the whole `ToolDto`);
  - `AddTool` / `ChangeTool` (which printed the whole `Tool`);
  - the tool-library import (`commands/tool_import.rs`);
  - the library reader (`tool_library.rs`).
  - Dropped: changing only the loader. The commands printed the same debug dump, and an MCP agent reads those refusals.
- **Not in scope:** a missing or unknown field in a project file is a serde parse error, raised before this law runs. Its message is serde's, and it already names the field.
- **Test changed:** `tool_cutter_params::a_reshape_is_refused_by_name_against_the_new_numbers` asserted the old "finite and positive" words. It now asserts the named rule.
