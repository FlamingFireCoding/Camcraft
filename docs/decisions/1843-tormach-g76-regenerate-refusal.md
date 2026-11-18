## #1843 - Explain how to recover from a cached Tormach G76 record

**Decided.** Append guidance to regenerate the threading operation to the
existing Tormach G76 finish-allowance mismatch diagnostic. Keep its refusal
condition and the Fanuc post unchanged.

**Why.** Records cached before #517 can carry a zero finish allowance and
still appear fresh because the stamp hashes inputs without a generator
version. The existing refusal safely rejects the record but gives no
recovery action. Regenerating refreshes the cached thread record.

**Considered and dropped.** Versioning all generator stamps would change
cache invalidation beyond this issue. A separate zero-allowance guard adds
logic where the existing mismatch refusal already identifies the problem.

**Validation.** The guidance pin failed before the fix and with the exact
upstream post restored; both pins passed after restoring the fix. The Fanuc
compatibility pin preserves its existing mm `R0.` and inch `R0.0001` output.
The quick gate passed all 45 steps, and app clippy passed with warnings
denied. No goldens changed.

**Cost to change later.** Update this diagnostic and its refusal pin.
