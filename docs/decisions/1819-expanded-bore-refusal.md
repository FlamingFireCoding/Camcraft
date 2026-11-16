## #1819 - Refuse expanded bores without spindle-stop support

**What was decided:** Heidenhain, TinyG and Carvera refuse a BORE cycle by
name, with or without a bottom dwell. The counted diagnostic blocks app
export. The developer post CLI still exposes diagnostic text; this change
does not promise that the engine suppresses every program line.

**Why:** Their expanded bore chain feeds in and rapids out with the spindle
running. It does not implement the spindle stop required before a G86
retract. Issue #1819 explicitly permits a named refusal instead of posting
that unsupported cycle. Ordinary drilling and backboring keep their existing
behavior.

**Considered and dropped:** Adding spindle stop and restart in these scripts
would need validated spindle modal state and matching verification support.
That is broader than the smallest safe fix. Keeping the existing chain with
only a dwell would leave the reported behavior unchanged.

**Cost to change later:** Replace each refusal with a validated spindle
sequence and pins that prove feed, dwell, spindle stop, rapid retract and
spindle restart ordering. Check the verifier and simulation against that
sequence before allowing export.
