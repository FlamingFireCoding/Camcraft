## #1498 - Keep Sodick control-side wire offsets unsupported

**Decided:** The Sodick post continues to accept generated wire-centre
paths only. Control-side G41/G42 with an H register, wear offsets, and
skim passes that reuse one part path with different H values remain
unsupported. This takes the issue's explicit "supported or not" decision
option. No H number format is adopted because the post emits no H word.

**Why:** `WireContourOp` and `WireSkim` store conditions, but no H-register
identifiers or values. The generator uses each condition's wire radius
and spark gap to generate a centre path. The wire laws judge that path
against the part and stock. An operator's H value could move the actual
wire away from the path those laws judged. The existing milling
`CompSpan` contract assumes Fanuc-style engage and cancel geometry; it
does not establish Sodick's geometry or the contents of an H register.
This change makes no claim that those control semantics were verified.

The post now handles every `COMP` record with a Sodick refusal that names
the operation and explains the missing verified H-register value. It
suppresses later cutting moves in that section. A request before
`SECTION` stays refused when the section opens. `COMP OFF` is also
unsupported and cannot clear a refusal. Section end still cuts an
already-threaded wire and resets the state for a later plain section.
The existing skim-condition refusal remains in place.

The engine already counted a warning when the Sodick script lacked the
compensation hook, so the app already blocked release of that program.
The change replaces that generic warning and suppresses the affected
cutting blocks in the diagnostic output; it does not repair a previously
silent release by the app.

**Considered and dropped:** Writing G41/G42 H around the current centre
path could apply an additional offset. Posting a part path with an
operator-chosen H value would disconnect the posted cut from the wire
laws. Reusing milling compensation would assume control behaviour the
Sodick implementation has not established. Accepting only `COMP OFF`
would introduce a partial compensation contract without enabling the
requested feature. Retaining only the generic engine warning would leave
the affected cutting blocks in its diagnostic output.

**Cost to change later:** Support needs published control semantics for
H-register format, range and units, including metric formatting and
condition changes between passes. The document must store each pass's
register and offset value. Generation must prove that the programmed
lead-in, contour and cancellation produce the centre path checked by the
wire laws, and value changes must invalidate that proof and revalidate
the cut. A rough-plus-skim punch golden and a refusal of a gouging offset
must pin the resulting behaviour before the post enables it.
