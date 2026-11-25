// Sodick wire EDM post, LN2W control dialect (#4045). Provenance: Sodick
// Inc., "Wire EDM Machine Operation Training, LN1W / LN2W Control" (AQ400L,
// AQ600L, AD360L; part number 6300015, January 2012; public at
// fab.cba.mit.edu/content/tools/sodick_edm/sodick_user_manual_small.pdf) as
// this repository recorded it (#4045, #1500, and the #1496 / #2649 / #2686
// decisions): G51 / G52 lean the upper guide left / right of travel by the
// A angle, G50 cancels and the wire stands upright again during the next
// X Y move, TP / TN load the Table-to-Program and Table-to-Next heights,
// T84 / T85 high / low flushing, T90 / T91 cut / thread the wire, M02 ends
// (no M30); and the statements of the owner of a Sodick AQ600L with the
// LN2W control (2026-11-25, on #4045 and #1500): the start sequence below,
// Z0 is the upper head height that the operator sets, the control alarms
// on G21 (E00170), TP / TN stand as TP100. and TN0. before any motion; and
// RS-274 / ISO 6983 for the motion words. No other CAM product's post,
// post library or output was read. The Mark 21 post (sodick-wire.cps) is a
// separate file and is not changed: the engine has no include, so the
// shared helpers are copied here. NOT VERIFIED ON A CONTROL in this form:
// check the output at the machine before cutting. What is confirmed and
// what is inferred is in docs/decisions/4045-sodick-ln2w-dialect.md.
//
// Written, in the owner's order for each cut: the setup's G54-G59, TP and
// TN for a tapered cut, G00 X Y to the start point, Z0., T91 (thread; an
// M00 without automatic threading), T84 / T85 and T94 / T89 when the
// machine file states edm.flushing / edm.tank, the C condition word,
// U0. V0. (the taper axes upright), then G01, G02 / G03 with I J; a taper
// as G51 / G52 A before the first leaning move and G50 before the move
// that stands the wire upright; each skim pass's flushing word
// (edm.skim_flushing) and C word before its first move; T90 (cut the wire);
// M00 / M01; M02.
// Never written (each a named refusal or an operator's task): G20 / G21
// (the unit is the control's setting; this post writes millimetres only),
// G92 and G29, the control's wire offset (G41 / G42, H - the path is the
// wire-centre path with the kerf in it), 4-axis (G74 / G75, U V other than
// the U0. V0. line), a taper on a machine file that states no
// edm.taper_angle or no edm.table_to_part_bottom, the tank fill / drain,
// pump and wire-drive T codes, M03, M30.
description = "Sodick wire EDM, LN2W control (AQ400L / AQ600L; start sequence from an owner's statements, taper as G51 / G52 A with TP / TN; not verified on a control)";
vendor = "Camcraft";
legal = "Original clean-room work; public documentation and a machine owner's statements only";
certificationLevel = 1;
minimumRevision = 1;
extension = "nc";
setCodePage = "utf-8";
capabilities = 1;          // reference contract: integer feature mask
wireEdm = true;            // voices PARAM EdmCondition (the engine's door)
afterModes = true;         // reads PARAM AfterOperation (M01 / M00; home refused)
programUnits = "mm";       // #4045: the control alarms on G21, so no unit word can be written - the engine refuses an inch document by name

// The LN manual's header names C000 the "Approach" condition (#2649). Off
// by default, as in the Mark 21 post.
declaredOptions = {
  approachCondition: {
    type: "boolean",
    title: "C000 approach condition",
    group: "Conditions",
    description: "Cut the approach move (start point to the contour) on condition C000, then select the operation's condition. C000 must exist in the control's condition table",
    default: false,
  },
};

// Determinism law: no timestamps, no randomness.

// ---------------------------------------------------------------- state
var wcsWord = "";
var orgX = 0, orgY = 0;
var opName = "", opKind = "", setupName = "";
var condition = "", side = "", afterOp = "";
var inSection = false;     // between onSection and onSectionEnd
var sectionOk = false;     // the section is one this post can write
var threaded = false;      // the wire is threaded through the work
var compOff = false;       // the section's first cut has been written
var cutZ = null;           // the section's one programmed plane
var awt = false;
var tankWord = "";         // T94 / T89 from edm.tank ("": not stated)
var flushWord = "";        // T84 / T85 from edm.flushing ("": not stated)
var skimFlushWord = "";    // T84 / T85 from edm.skim_flushing ("": not stated)
var taper = 0;             // the section's stated taper, degrees (0: vertical)
var dialect = "", corner = "";  // "angle": the stream carries angle words; the machine's corner rule
var plane = null, bottom = null; // the programmed plane (the work's top) and the work's bottom, work frame Z
var angleNext = null;      // the next move's angle word (WireAngle); null: none stated
var tapering = 0;          // the active taper word: 1 G51, -1 G52, 0 upright
var controlOffset = false; // a COMP record refuses this section, even before SECTION
var fourAxis = "";         // the section's 4-axis pairing rule ("": one profile)
var leaning = false;       // the next move states an upper guide offset (WireU / WireV)
// A wire number the stream states but that is no finite number (the
// wire-iso post's #3236 guard): refused, and no cut of the section is written.
var WIRE_NUMBERS = ["WireTaper", "WirePlane", "WireBottom", "WireAngle", "WireU", "WireV"];
var notFinite = "";        // the name of such a param ("": none)
var machineGood = false;
var approach = false;      // the approachCondition option
var approachPending = false; // C000 is selected; the pass's C word follows the approach
var skim = 0;              // skim passes begun in this section
var skimPending = false;   // a skim's condition arrived; select it at its first move
var refused = {};

// ------------------------------------------------------------ helpers
function has(v) { return v !== null && v !== undefined && isFinite(v); }
// A number always carries its decimal point: the control reads a number
// without one in its smallest unit, so "X5" would be a different place.
function fu(v) {
  var s = Number(v).toFixed(3).replace(/0+$/, "");
  return (s === "-0." || s === "-.") ? "0." : s;
}
function wo(letter, v, o) { return has(v) ? letter + fu(v - o) : ""; }
function arcWord(k, v) { return k === "Z" ? "" : wo(k, v, k === "X" ? orgX : orgY); }  // #3442: a position word as written (no Z), for arcFate
function block(words) {
  var a = [];
  for (var i = 0; i < words.length; i++) {
    if (words[i] !== "" && words[i] !== null && words[i] !== undefined) a.push(words[i]);
  }
  if (a.length > 0) writeln(a.join(" "));
}
function msg(s) { if (typeof message === "function") message(s); else note(s); }
function refuse(key, s) {
  if (refused[key]) return;
  refused[key] = true;
  msg(s);
}
function who() { return opName ? "operation " + opName + ": " : ""; }
function clean(s) {
  var t = String(s === null || s === undefined ? "" : s);
  t = t.replace(/[()%;\/]/g, " ").replace(/[^\x20-\x7E]/g, "?").replace(/\s+/g, " ").trim();
  return t.length > 48 ? t.substring(0, 48).trim() : t;
}
// #3213: the control rejects hashes, brackets and escapes in every comment.
function note(s) { block(["(" + String(s).replace(/[#\[\]\\]/g, " ") + ")"]); }
// #3214: the posted condition's row in the machine file. The path's kerf
// offset used its gap; the program states that gap so the operator can
// compare it with the control's own row, and says so loudly when the row
// is an example (notes starting "EXAMPLE").
function conditionRow(id) {
  var e = typeof machine !== "undefined" && machine ? machine.edm : null;
  var list = e && e.conditions ? e.conditions : [];
  var key = String(id).replace(/\s+/g, "").toUpperCase();
  for (var i = 0; i < list.length; i++) {
    if (list[i] && String(list[i].id).replace(/\s+/g, "").toUpperCase() === key) return list[i];
  }
  return null;
}
function conditionNotes() {
  var c = conditionRow(condition);
  if (!c) return;
  var gap = has(c.gap) ? Number(c.gap).toFixed(3) + " MM" : "NOT STATED";
  if (/^\s*EXAMPLE/i.test(String(c.notes || ""))) {
    note("WARNING CONDITION " + condition + " IS AN EXAMPLE ROW - THE KERF USES AN EXAMPLE GAP OF " + gap + " - REPLACE IT BEFORE CUTTING");
  } else {
    note("CONDITION " + condition + " KERF GAP " + gap + " - COMPARE WITH THE CONTROL ROW");
  }
}
// Program notes use this dialect without truncating the stated bound.
function onComment(s) { note(commentText(s)); }

// ------------------------------------------------------------- machine
function machineName() {
  var m = typeof machine !== "undefined" ? machine : null;
  return (m && (m.model || m.name)) || "no machine";
}
function edm() {
  var m = typeof machine !== "undefined" ? machine : null;
  return (m && m.edm) || {};
}
// The machine check, once per program: a wire EDM with no rotary.
function machineOk() {
  var m = typeof machine !== "undefined" ? machine : null;
  if (!m || m.kind !== "wire_edm") {
    refuse("kind", "the Sodick LN2W wire post writes wire words only a wire EDM reads - machine " + machineName() + " is not declared kind wire_edm");
    return false;
  }
  awt = !!(m.edm && m.edm.awt === true);
  if (!dielectricOk(m.edm || {})) return false;
  if (m.rotaries && m.rotaries.length > 0) {
    refuse("rotary", "machine " + machineName() + " states a rotary axis - the Sodick LN2W wire post writes no rotary word");
    return false;
  }
  return true;
}

// The machine file's tank and flushing, as the manual's T words (#1499,
// #2686). Unstated writes nothing (the operator sets it at the control);
// a word this post does not know is refused by name, never guessed.
function dielectricOk(e) {
  var ok = true;
  if (e.tank !== undefined && e.tank !== null) {
    if (e.tank === "submerged") tankWord = "T94";
    else if (e.tank === "non-submerged") tankWord = "T89";
    else {
      refuse("tank", "machine " + machineName() + ": edm.tank " + JSON.stringify(e.tank) + " is not submerged or non-submerged - the Sodick LN2W wire post writes T94 or T89 only");
      ok = false;
    }
  }
  if (e.flushing !== undefined && e.flushing !== null) {
    if (e.flushing === "high") flushWord = "T84";
    else if (e.flushing === "low") flushWord = "T85";
    else {
      refuse("flushing", "machine " + machineName() + ": edm.flushing " + JSON.stringify(e.flushing) + " is not high or low - the Sodick LN2W wire post writes T84 or T85 only");
      ok = false;
    }
  }
  // A T word is modal: a skim's flushing would stay active for the next
  // operation's rough cut if that cut selected none of its own.
  if (e.skim_flushing !== undefined && e.skim_flushing !== null) {
    if (e.skim_flushing === "high") skimFlushWord = "T84";
    else if (e.skim_flushing === "low") skimFlushWord = "T85";
    else {
      refuse("skimflushing", "machine " + machineName() + ": edm.skim_flushing " + JSON.stringify(e.skim_flushing) + " is not high or low - the Sodick LN2W wire post writes T84 or T85 only");
      ok = false;
    }
    if (e.flushing === undefined || e.flushing === null) {
      refuse("skimflushing", "machine " + machineName() + ": edm.skim_flushing is stated without edm.flushing - the skim's flushing word would stay active for the next rough cut; state edm.flushing (high or low) too");
      ok = false;
    }
  }
  return ok;
}

// The header's word on tank and flushing: what is programmed, what is
// left to the operator.
function dielectricNotes() {
  if (tankWord === "" && flushWord === "") {
    note("TANK FILL AND FLUSHING ARE NOT PROGRAMMED - SET THEM AT THE CONTROL");
    return;
  }
  if (tankWord === "T94") note("T94 SUBMERGED MACHINING AFTER THREADING - NO FILL OR DRAIN WORD IS WRITTEN - CHECK THE TANK IS FULL");
  else if (tankWord === "T89") note("T89 NON-SUBMERGED MACHINING AFTER THREADING");
  else note("TANK IS NOT PROGRAMMED - SET IT AT THE CONTROL");
  if (flushWord === "T84") note("T84 HIGH-PRESSURE FLUSHING AFTER THREADING");
  else if (flushWord === "T85") note("T85 LOW-PRESSURE FLUSHING AFTER THREADING");
  else note("FLUSHING IS NOT PROGRAMMED - SET IT AT THE CONTROL");
  if (skimFlushWord === "T84") note("T84 HIGH-PRESSURE FLUSHING BEFORE EACH SKIM PASS");
  else if (skimFlushWord === "T85") note("T85 LOW-PRESSURE FLUSHING BEFORE EACH SKIM PASS");
}

// -------------------------------------------------------------- header
// Absolute positions; no unit word (the control alarms on G21). The
// header states what the program leaves to the operator.
function onOpen() {
  machineGood = machineOk();
  approach = typeof postOptions !== "undefined" && postOptions && postOptions.approachCondition === true;
  note(clean(machineName()));
  note("SODICK LN2W WORDS - NOT VERIFIED ON A CONTROL - CHECK BEFORE CUTTING");
  note("PROGRAM IS IN MM - NO G20 OR G21 IS WRITTEN - THE CONTROL UNIT SETTING MUST BE MM");
  note(awt ? "T91 THREADS THE WIRE AT EACH START POINT" : "WIRE NOT THREADED AT START - THREAD IT AT EACH M00");
  note("WORK OFFSET SET AT THE CONTROL - NO G92 IS WRITTEN");
  note("Z0. FOLLOWS EACH MOVE TO A START POINT - SET Z0 AT THE UPPER HEAD HEIGHT ABOVE THE PART BEFORE START");
  dielectricNotes();
  note("WIRE-CENTRE PATH - NO WIRE OFFSET G41 G42 IS USED");
  note("EACH C CONDITION NAMED BELOW MUST EXIST IN THE CONTROL CONDITION TABLE BEFORE START");
  if (approach) note("C000 APPROACH CONDITION ON EACH APPROACH MOVE - C000 MUST EXIST IN THE CONTROL CONDITION TABLE");
  block(["G90"]);
}

function onParameter(name, value) {
  if (WIRE_NUMBERS.indexOf(name) >= 0 && !has(value)) {
    sectionOk = false;
    notFinite = name;
    refuse("nf" + name + opName, who() + "PARAM " + name + " is not a finite number (" + (value === null || value === undefined ? "null" : clean(value)) + ") - refused, the Sodick LN2W wire post writes no cut from it");
    return;
  }
  if (name === "WorkOffset" && value) wcsWord = String(value);
  else if (name === "OriginX" && has(value)) orgX = Number(value);
  else if (name === "OriginY" && has(value)) orgY = Number(value);
  else if (name === "OperationName") opName = clean(value);
  else if (name === "OperationKind") opKind = clean(value);
  else if (name === "SetupName") setupName = clean(value);
  else if (name === "EdmCondition") {
    // With the wire threaded, a condition is the next skim pass's (#538,
    // #2649): startSkim writes it before that pass's first move.
    condition = String(value === null || value === undefined ? "" : value);
    if (inSection && threaded) skimPending = true;
  }
  else if (name === "WireSide") side = clean(value);
  else if (name === "AfterOperation") afterOp = String(value);
  else if (name === "WireTaper") taper = Number(value);
  else if (name === "WireFourAxis") fourAxis = clean(value);
  else if (name === "WireTaperDialect") dialect = clean(value);
  else if (name === "WireCorner") corner = clean(value);
  else if (name === "WirePlane") plane = Number(value);
  else if (name === "WireBottom") bottom = Number(value);
  else if (name === "WireAngle") angleNext = Number(value);
  else if (name === "WireU" || name === "WireV") leaning = true;
  else if (name === "SpindleSpeed" && has(value) && Number(value) > 0) {
    refuse("rpm" + opName, who() + "states " + Math.round(Number(value)) + " rpm - a wire EDM turns no spindle");
  }
}

function onToolChange(number, name) {
  refuse("tool" + opName, who() + "changes to tool " + number + " - a wire EDM mounts no tool (the wire is the condition's)");
}

// #1498: the wire laws judge the generated wire-centre path. No stored
// H-register value ties a control-offset path to that judged geometry.
// Refuse every COMP record (including wear and OFF); OFF cannot make a
// refused section safe again. Section end resets the refusal.
function onRadiusCompensation() {
  controlOffset = true;
  sectionOk = false;
  refuse("offset" + opName, who() + "Sodick LN2W: control-side wire offset (G41/G42 H) is not supported - no verified H-register value is stored; post a generated wire-centre path without control or wear compensation");
}

// ------------------------------------------------------------- section
// A tapered section's heights for TP / TN, or null with the refusal said.
// The stream's X Y are on the work's top plane (WirePlane), so that plane
// is the program plane: TP is the worktable to the work's top, TN the
// worktable to the work's bottom (edm.table_to_part_bottom).
function taperHeights() {
  var t = edm().table_to_part_bottom;
  if (!has(t) || Number(t) < 0) {
    refuse("tp" + opName, who() + "is tapered, and machine " + machineName() + " states no edm.table_to_part_bottom (the height of the work's bottom above the worktable) - the Sodick LN2W wire post writes TP and TN from it and never guesses a height");
    return null;
  }
  if (!has(plane) || !has(bottom) || !(plane - bottom > 0.0005)) {
    refuse("tpz" + opName, who() + "is tapered, and the path states no work top above its bottom (PARAM WirePlane, WireBottom) - regenerate it");
    return null;
  }
  return [Number(t) + (plane - bottom), Number(t)];
}

function onSection() {
  inSection = true;
  sectionOk = false;
  threaded = false;
  compOff = false;
  cutZ = null;
  skim = 0;
  skimPending = false;
  approachPending = false;
  tapering = 0;
  if (condition === "") {
    refuse("op" + opName, who() + "is not wire EDM work (no EDM condition) - the Sodick LN2W wire post posts Wire Contour operations only");
    return;
  }
  var ok = machineGood && !controlOffset && notFinite === "";
  // The condition word: C and three digits (C000 to C999). The operation's
  // condition id is that word; anything else is refused.
  if (!/^C[0-9]{3}$/.test(condition)) {
    refuse("cid" + opName, who() + "condition id " + JSON.stringify(condition) + " is not a Sodick condition word (C and three digits, C000 to C999) - name the machine file's condition that way");
    ok = false;
  }
  var heights = null;
  if (fourAxis !== "") {
    refuse("four" + opName, who() + "is a 4-axis cut - the Sodick LN2W wire post writes a constant taper only (G51 / G52 A); no G74 / G75 or U V profile words");
    ok = false;
  } else if (taper !== 0) {
    if (dialect !== "angle") {
      refuse("taperuv" + opName, who() + "is tapered, and machine " + machineName() + " states no edm.taper_angle - the LN2W control tapers by angle word (G51 / G52 A), not by U V moves; state edm.taper_angle (positive, corner) in the machine file");
      ok = false;
    } else {
      heights = taperHeights();
      if (heights === null) ok = false;
    }
  } else if (dialect !== "") {
    refuse("angle0" + opName, who() + "states an angle-word dialect without a taper angle - refused");
    ok = false;
  }
  if (!/^G5[4-9]$/.test(wcsWord)) {
    refuse("wcs" + opName, who() + "work offset " + JSON.stringify(wcsWord) + " is not G54 to G59 - the Sodick LN2W wire post writes no G92, so the setup must state one");
    ok = false;
  }
  if (!ok) return;
  sectionOk = true;
  note(opName + (opKind ? " - " + opKind : ""));
  if (setupName) note("SETUP " + setupName);
  if (side) note("SIDE " + side.toUpperCase() + " - KERF IN THE PATH");
  // The C word selects a row of the control's own condition table; the
  // program carries no table, so the operator checks the row exists.
  note("CONDITION " + condition + " MUST EXIST IN THE CONTROL CONDITION TABLE - CHECK ITS VALUES");
  conditionNotes();
  if (heights !== null) {
    note("TAPER " + fu(Math.abs(taper)) + " DEG PER SIDE BY G51 G52 A - XY ON THE WORK TOP - CORNERS " + corner.toUpperCase());
    note("TP IS THE WORKTABLE TO THE WORK TOP - TN IS THE WORKTABLE TO THE WORK BOTTOM - CHECK BOTH AT THE CONTROL");
  }
  block([wcsWord]);
  // The owner's known-good program states TP and TN before any motion.
  if (heights !== null) {
    block(["TP" + fu(heights[0])]);
    block(["TN" + fu(heights[1])]);
  }
}

// The first motion of a section, in the order the owner of an AQ600L
// (LN2W) gave: G00 to the start point, Z0. (the upper head height the
// operator set), T91 (or the operator at an M00), flushing, tank, the
// condition, U0. V0. (the taper axes upright), then the first cut.
function openRun() { return side === "left" || side === "right"; }
function startCut(p) {
  if (threaded) {
    if (skimPending) return startSkim();
    // The approach (the section's first move) was cut on C000; the
    // operation's own condition takes over for the contour (#2649).
    if (approachPending && compOff) {
      approachPending = false;
      block([condition]);
    }
    return true;
  }
  if (!sectionOk) return false;
  if (!has(p.StartX) || !has(p.StartY)) {
    refuse("start" + opName, who() + "the path states no threading point - regenerate it");
    return false;
  }
  block(["G00", wo("X", p.StartX, orgX), wo("Y", p.StartY, orgY)]);
  block(["Z0."]);
  if (awt) block(["T91"]);
  else {
    note(openRun() ? "THREAD THE WIRE AT THE RUN-IN POINT OUTSIDE THE STOCK" : "THREAD THE WIRE AT THE START HOLE");
    block(["M00"]);
  }
  if (flushWord !== "") block([flushWord]);
  if (tankWord !== "") block([tankWord]);
  if (approach && condition !== "C000") {
    block(["C000"]);
    approachPending = true;
  } else block([condition]);
  block(["U0.", "V0."]);
  threaded = true;
  cutZ = has(p.StartZ) ? p.StartZ : null;
  return true;
}

// A skim pass (#2649): the previous pass ended back at the start point
// with the wire threaded and upright; the skim's flushing word, then its
// own C word, come before its first move. Its offset is in the path.
function startSkim() {
  skimPending = false;
  approachPending = false;
  if (!/^C[0-9]{3}$/.test(condition)) {
    refuse("skim" + opName, who() + "skim condition id " + JSON.stringify(condition) + " is not a Sodick condition word (C and three digits, C000 to C999) - name the machine file's condition that way");
    sectionOk = false;
    return false;
  }
  if (tapering !== 0) {
    refuse("skimlean" + opName, who() + "a skim pass starts with the wire leaning - refused; every pass must end upright at the start point");
    sectionOk = false;
    return false;
  }
  // #2686: the rough cut's flushing is programmed and the machine file
  // states none for a skim: never cut the skim on the rough cut's flushing.
  if (flushWord !== "" && skimFlushWord === "") {
    refuse("skimflush" + opName, who() + "has a skim pass, and machine " + machineName() + " states edm.flushing but no edm.skim_flushing - the Sodick LN2W wire post does not cut a skim on the rough cut's flushing; state edm.skim_flushing (high or low) in the machine file");
    sectionOk = false;
    return false;
  }
  skim++;
  note("SKIM " + skim + " - CONDITION " + condition + " MUST EXIST IN THE CONTROL CONDITION TABLE");
  conditionNotes();
  if (skimFlushWord !== "") block([skimFlushWord]);
  block([condition]);
  return true;
}

// A cut: one programmed plane, no pose, no guide-offset lean, from a
// threaded wire.
function cutOk(p) {
  var lean = leaning;
  leaning = false;
  if (!inSection || condition === "") {
    refuse("free" + opName, who() + "a move outside a wire section - refused");
    return false;
  }
  if (!sectionOk || notFinite !== "") return false;
  if (has(p.A) || has(p.B) || has(p.C)) {
    refuse("tilt" + opName, who() + "a posed move - the Sodick LN2W wire post leans the wire by G51 / G52 A only");
    sectionOk = false;
    return false;
  }
  if (lean) {
    refuse("uv" + opName, who() + "a leaning move (upper guide offset) - the Sodick LN2W wire post writes no U V lean; the LN2W control tapers by angle word (edm.taper_angle)");
    sectionOk = false;
    return false;
  }
  if (!startCut(p)) return false;
  if (has(p.Z) && cutZ !== null && Math.abs(p.Z - cutZ) > 0.0005) {
    refuse("z" + opName, who() + "a move changes Z - a wire cut rides one programmed plane");
    return false;
  }
  return true;
}

function onRapid() {
  var lean = leaning;
  leaning = false;
  angleNext = null;
  if (threaded) {
    refuse("rapid" + opName, who() + "a rapid with the wire threaded through the work - refused");
    return;
  }
  // Before the first cut the wire is not threaded: the G00 to the first
  // cut's start (written by startCut) stands for every positioning move.
  if (!inSection || condition === "") {
    refuse("free" + opName, who() + "a move outside a wire section - refused");
    return;
  }
  // A positioning move that states an upper guide offset leans the wire
  // before it is threaded (#1497): refuse the section by name.
  if (lean) {
    refuse("uvrapid" + opName, who() + "a leaning positioning move (upper guide offset) - the Sodick LN2W wire post writes no U V lean");
    sectionOk = false;
  }
}

// The taper word of the next move. The control starts the lean during the
// move after G51 / G52 A and stands the wire upright during the move after
// G50, so each word is written before the move that the stream's angle
// word rides. Returns false with the refusal said.
function taperWord(ang) {
  if (ang === null) {
    if (tapering !== 0) {
      refuse("noangle" + opName, who() + "a move with no angle word while the wire leans - refused, the lean of that move is not stated");
      sectionOk = false;
      return false;
    }
    return true;
  }
  if (taper === 0 || dialect !== "angle") {
    refuse("ang" + opName, who() + "an angle word in a section that states no angle-word taper - refused");
    sectionOk = false;
    return false;
  }
  if (ang === 0) {
    if (tapering !== 0) block(["G50"]);
    tapering = 0;
    return true;
  }
  var want = ang > 0 ? 1 : -1;
  if (Math.abs(Math.abs(ang) - Math.abs(taper)) > 0.0005) {
    refuse("angval" + opName, who() + "an angle word of " + fu(ang) + " degrees in a section that states a " + fu(Math.abs(taper)) + " degree taper - refused");
    sectionOk = false;
    return false;
  }
  if (tapering === 0) {
    // G51: the upper guide leans left of travel; G52: right (a positive
    // stream angle is "top left" when the machine file says so).
    block([want > 0 ? "G51" : "G52", "A" + fu(Math.abs(ang))]);
    tapering = want;
  } else if (tapering !== want) {
    refuse("angside" + opName, who() + "the wire's lean changes side along the cut - refused, one G51 / G52 word holds one side");
    sectionOk = false;
    return false;
  }
  return true;
}

function onLinear() {
  var p = properties;
  var ang = angleNext;
  angleNext = null;
  if (!cutOk(p)) return;
  if (!taperWord(ang)) return;
  // No F word: the condition sets the rate.
  block(["G01", wo("X", p.X, orgX), wo("Y", p.Y, orgY)]);
  compOff = true;
}

function onCircular() {
  var p = properties;
  var ang = angleNext;
  angleNext = null;
  if (p.CircularPlane && p.CircularPlane !== "XY") {
    refuse("plane" + opName, who() + "an arc in the " + p.CircularPlane + " plane - a wire cut is flat");
    return;
  }
  if (!threaded && !compOff && sectionOk && inSection && condition !== "") {
    refuse("arcfirst" + opName, who() + "the first cut is an arc - the Sodick LN2W wire post starts every cut with a straight move from the start point; give the cut a straight lead-in");
    sectionOk = false;
    return;
  }
  if (ang !== null || tapering !== 0) {
    refuse("taperarc" + opName, who() + "a tapered arc - the Sodick LN2W wire post writes straight tapered moves only");
    sectionOk = false;
    return;
  }
  if (!cutOk(p)) return;
  // #3442: an arc whose written ends coincide is no arc block (arcFate, PRELUDE).
  var fate = arcFate(arcWord, arcIncOf(fu));  // #3474: I J K are increments from the start
  if (fate !== "arc") { if (fate === "linear") onLinear(); return; }
  var c = p.IJKIncrement || {};
  if (!has(c.I) || !has(c.J) || !has(p.StartX) || !has(p.StartY)) {
    refuse("arcc" + opName, who() + "an arc states no centre - refused");
    return;
  }
  block([
    p.CircularDirection === "CW" ? "G02" : "G03",
    wo("X", p.X, orgX), wo("Y", p.Y, orgY),
    "I" + fu(c.I - p.StartX), "J" + fu(c.J - p.StartY),
  ]);
}

function onCycle() {
  refuse("cycle" + opName, who() + "a canned cycle - a wire EDM has no drilling cycle");
}

// The section's end: cut the wire (T90, or the operator at an M00), then
// the operator's stop if the op asks one. A cut that ends with the wire
// leaning is refused: G50 needs a move to stand the wire upright.
function onSectionEnd() {
  if (tapering !== 0) refuse("endlean" + opName, who() + "the cut ends with the wire leaning - refused; the last move must stand the wire upright");
  if (threaded) {
    if (awt) block(["T90"]);
    else {
      note(openRun() ? "CUT THE WIRE AT THE RUN-OUT POINT" : "CUT THE WIRE");
      block(["M00"]);
    }
  }
  if (afterOp === "optional-stop") block(["M01"]);
  else if (afterOp === "stop") block(["M00"]);
  else if (afterOp === "home") {
    refuse("home" + opName, who() + "asks a trip home - the Sodick LN2W wire post states no home position; set the operation to Continue");
  }
  inSection = false;
  sectionOk = false;
  threaded = false;
  compOff = false;
  skim = 0;
  skimPending = false;
  approachPending = false;
  condition = "";
  side = "";
  afterOp = "";
  taper = 0;
  dialect = "";
  corner = "";
  plane = null;
  bottom = null;
  angleNext = null;
  tapering = 0;
  controlOffset = false;
  fourAxis = "";
  leaning = false;
  notFinite = "";
}

// ------------------------------------------------------------ footer
function onClose() {
  if (threaded) refuse("open", "the program ends with the wire threaded - refused");
  block(["M02"]);
}
