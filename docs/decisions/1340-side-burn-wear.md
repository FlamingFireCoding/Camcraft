## #1340 - Sinker Side Burn: electrode wear compensation in Z

**Decided.** A Sinker Side Burn takes a measured electrode wear (length
lost over depth burned) and a layer step, as a Sinker Burn does (#546).
The plunge is `sinker_wear::burn_compensated`'s: each layer tip is made
deeper by the rate, under all of its refusals (rate, layer step,
electrode length, the deepest-tip bound). The lateral move and its return
run at the last layer's compensated tip, which is checked against the
same bound. The stock sim reads the side burn's moves as the worn
electrode burns them (`sinker_wear_op::as_burned`). The by-name refusals
`SIDE_WEAR_REFUSAL` and `SIDE_LAYER_REFUSAL` are removed. A half-stated or
nonsense wear or layer step is still refused by name at every door.

**Only the measured wear applies, not the condition's `wear`.** A side
burn without a measured wear keeps its plain moves, even when its
condition states a wear rate. Why: a side burn that the user saved before
this change has no measured wear, and its posted program must stay byte
for byte. The condition's rate is applied to a side burn only when the
user states it on the op. Considered and dropped: apply the condition's
wear as the Sinker Burn does. That changes the posted Z of every saved
side burn whose condition states a wear, with no action of the user.

**Only the axial (end) wear is compensated.** The lateral move is not
lengthened for side wear. A worn electrode is smaller, so the wall is
left short of the stated amount, never cut past the allowance. The
allowance law (`undersize - gap`) stays the same. Considered and dropped:
a side-wear rate. No public source gives a model that we could check, and
a longer move could go past the allowance.

**Cost to change later.** Opting a side burn in to the condition's wear
is one line in `sinker_wear_op::side_wear` plus the staleness stamp; it
changes posted output of saved documents, so it is a breaking change.
