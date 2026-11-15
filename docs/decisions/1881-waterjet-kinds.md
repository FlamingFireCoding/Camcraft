## #1881 - Waterjet machine, nozzle and cutting-data seam

- **Declare waterjet explicitly and require linear X/Y, with optional Z.**
  An axis inventory cannot distinguish a waterjet from another gantry.
  The parser and editor share the existing `NoSpindle` walker for named
  spindle, live-tool and milling-coolant refusals; waterjet also refuses
  rotary axes. Saved machines enforce the same restrictions. Reusing plasma
  directly was dropped because plasma requires Z. Adding another machine
  variant later will require its own declared rules, not relaxed waterjet
  validation. Asymmetric travel ranges retain their existing meaning.

- **Store the stream kerf in the tool's diameter and require zero RPM.**
  This preserves the existing radius and tool-library paths while giving
  invalid kerf and RPM values waterjet-specific refusal text. A corner
  radius is refused because this diameter describes a stream, not a cutter.
  A milling-tool alias was dropped because it would admit spindle behavior.
  A separate nozzle-orifice model would need additional fields and file/UI
  handling when a later slice needs physical nozzle geometry.

- **Add optional waterjet data to each cutting row and select by material
  and exact thickness.** The row carries cut feed; its waterjet data carries
  thickness, kerf, pierce time, abrasive delay, re-pierce time and the
  stationary/dynamic/wiggle method. Existing rows keep no waterjet payload.
  Dimensions and feed must be finite and positive; delays may be zero.
  Duplicate material/thickness pairs and waterjet data on other tool kinds
  are refused. Material-only selection and nearest-thickness substitution
  were dropped because either could silently choose the wrong recipe.
  Interpolation or additional recipe keys would need an explicit selection
  policy and corresponding command, file and UI changes.

- **Check cached moves with waterjet machine rules, while leaving concrete
  waterjet operations and built-in posts to their later slices.** The
  checker tests XY feed ceilings and travel, including arc extrema, WCS
  origin and axis direction. It checks Z travel only when Z exists and
  names skipped Z/spindle checks. Only `Family::Waterjet` belongs on this
  machine; existing operation kinds therefore remain refused. Shipped
  milling/plasma/EDM posts do not serve it. A user script must explicitly
  declare that it serves waterjet. Reusing a plasma operation or inferring
  post support from a generic cutting capability was dropped because those
  paths do not establish waterjet behavior. Future operation and post work
  must supply that behavior before it can produce waterjet NC output.

- **Preserve recipes through the existing tool editor and undoable command
  path, with an original nozzle icon.** A new recipe starts with invalid
  zero thickness so Apply requires a stated thickness. Multiple thicknesses
  for one material remain editable. Reusing the torch icon was dropped
  because the tool kinds should be distinguishable. A separate recipe
  editor was unnecessary for this data seam and would duplicate persistence
  and command handling. No new crate dependency is needed.

- **Reuse the released local patch on current upstream main.** The previous
  worktree is preserved as held work, and the issue branch starts at
  `cda3d5896`. Rebuilding the same implementation from scratch was dropped
  because it would discard useful work without changing the design. The
  allowed-kind message assertion is updated with the intentionally extended
  vocabulary. Process files and NC/CL goldens remain outside this change.
