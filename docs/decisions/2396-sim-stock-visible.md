## #2396 - Simulation: the stock body is not drawn at all

**Decided:** the playback worker packs the session's constant GPU lattice
(`PackedBase`) from the uncut stock, before the calculation before playback
(#2181) runs, exactly as it did before #2181. The pinning test reads the
packed picture the eye draws at move 0, not only the sparse field.

**Why:** `PackedBase::of` runs the eikonal pass (#22), which writes the
composed `max(mesh, cut)` into the mesh layer; every later frame's cut layer
can only lift cells over it. #2181 moved the pack after the calculation,
which leaves the playback standing at the end of the program, so the base
was the finished part and every frame drew the finished part. In a program
that machines the part to size, that body sits inside the model: no stock
was seen at any playhead. The existing #2181 tests compared the sparse
field's cut layer bit for bit, which was right - only the packed base was
wrong, and nothing read it.

**Considered and dropped:** re-packing the base from keyframe 0 after the
calculation (same result, an extra pack and an extra clone of the uncut
stock); skipping the eikonal pass for playback bases (changes the shading
of every frame and keeps the wrong base anyway).

**Cost to change later:** none - the pack is the same call in a different
place.
