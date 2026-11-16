## #1849 - Split `operation/params.rs` along its operation families

**What was decided.** `params.rs` held one function, `OpParams::stamp_with`,
of about 1780 lines. It is now split into child modules under
`operation/params/`. The match in `params.rs` stays exhaustive. It keeps the
Face, Contour, Pocket and Adaptive2D arms inline and sends every other kind
to one method per family: `mill2d`, `mill3d`, `multiaxis`, `turning`,
`holes`, `cutting` (wire, sinker, plasma). The shared writers (`w`, `wh`,
`wh_opt`, `wp2`, `wt`) and the link trailer moved to `params/digest.rs`.
Every arm body moved verbatim, so no digest byte changed.

**Why.** The 400-line law (AGENTS.md "Code shape"). The pocket family stays
in `params.rs` because open PRs #1817 and #2037 edit the Pocket arm. Leaving
it in place lets them merge without a conflict.

**Why child modules of `params`.** `operation/mod.rs` needs no new lines,
and the public path (`OpParams::stamp_with`, `pub(super)`) is unchanged.

**Considered and dropped.**
- A `Digest` struct with methods in place of the closures. It would rewrite
  every arm body (`w(h, x)` -> `d.w(x)`), conflict with #1817/#2037 and make
  the move hard to check line by line.
- A pinned digest value per kind as a permanent test. `DefaultHasher` is not
  stable across Rust releases, so such a pin would break on a toolchain
  bump. The byte identity was checked once instead: the digests of every
  kind's seed, with default and edited links (132 cases), matched before and
  after the split.

**Cost to change later.** Low. A new kind adds its variant to the dispatch
in `params.rs` (the compiler demands it) and its arm to its family file. Each
family method ends in `_ => {}`; the exhaustive dispatch is what keeps a kind
from reaching the wrong family.
