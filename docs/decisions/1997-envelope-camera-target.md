## #1997 - The orbit pivot after a document open

**What was decided.** A document open frames the camera on the opened
document (the same `frame_all` as the F key: part and stock facets, fixtures
and WCS origins). The work envelope stays out of every camera bound.

**Why.** No code path puts the work envelope (#1567, `c0b4dfc85`) into
`Camera::target`: the envelope is only overlay lines
(`crates/ui/src/work_envelope.rs` into `interact::collect_overlay`), and
`State::frame_all` reads only facets and WCS origins. The pivot that the
maintainer saw is the target that an open left in place: `reset_for_open`
kept the camera of the old document, or the default target at the world
origin. With a machine mounted, the envelope box is drawn about the work
origin, so the orbit appeared to turn about the envelope and not about the
part. Framing on open puts the pivot on the part and the stock.

**Considered and dropped.**
- Tests only, no code change: the pins would pass, but the reported
  behaviour (the pivot away from the part after an open) would stay.
- Orbit about the model point under the pointer: a new feature, filed as
  #1998 (P6).
- Keep the old camera across an open: a pivot from another document has no
  meaning for the new one.

**Cost to change later.** One call in `State::reset_for_open`; the pins are
in `crates/ui/src/egui_app/tests/camera_target.rs`.
