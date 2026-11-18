## #2411 - Map the DXF profile MCP suite to its app modules

**Decided:** Add `app::mcp`, `app::tool_profile_dxf`, and `app::commands` to
the suite's Gate map header. Add a selection pin for each module in the
existing xtask test module.

**Why:** The missing header fails the xtask gate. The map also lets the
quick gate select `tool_dxf_profile_mcp` when these modules change.

**Dropped:** Do not weaken the Gate map rule or exempt this suite. Either
choice would hide missing coverage. Runtime changes are unnecessary.

**Cost to change later:** Update the header and selection pin if the
covered modules move.
