## #2051 - troubleshooting stem follows the incomplete-model message

**What was decided.** The troubleshooting heading for the incomplete-model
refusal now quotes the message as the app prints it: `model '<model>' was`
then the backticked fixed text `imported incomplete - <hole> - so it has
walls missing and no gouge check against it can be trusted; ...`. The
guard stems are the fixed text in the sources, not the old hole wording.

**Why.** Commit 35b422cf made the message a format string
(`crates/app/src/mesh_open.rs`, `incomplete`), so the old literal
`closed shell ... meshes open` no longer exists in `crates/*/src`. The
guard reads sources only, and it must not be weakened to read tests.

**Considered and dropped.** Re-adding the old literal to `mesh_open.rs`:
that keeps a message wording that no user sees. Keeping `was imported
incomplete` as one backticked stem: `was` is not literal in the sources
(it comes from `model_fidelity.rs`), so the guard would still fail.

**Cost to change later.** Low. A new wording of the message needs a new
heading and passes the guard only when its fixed text is literal in the
sources.
