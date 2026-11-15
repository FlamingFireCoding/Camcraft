# PR #1756 boundary-pick evidence

Both images render the actual egui shell in the structural test
`egui_app::tests::edge_click_modifiers::an_armed_boundary_row_takes_edges_in_faces_mode`.
The fixture is the test's prism, with a Face operation and the Boundary group open.

- `before.png`: the previous face-only pick routing restored temporarily for the
  mutation check. Clicking the near edge in Faces mode produces `1 face loop,
  closed`. The test fails because the stored feature is a face rather than a chain.
- `after.png`: the restored PR routing. Clicking the near edge, then Ctrl-clicking
  the far edge, keeps Faces mode and produces `2 edges, 2 open chains`.

These are review evidence, not release pixel goldens. The structural assertion
checks the actual painted entry. To refresh the after image from the current code:

```sh
BOUNDARY_REVIEW_IMAGES="$PWD/docs/reviews/1756" cargo test -p camcraft-ui --lib an_armed_boundary_row_takes_edges_in_faces_mode
```
