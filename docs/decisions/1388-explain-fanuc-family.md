## #1388 - Bounded Explain comments for the Fanuc family

- **Decided:** Fanuc, Haas, Mazak, Doosan DNM and Fanuc lathe declare
  `explains = "paren"`, `commentMax = 60` and `explainBlockMax = 80`.
  Their existing `block()` helpers filter and join executable words as
  before, then append the shared `explain()` result only when the entire
  line, separating space and comment fit the 80-character budget.
  Otherwise they omit the optional explanation. Explain off preserves
  the existing program bytes. Fanuc lathe retains its own word meanings
  and station/offset tool reading.
- **Why:** the shared helper already guards comment text and bounds its
  length, but a comment cap alone does not bound the complete block.
  The 80-character limit is a conservative house budget for added
  explanations, not a claimed manufacturer maximum. A block that is
  already longer remains intact; Explain never truncates or splits
  executable words. Existing header and standalone comments are outside
  this change.
- **Considered and dropped:** changing the engine's comment cap for each
  block or adding a new engine helper would widen this post-only change.
  Cutting an explanation midway could leave an incomplete meaning.
  Dropping the complete optional explanation keeps both its meaning and
  the original machine program intact. Editing executable blocks to make
  room, or changing unrelated post functions, is unnecessary.
- **Verification:** per-post Explain goldens, comparison with Explain off
  after removing only appended comments, line-budget boundaries and
  per-post mutation checks pin the behavior. No controller simulator or
  machine execution is part of this verification.
- **Cost to change later:** a different output budget changes the post's
  declared limit and its boundary tests and goldens. A future shared
  block-budget helper could remove the repeated append check across
  posts, with tests preserving each post's executable output.
