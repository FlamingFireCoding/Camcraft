# HEIDENHAIN TNC: differences between controls and NC software versions (post-processor view)

Sources: the HEIDENHAIN manuals and overview documents listed in `inventory.md` (document ID, title, edition date, download URL). The manuals themselves are not in this repository.

**How this file was made (read this first).**

- The manuals were not read page by page. The text was extracted with `pdftotext -layout` and searched with scripts. The main source is the "New functions" / "Changed functions" lists at the front of each manual, and the "Overview of New and Modified Software Functions" documents (TNC 620 ID 1322094, TNC 640 ID 1322095, TNC7 ID 1373081 / 1465689). Only short excerpts of the full chapters were read, to confirm a detail.
- The Q-parameter lists of cycles come from the example blocks in the manuals (a script collects the `Qnnn=` lines after `CYCL DEF nnn`). These lists are reliable for cycles 200 (DRILLING), 203 (UNIVERSAL DRILLING), 205 (UNIVERSAL PECKING), 207 (RIGID TAPPING) and 209 (TAPPING W/ CHIP BRKG). They are not reliable for 247 and 32 (the script picked up a neighbouring cycle), so they are not used for those.
- References: `<manual ID> p<N>` means the PDF page N of that manual. "printed p" means the page number printed in the manual. A version written as `34049x-05` means all variants (340490/340491/340492/340493/340494-05). The iTNC 530 340 49x numbers and 606 42x numbers run in parallel: 606 42x-01 follows 340 49x-05 (single-processor/HSCI line), and 340 49x-06..-08 have the same functions as 606 42x-01..-03 (the 34049x-08 manual lists "60642x-01 since the predecessor version 34049x-05").
- "not found in the extracted text" means a keyword search found nothing. It does not prove that the control lacks the function.
- Section C is in its own file, `differences-by-version.md`. It is generated from the new/changed-function lists. A bullet is "post-relevant" when it names a post-dialect keyword (TOOL CALL, PLANE, TCPM, M-functions, cycles 1-32/200-247, ISO, Q/QS parameters, FMAX/FZ/FU, arcs/helix, program names, removed/import notes). All other bullets are cut to one short line under "other". Some bullets are cut off in the middle where the PDF column layout broke them; follow the manual ID and page.
- Cycle names: the first bare cycle number in each section of sections A, B, B2 and B3 has its HEIDENHAIN short name in parentheses, for example "205 (UNIVERSAL PECKING)". `cycles.md` has the full table (name, purpose, DIN/ISO code, option, versions).
- "feature-mentions.md" in a source column means a keyword search of the extracted text (which keywords appear in which version's manuals). That matrix is not published here. It is evidence for a search, not proof of support.

---

## A. Standard library (all covered controls)

The covered controls do not share one dialect. Three groups exist:

1. **Positioning displays / straight-cut controls (TNC 124, TNC 128).** Each positioning block holds ONE axis. There is no `L` word, no circle and no DIN/ISO. Radius compensation is `R+` / `R-` (lengthen or shorten a paraxial move) or `R0`.
   - TNC 124 (246 xxx-08, 284 679-24 p72): `BEGIN PGM 10 MM` (program NUMBER), `F 9999`, `Z+20`, `X-20 R0`, `TOOL CALL 1 Z`, `S 1000`, `M 3`, `CYCL 1.0 PECKING` ... `CYCL 1.7 F 80`, `CYCL CALL`, `END PGM 10 MM`. Max 1000 NC blocks per program, 2000 in total (p? "2000 NC blocks. A single program can contain a maximum of 1000", 284 679-24, line 3222 of the text).
   - TNC 128 (77184x, 819494-26 printed p132): `BEGIN PGM LINEAR MM`, `BLK FORM 0.1/0.2`, `TOOL CALL 1 Z S4000`, `Z+250 R0 FMAX`, `X+5 R- F500`, `CYCL DEF 200 ...` with Q-parameters, M99. It has the 200-series drilling cycles, 247, 251-256, FN functions and QS strings, but no arcs.
2. **Contouring controls, Klartext only (TNC 310).**
3. **Contouring controls with Klartext and (from a version on) DIN/ISO (TNC 410, TNC 426/430, iTNC 530, TNC 320, TNC 620, TNC 640, TNC7).**

**Common to ALL controls (groups 1-3):** `BEGIN PGM <name|number> MM|INCH` ... `END PGM <same> MM|INCH`; numbered blocks (the control numbers them); `TOOL CALL <n> Z` (tool number, spindle axis); F, S and M words; `R0`; `FMAX` (TNC 124 uses `F 9999` instead); `CYCL DEF` / `CYCL CALL`; M3/M4/M5, M8/M9, M2/M30 (by the standard M list of each manual).

**Common to all CONTOURING controls (TNC 310 and later) - the practical "standard library" for CamCraft:**

| Element | Form | Evidence (oldest manual of the group) |
|---|---|---|
| Program frame | `BEGIN PGM NAME MM` / `END PGM NAME MM` | TNC 310 313 509-22 p43 |
| Blank | `BLK FORM 0.1 Z X.. Y.. Z..` / `BLK FORM 0.2 X.. Y.. Z..` | 313 509-22 p45 |
| Tool | `TOOL DEF n L.. R..` (program-local) and `TOOL CALL n Z S..` | p54, p21 |
| Line | `L X.. Y.. Z.. RL/RR/R0 F../FMAX M..` | p46, p53 |
| Arcs | `CC X.. Y..` + `C X.. Y.. DR+/DR-`; `CR X.. Y.. R± DR±`; `CT X.. Y..`; `RND R..`; `CHF ..` | p38, p73, p74 |
| Polar | `CC` as pole, `LP PR.. PA..`, `CP IPA.. IZ.. DR±` (helix, 3 axes) | p80-81 (TNC 310 spec: "Helices: 3 axes") |
| Radius comp | `RL` / `RR` / `R0` on L/C blocks | p46 |
| M functions | M91 (machine datum), M92 (builder position), M94, M97, M99 (cycle call per block), M89 (modal cycle call) | p58, p91-96 |
| Cycles | 200 DRILLING (Q200 Q201 Q206 Q202 Q210 Q203 Q204), 7 DATUM SHIFT, 10 ROTATION, 12 PGM CALL; old 1 PECKING and 5 CIRCULAR POCKET | 313 509-22 / 331 645-22 |
| Subprograms | `LBL n`, `CALL LBL n REP ..` | - |
| Q parameters | `FN 0..` arithmetic and jumps | 331 645-22 ("PARAMETER FUNCTIONS") |
| Comment | `; text` block | all manuals |

Number format: signed coordinates (`X+10`), 3 decimals on 1 µm controls, 4 decimals on 0.1 µm controls (see B).

---

## B. Per series: what the baseline (oldest manual on the portal) adds to the standard library

| Series / control | Oldest manual on disk | Adds | Lacks (not found in the extracted text) |
|---|---|---|---|
| TNC 124 | 246 xxx-08, 284 679-24 | Own positioning dialect (A.1); program numbers; cycles 1.0 PECKING, 2.0 TAPPING (floating holder), hole patterns | L, arcs, BLK FORM, Q parameters, DIN/ISO |
| TNC 128 | 77184x-01, 819494-20 (2013) | Paraxial blocks with BLK FORM, `TOOL CALL n Z S F DL DR DR2` (819494-26 printed p? "TOOL CALL 5.2 Z S2500 F350 DL+0.2 DR-1 DR2+0.05"), cycles 200-207, 240, 241, 247, 251, 256, FN/QS/QL, M136/M137, M140 | arcs, DIN/ISO, PLANE, TCPM |
| TNC 310 | 286 040, 313 509-22 | standard library; 1 µm input (3 decimals); program names up to 8 characters (331 645-22 line 1348); memory 128 KB / about 6000 blocks / 64 files (331 645-22 printed p227) | DIN/ISO ("HEIDENHAIN conversational programming" is the only file type, printed p227), TOOL CALL by name, M126/M128/M140, cycle 19 (WORKING PLANE), FAUTO, QS |
| TNC 410 | 286 060/286 080, 309 740-25 (6/2001) + ISO 345 940-22 | DIN/ISO; old cycles 1 (PECKING)/2 (TAPPING)/3 (SLOT MILLING)/4 (POCKET MILLING)/5 (CIRCULAR POCKET)/17 (RIGID TAPPING GS); 200-209; SL 20-24; cycle 19 (on the TNC 410 only for the test graphics? not checked); M120 look-ahead; M126; TCH PROBE; 1 µm input ("TNC 410: 1 µm", 340 135-22 spec) | TOOL CALL by name (345 940-22: "type the tool name in quotation marks (not TNC 410)"), M140, PLANE, QS |
| TNC 426 / TNC 430 | 280 460 (scan, not analysed); 280 462, 311 062-21 (4/97) | TOOL CALL by name in quotes (from 280 462); cycle 19; old cycles 1-5/17/18; M112/M114/M116/M118/M120/M126; FK; LN (3-D comp); 0.1 µm input (4 decimals) | DIN/ISO before 280 472 (the ISO manual 318 179-24 covers 280 472-475; earlier: not found); M128 before 280 472 (first mention 280 472); M140; PLANE; FUNCTION TCPM; QS |
| iTNC 530 | 340 420, 360 994-23 / 369 475-21 | Klartext + DIN/ISO; old cycles 1-5/17/18 still definable; 200-series incl. Q211; cycle 32 TOLERANCE; cycle 247 DATUM SETTING (datum table based); M128/M129/M144/M140/M136/M137/M138; SPLINE; FAUTO; 0.1 µm input | PLANE, FUNCTION TCPM (both come with 340 422), FZ/FU, QS, QL/QR, label names, FUNCTION DWELL (never on iTNC 530) |
| TNC 320 | 340 551-01, 550 671-20 | 200-209, SL 20-24, 4/5 pockets; QS strings from the start; M140; FN 16/18; SQL | DIN/ISO before 340 55x-04 (no ISO manual; ISO keywords only 6 times in the -03 manual: unsure); cycle 19 and preset table before 340 55x-03; PLANE before -04; FUNCTION TCPM / M128 5-axis (option 9 is not offered for the TNC 320, 827222-27 p3) |
| TNC 620 | 340 56x-01, 636 ... (User's Manual) | as TNC 320 -03 plus cycle 19, M128, cycle 32 | DIN/ISO in -01 (no ISO manual: unsure); PLANE before -02; FUNCTION TCPM before -03 |
| TNC 640 | 34059x-01, 892903/892905/892909-20 | Everything of the iTNC 530 60642x line (PLANE all forms, FUNCTION TCPM in Klartext, QS/QL/QR, label names, TOOL CALL by name), FUNCTION MODE MILL/TURN, turning cycles 8xx | FUNCTION TCPM in ISO mode before -10; FUNCTION DWELL before -07; old cycles 1/2/17 (not in the cycle list; the manuals mention them only in conversion notes) |
| TNC7 | 81762x-16, 1358773-20 | TNC 640 -16 functions, new UI; TRANS DATUM/MIRROR/ROTATION, FUNCTION DWELL, PRESET SELECT, OCM | DIN/ISO in -16 ("With NC software 81762x-16, the TNC7 does not support ISO programming", 1358773-20 line 4071 of the text); cycle 19 cannot be inserted (from -17); cycle 7 (DATUM SHIFT) is converted to TRANS DATUM on edit (1358773-24 printed p? "The TRANS DATUM function replaces Cycle 7 DATUM SHIFT") |

Not covered: TNC 406 / TNC 416 (280 620/280 621/286 180) are EDM sinking controls (electrode programming) and are out of scope.

---

## B2. Post-dialect features: introduced / changed / removed (the summary the posts need)

Abbreviations: iTNC = iTNC 530, 640 = TNC 640, 620 = TNC 620, 320 = TNC 320, 7 = TNC7.

### Program structure and number format

| Feature | Status | Source |
|---|---|---|
| DIN/ISO programs (`%NAME G71 *` ... `N99999999 %NAME G71 *`) | TNC 410: yes. TNC 426/430: from 280 472 (earlier: not found). iTNC: all versions. 320: from 340 55x-04. 620: from 340 56x-02 (-01: not found). 640: all. TNC7: from 81762x-17. Never: TNC 124, TNC 128, TNC 310. | 345 940-22; 318 179-24 title; 679 226-20; TNC7 1373081_03 p17 ("You can run and edit ISO programs"); 1358773-20 (not supported in -16) |
| Coordinate decimals | 1 µm (3 decimals): TNC 310, TNC 410, TNC 124. 0.1 µm (4 decimals): TNC 426/430, iTNC, 320 (unsure for 340 551), 620, 640, TNC7, TNC 128. 0.01 µm display step: option 23 on 620/640 only. | 331 645-22 printed p228; 340 135-22 spec ("TNC 410: 1 µm"); brochures 892916-28 / 896140-29 "Comparison of controls"; 827222-27 p3 |
| Program name | TNC 124: number only. TNC 310: name up to 8 characters. iTNC: names longer than 25 characters are not fully displayed; the characters `; * \ / " ? < > .` are not permitted in names (340 49x-03 manual text line 3744). TNC 640: path up to 256 characters. | 284 679-24; 331 645-22; bhb 340 49x-03; 892903-2C |
| Max size | TNC 124: 1000 blocks/program; TNC 310: about 6000 blocks, 128 KB, 64 files; 620: 1.8-2 GB; 640 and iTNC (HSCI): > 21 GB | 284 679-24; 331 645-22 p227; brochures |
| Q formula priority | Changed on 640 in 34059x-11: "The priority of arithmetic operations has been changed in the Q parameter formula". A post that writes Q formulas must use parentheses. | 1322095-02 p129 |
| Control structures `IF/ELSE/FOR/WHILE` | TNC7 81762x-19 only | 1373081_03 p54 |

### TOOL CALL / TOOL DEF

| Feature | Status | Source |
|---|---|---|
| `TOOL CALL n Z S..` | all controls | - |
| Tool name in quotes `TOOL CALL "NAME" Z` | TNC 426 from 280 462; iTNC all; 320/620/640/7 all; TNC 128 yes. Not TNC 410, not TNC 310 (not found). | 311 062-21; 345 940-22 ("(not TNC 410)") |
| Name length | 640: 16 → 32 characters in 34059x-02 ("TOOL DEF blocks (ISO: T) or TOOL CALL blocks"). | 1322095-02 p24 |
| `F` / `DL` / `DR` / `DR2` in TOOL CALL | DL/DR in TOOL CALL: all contouring controls. DR2: TNC 410 / 426 and later. DL/DR/DR2 range widened to 999.9999 mm: iTNC 34049x-07 / 60642x-02. | 34049x-08 Klartext p21 |
| Cutting speed `VC` instead of S | iTNC 34049x-02 | 670387-23 p12 |
| FZ / FU feed | iTNC 34049x-01; in cycles: 320 340 55x-04, 620 340 56x-02; in TOOL CALL on TNC 128: 77184x-03 | 670387-23 p11; 679 220-20 p7 |
| Tool name from a QS parameter | iTNC 60642x-04 | 737759-24 p20 |
| Same tool + same axis = no tool-change macro | 620 81760x-06 (640: same release line, 34059x-09: unsure) | 1322094_03 p72 |
| `TOOL DEF n L.. R..` | TNC7 81762x-19: "You can no longer program L and R in the TOOL DEF NC function. The control issues an error message during execution." | 1373081_03 p63 |

### Lines, arcs, compensation, feed

| Feature | Status | Source |
|---|---|---|
| L, CC/C, CR, CT, CP, LP, RND, CHF, RL/RR/R0 | all contouring controls (TNC 310 on). Not TNC 124/128. | A |
| Arcs outside the active plane as spatial arcs | iTNC 34049x-03 ("Arcs that are not programmed in the active working plane can now also be run as spatial arcs"); 640 34059x-07; 620 81760x-04 | 533 190-23 p16; 1322095-02 p65; 1322094_03 p48 |
| Helix by `LIN_` in C/CR/CT; third axis on G02/G03/G05 in ISO | 640 34059x-16; 620 81760x-16; TNC7 81762x-17. Before: helix only as polar `CP` (Klartext) / polar G12/G13 (ISO). | 1322094_03 p94; 1373081_03 p16 |
| Input range of polar angle increased | iTNC 34049x-04 | 533 190-23 p17 |
| Up to 6 axes in one L block | 640 34059x-07 | 1322095-02 (34059x-07 modified functions) |
| FAUTO | iTNC and later; not found on TNC 310/410/426 | feature-mentions.md |
| M136 + FU in inch programs | not combinable from iTNC 34049x-02 | 670387-23 p27 |

### M functions

| M | Status | Source |
|---|---|---|
| M91 / M92 / M94 / M97 / M99 / M89 | all contouring controls; M89 + FK refused from 620 81760x-06 | 1322094_03 p72 |
| M126 (shortest path rotary) | TNC 410, 426 and later | feature-mentions.md |
| M128 / M129 | TNC 426/430 from 280 472, TNC 410 ISO manual, iTNC all, 620/640/7. TNC 320: no 5-axis option (option 9 not offered). M128 for block scan: iTNC 34049x-01. Tool change with M128/TCPM active: iTNC 34049x-07 / 60642x-02. M116 with M128: iTNC 34049x-05. | 827222-27 p3; 670387-23 p11/p16/p21 |
| M140 (retract along tool axis, `MB MAX`) | iTNC from 340 420; 320/620/640/7; TNC 128. Not found on TNC 310/410/426. | 360 994-23 p20 |
| M150 | iTNC 34049x-01 | 670387-23 p11 |
| M124 without T switches it off | iTNC 34049x-07 | 670387-23 p32 |
| M197 | 640 34059x-02; 620 73498x-02 | 1322095-02 p21 |
| M109/M110 with M120 changes | 640 34059x-11; 620 81760x-08 | 1322094_03 p90 |

### Cycles

| Feature | Status | Source |
|---|---|---|
| Old cycles 1 PECKING, 2 TAPPING, 3 SLOT, 4/5 POCKET, 17 RIGID TAPPING, 18 THREAD CUTTING | TNC 310 (1, 5), TNC 410, TNC 426/430, iTNC 340 420: definable. iTNC 340 422: "The old fixed cycles 1, 2, 3, 4, 5, 17 and 18 were removed from the soft-key structure and can therefore no longer be defined. Old programs containing these cycles can still be run". TNC 320/620/640/TNC7: not in the cycle lists (not found). | 375 738-24 p10 |
| 200-series drilling (200 DRILLING, 201 REAMING, 202 BORING, 203 UNIVERSAL DRILLING, 204 BACK BORING, 205 UNIVERSAL PECKING, 206 TAPPING, 207 RIGID TAPPING, 208 BORE MILLING, 209 TAPPING W/ CHIP BRKG) | 200/201: TNC 310 and TNC 426 280 462 on. 203/205/206/207/209: TNC 410, TNC 426/430 280 472 on, iTNC all, 320/620/640/7 all. On TNC 620, 201-205, 208 and 209 need option 19 ("Cycle 205 UNIVERSAL PECKING (option 19)"); 200, 206, 207 are standard. TNC 320/640/iTNC: option 19 is standard. | 1303427-23 (620 81760x-18 machining cycles) headings; 827222-27 p3 |
| 210-214 pockets/slots (210 SLOT RECIP. PLNG, 211 CIRCULAR SLOT, 212 POCKET FINISHING, 213 STUD FINISHING, 214 C. POCKET FINISHING) | removed from the soft keys (still run): iTNC 34049x-04, 320 340 55x-04, 620 340 56x-02. Replaced by 251-257 (251 RECTANGULAR POCKET, 252 CIRCULAR POCKET, 253 SLOT MILLING, 254 CIRCULAR SLOT, 256 RECTANGULAR STUD, 257 CIRCULAR STUD; iTNC 340 422, 320 -04, 620 -02, 640 -01). | 533 190-23 p17 |
| Cycle 200 (DRILLING) Q list | Q200 Q201 Q206 Q202 Q210 Q203 Q204 (TNC 310, TNC 426 ≤280 472); + Q211 (TNC 426/430 280 474, iTNC all, 320/620/640 all); + **Q395** (iTNC 60642x-04 only; 640 34059x-04; 620 81760x-01; 320 77185x-01; TNC 128 77184x-02). | section B3 |
| Cycle 205 (UNIVERSAL PECKING) Q list | Q200 Q201 Q206 Q202 Q203 Q204 Q212 Q205 Q258 Q259 (TNC 320 340 551); + Q257 Q256 Q211 (TNC 426/430 280 472, iTNC 340 420); + **Q379 Q253** (iTNC 340 422; 640 -01; 620 340 56x-01; 320 34055x-06); + **Q208 Q395** (iTNC 60642x-04 only; 640 34059x-04; 620 81760x-01; 320 77185x-05); + Q373 (640 34059x-16; 320 771851-18; TNC7 -17). | iTNC 670388-25 cycle 205; "Cycle 205: Input parameter Q208 for retraction feed rate added" (737759-24 p20 / 670388-25 p13) |
| Cycle 207 (RIGID TAPPING) / 209 (TAPPING W/ CHIP BRKG) | 207: Q200 Q201 Q239 Q203 Q204 (stable). 209: Q200 Q201 Q239 Q203 Q204 Q257 Q256 Q336 + Q403 (retraction speed factor, iTNC 34049x-04). Pitch from the tool table PITCH column monitored for 206/207/209: iTNC 60642x-02/-04. | 670388-25 p15 |
| Depth reference parameter | The iTNC 60642x-04 list names it "Q359" for cycles 200/203 (UNIVERSAL DRILLING)/205; the example blocks of the same manuals show `Q395`. Treat as Q395 (**unsure**: typo in the list). | 737759-24 p20 vs 670388-24 p99 |
| Cycle 247 DATUM SETTING | iTNC 340 422: "Cycle 247 now activates a preset from the preset table" (before: a datum value). TNC 128 77184x-05: preset number selectable. TNC 320 from 340 55x-03 (preset table). | 375 738-24 p10; 819494-24 p20 |
| PRESET SELECT | 640 34059x-11 / 620 81760x-08 and later; TNC7 | feature-mentions.md |
| Cycle 32 TOLERANCE (ISO G62) | TNC 410/426 280 474 and later, iTNC all, 320 from 340 55x-03, 620, 640, 7 | feature-mentions.md |
| Cycle 9 DWELL TIME (ISO G04) | all contouring controls | 670 391 (iTNC 49x-03) "DWELL TIME (Cycle G04)"; 1303406-23 p? "Cycle 9 DWELL TIME" |
| `FUNCTION DWELL` | 640 34059x-07; 620 81760x-04; 320 77185x-04; TNC 128 77184x-05; TNC7 all. **iTNC 530: not found in any version.** | 1322095-02 p65; 1322094_03 p48; 819494-24 p14 |
| `FUNCTION FEED DWELL` (cyclic) | 640 34059x-05; 620 81760x-02; TNC 128 77184x-03 | 1322095-02 p44 |
| Cycles 7 (DATUM SHIFT) / 8 (MIRRORING) / 10 (ROTATION) / 11 (SCALING FACTOR) / 26 (AXIS-SPECIFIC SCALING) | all contouring controls (11/26: TNC 410/426 on). TRANS DATUM ("Define coordinate transformations"): 620 340 56x-03, 320 340 55x-05, iTNC 34049x-04 (unsure). TRANS MIRROR / TRANS ROTATION: 640 34059x-16, 620 81760x-16. TRANS RESET: -18. TNC7: cycle 7 is changed into TRANS DATUM when edited. | 1322094_03 p95/p108; 1358773-24 |

### Tilted working plane and TCPM

| Feature | Status | Source |
|---|---|---|
| Cycle 19 WORKING PLANE (ISO G80) | TNC 410 (unsure), TNC 426 280 462 on, iTNC all, 320 from 340 55x-03 (option), 620, 640. TNC7 from 81762x-17: "You can edit and execute Cycle 19 ... but you cannot insert it into an NC program as a new element". | 550 671-22 p7; 1373081_03 p23 |
| PLANE SPATIAL / PROJECTED / EULER / VECTOR / POINTS / RELATIV / RESET + MOVE/TURN/STAY | iTNC 340 422; 620 340 56x-02; 320 340 55x-04; 640 and TNC7 all. TNC 410/426/310: not found. | 375 738-24 p7; 679 222-20; 679 220-20 p8 |
| PLANE AXIAL | iTNC 34049x-03; 640/620 all; 320 (from -04, unsure) | 533 190-23 p12 |
| FMAX in MOVE/TURN | iTNC 34049x-03 | 533 190-23 p16 |
| TURN with clearance height | iTNC 34049x-06 / 60642x-01 | 670387-23 p18 |
| SEQ from a Q parameter | iTNC 34049x-08 / 60642x-03; 640 34059x-02; 620 73498x-02 | 1322095-02 p25 |
| SYM instead of SEQ | 640 34059x-09; 620 81760x-06; TNC7 | 1322095-02 p96 |
| `MOVE ... ABST` | TNC7 81762x-20 executes old `ABST` and changes it to `DIST` when edited (compatibility). | 1373081_03 p85 |
| `FUNCTION TCPM F TCP AXIS ... PATHCTRL ...` | Klartext: iTNC 340 422 (first mention; the iTNC 340 422 list does not name it: unsure), 620 340 56x-03 ("The TCPM function was introduced"), 640 all, TNC7 all. **ISO mode: 640 34059x-10, 620 81760x-07** ("FUNCTION TCPM (option 9) is available in ISO programming mode"). iTNC ISO manual lists FUNCTION TCPM in its feature summary only; the ISO form was not found (unsure: use M128 in iTNC ISO). TNC 320: none (option 9 not offered). | 1322095-02 p109; 1322094_03 p77; 737760-24 summary |
| M128 | see M functions. Still accepted on 620/640/TNC7; FUNCTION TCPM is "an improvement over M128" (1322095-02 p109). | - |

### Q parameters

| Feature | Status | Source |
|---|---|---|
| Q parameters / FN 0-FN 12 | all contouring controls | - |
| 2000 Q parameters; label names; FN 9-12 jump to label names | iTNC 34049x-01 | 670387-23 p11 |
| QS strings | iTNC 34049x-03; 320 from 340 551-01; 620/640/7 all | 533 190-23 p12 |
| QL / QR | iTNC 34049x-05; 620 340 56x-03; 320 340 55x-05 | 670387-23 p16; 679 220-21 p10 |
| QS as jump target | iTNC 34049x-06 / 60642x-01 | 670387-23 p18 |
| `SEL PGM` with QS | iTNC 34049x-07 / 60642x-02 | 670387-23 p21 |
| FN 15, FN 31, FN 32, FT, FMAXT (old controls) | TNC 128 77184x-03: imported without ERROR block, but the program stops with an error when it reaches them | 819494-24 p13 |

---

## B3. Cycle Q-parameter lists by version (extracted from example blocks)

Format: `cycle control/version Q-list (PDF page, manual)`. Only cycles 200 (DRILLING)/203 (UNIVERSAL DRILLING)/205 (UNIVERSAL PECKING)/207 (RIGID TAPPING)/209 (TAPPING W/ CHIP BRKG) and the controls iTNC 530, TNC 320, TNC 620, TNC 640 and TNC7 are listed here. The full extraction (all controls; also 201, 206, 240, 241, 251-257, 262) is the script output (not published). A list can be short when a page break cut the example (for example iTNC 340 49x-01 cycle 200 is complete, but some lists end early); compare neighbouring versions before you trust a missing parameter.

```

200 TNC7/81762x-16 200 201 206 202 210 203 204 211 395 (p386 Programming_and_Testing__13587)
200 TNC7/81762x-17 200 201 206 202 210 203 204 211 395 (p85 Machining_cycles__1358775-21.t)
200 TNC7/81762x-18 200 201 206 202 210 203 204 211 395 (p135 Machining_cycles__1358775-22.t)
200 TNC7/81762x-19 200 201 206 202 210 203 204 211 395 (p127 Machining_cycles__1358775-23.t)
200 TNC7/81762x-20 200 201 206 202 210 203 204 211 395 (p145 Machining_cycles__1358775-24.t)
200 vTNC7/81762x-20_SP1 200 201 206 202 210 203 204 211 395 (p473 User_documentation__1505672_00)
200 TNC_320/34055x-06 200 201 206 202 210 203 204 211 (p49 ISO_programming__679226-22.txt)
200 TNC_320/340_551-01 200 201 206 202 210 203 204 211 (p50 User_s_Manual_NC_software__550)
200 TNC_320/340_551-02 200 201 206 202 210 203 204 211 (p50 User_s_Manual_NC_software__550)
200 TNC_320/340_55x-03 200 201 206 202 210 203 204 211 (p67 User_s_Manual_NC_software__550)
200 TNC_320/340_55x-04 200 201 206 202 210 203 204 211 (p45 Conversational_programming__67)
200 TNC_320/340_55x-05 200 201 206 202 210 203 204 211 (p43 Cycle_programming__679_220-21.)
200 TNC_320/771851-18 200 201 206 202 210 203 204 211 395 (p78 Klartext_programming__1096950-)
200 TNC_320/77185x-01 200 201 206 202 210 203 204 211 395 (p89 Cycle_programming__1096959-20.)
200 TNC_320/77185x-02 200 201 206 202 210 203 204 211 395 (p45 Cycle_programming__1096959-21.)
200 TNC_320/77185x-04 200 201 206 202 210 203 204 211 395 (p47 Cycle_programming__1096959-22.)
200 TNC_320/77185x-05 200 201 206 202 210 203 204 211 395 (p47 Cycle_programming__1096959-23.)
200 TNC_320/77185x-06 200 201 206 202 210 203 204 211 395 (p52 Klartext_programming__1096950-)
200 iTNC_530/34049x-08 200 201 206 202 210 203 204 211 (p269 ISO_programming__670391-23.txt)
200 iTNC_530/340_420 216 217 244 245 246 247 241 200 203 204 (p294 User_s_Manual_Date__369_475-21)
200 iTNC_530/340_422 216 217 244 245 246 247 241 200 203 204 365 (p325 User_s_Manual_Date__389_512-20)
200 iTNC_530/340_49x-01 216 217 244 245 246 247 241 200 203 204 365 (p334 User_s_Manual_NC_software__533)
200 iTNC_530/340_49x-02 216 217 244 245 246 247 241 200 203 204 365 (p359 User_s_Manual_NC_software__din)
200 iTNC_530/340_49x-03 216 217 244 245 246 247 241 200 203 204 365 (p377 User_s_Manual_NC_software__din)
200 iTNC_530/340_49x-04 216 217 244 245 246 247 241 200 203 204 365 (p390 User_s_Manual_NC_software__533)
200 iTNC_530/340_49x-05 200 201 206 202 210 203 204 211 (p63 Conversational_programming__67)
200 iTNC_530/340_49x-06 200 201 206 202 210 203 204 211 (p69 Conversational_programming__67)
200 iTNC_530/340_49x-07 200 201 206 202 210 203 204 211 (p71 Conversational_programming__67)
200 iTNC_530/60642x-03 200 201 206 202 210 203 204 211 (p273 ISO_programming__737760-22.txt)
200 iTNC_530/60642x-04 200 201 206 202 210 203 204 211 395 (p74 Cycle_programming__670388-24.t)
200 iTNC_530/60642x-04_SP8 200 201 206 202 210 203 204 211 395 (p76 Cycle_programming__670388-25.t)
200 iTNC_530/606_42x-01 200 201 206 202 210 203 204 211 (p49 Cycle_programming__670_388-21.)
200 iTNC_530/606_42x-02 200 201 206 202 210 203 204 211 (p269 DIN_ISO_programming__737_760-2)
200 TNC_620/34056x-04 200 201 206 202 210 203 204 211 (p53 ISO_programming__679355-22.txt)
200 TNC_620/340_56x-01 200 201 206 202 210 203 204 211 (p69 User_s_Manual_NC_software__636)
200 TNC_620/340_56x-02 200 201 206 202 210 203 204 211 (p47 Conversational_programming__67)
200 TNC_620/340_56x-03 200 201 206 202 210 203 204 211 (p225 Conversational_programming__67)
200 TNC_620/73498x-01 200 201 206 202 210 203 204 211 (p44 DIN_ISO_programming__819500-20)
200 TNC_620/73498x-02 200 201 206 202 210 203 204 211 (p55 Klartext_programming__819499-2)
200 TNC_620/81760x-01 200 201 206 202 210 203 204 211 395 (p92 Cycle_programming__1096886-20.)
200 TNC_620/81760x-02 200 201 206 202 210 203 204 211 395 (p59 Klartext_programming__1096883-)
200 TNC_620/81760x-03 200 201 206 202 210 203 204 211 395 (p63 Klartext_programming__1096883-)
200 TNC_620/81760x-04 200 201 206 202 210 203 204 211 395 (p67 Klartext_programming__1096883-)
200 TNC_620/81760x-05 200 201 206 202 210 203 204 211 395 (p51 Cycle_programming__1096886-24.)
200 TNC_620/81760x-06 200 201 206 202 210 203 204 211 395 (p51 Cycle_programming__1096886-25.)
200 TNC_620/81760x-07 200 201 206 202 210 203 204 211 395 (p53 Cycle_programming__1096886-26.)
200 TNC_620/81760x-08 200 201 206 202 210 203 204 211 395 (p47 Machining_cycles__1303427-20.t)
200 TNC_620/81760x-16 200 201 206 202 210 203 204 211 395 (p42 Machining_cycles__1303427-21.t)
200 TNC_620/81760x-17 200 201 206 202 210 203 204 211 395 (p40 Machining_cycles__1303427-22.t)
200 TNC_620/81760x-18 200 201 206 202 210 203 204 211 395 (p40 Machining_cycles__1303427-23.t)
200 TNC_640/34059x-01 200 201 206 202 210 203 204 211 (p44 DIN_ISO_programming__892909-20)
200 TNC_640/34059x-02 200 201 206 202 210 203 204 211 (p57 Klartext_programming__892903-2)
200 TNC_640/34059x-04 200 201 206 202 210 203 204 211 395 (p98 Cycle_programming__892905-22.t)
200 TNC_640/34059x-05 200 201 206 202 210 203 204 211 395 (p418 Klartext_programming__892903-2)
200 TNC_640/34059x-06 200 201 206 202 210 203 204 211 395 (p67 Klartext_programming__892903-2)
200 TNC_640/34059x-07 200 201 206 202 210 203 204 211 395 (p73 Klartext_programming__892903-2)
200 TNC_640/34059x-08 200 201 206 202 210 203 204 211 395 (p80 Klartext_programming__892903-2)
200 TNC_640/34059x-09 200 201 206 202 210 203 204 211 395 (p62 Klartext_programming__892903-2)
200 TNC_640/34059x-10 200 201 206 202 210 203 204 211 395 (p63 Cycle_programming__892905-28.t)
200 TNC_640/34059x-11 200 201 206 202 210 203 204 211 395 (p68 ISO_programming__892909-29.txt)
200 TNC_640/34059x-16 200 201 206 202 210 203 204 211 395 (p70 Klartext_programming__892903-2)
200 TNC_640/34059x-17 200 201 206 202 210 203 204 211 395 (p64 Klartext_programming__892903-2)
200 TNC_640/34059x-18 200 201 206 202 210 203 204 211 395 (p64 Klartext_programming__892903-2)
203 TNC7/81762x-17 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p106 Machining_cycles__1358775-21.t)
203 TNC7/81762x-18 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p203 Machining_cycles__1358775-22.t)
203 TNC7/81762x-19 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p197 Machining_cycles__1358775-23.t)
203 TNC7/81762x-20 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p215 Machining_cycles__1358775-24.t)
203 vTNC7/81762x-20_SP1 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p538 User_documentation__1505672_00)
203 TNC_320/34055x-06 200 201 206 202 211 203 204 212 213 205 208 256 (p71 Cycle_programming__679220-22.t)
203 TNC_320/340_551-01 200 201 206 202 210 203 204 212 213 205 211 208 256 (p189 User_s_Manual_NC_software__550)
203 TNC_320/340_551-02 200 201 206 202 210 203 204 212 213 205 211 208 256 (p195 User_s_Manual_NC_software__550)
203 TNC_320/340_55x-03 200 201 206 202 210 203 204 212 213 205 211 208 256 (p224 User_s_Manual_NC_software__550)
203 TNC_320/340_55x-04 200 201 206 202 210 203 204 212 213 208 205 211 256 (p72 Cycle_programming__679_220-20.)
203 TNC_320/340_55x-05 200 206 202 210 203 204 212 213 208 205 211 256 (p74 Cycle_programming__679_220-21.)
203 TNC_320/771851-18 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p93 Machining_cycles__1303429-20.t)
203 TNC_320/77185x-01 200 201 206 202 211 203 204 212 213 205 208 256 395 (p75 Cycle_programming__1096959-20.)
203 TNC_320/77185x-02 200 201 206 202 211 203 204 212 213 205 208 256 395 (p73 Cycle_programming__1096959-21.)
203 TNC_320/77185x-04 200 201 206 202 211 203 204 212 213 205 208 256 395 (p75 Cycle_programming__1096959-22.)
203 TNC_320/77185x-05 200 201 206 202 211 203 204 212 213 205 208 256 395 (p80 Cycle_programming__1096959-23.)
203 TNC_320/77185x-06 200 201 206 202 211 203 204 212 213 205 208 256 395 (p82 Cycle_programming__1096959-24.)
203 iTNC_530/34049x-08 200 201 206 202 210 203 204 212 213 208 205 211 256 (p90 Cycle_programming__670388-23_S)
203 iTNC_530/340_420 200 201 206 202 210 203 204 212 213 205 211 208 256 (p252 User_s_Manual_Date__360_994-23)
203 iTNC_530/340_422 200 201 206 202 210 203 204 212 213 205 211 208 256 (p239 User_s_Manual_Date__389_512-20)
203 iTNC_530/340_49x-01 200 201 206 202 210 203 204 212 213 205 211 208 256 (p256 User_s_Manual_NC_software__533)
203 iTNC_530/340_49x-02 200 201 206 202 210 203 204 212 213 205 211 208 256 (p314 User_s_Manual_NC_software__bhb)
203 iTNC_530/340_49x-03 200 201 206 202 210 203 204 212 213 205 211 208 256 (p342 User_s_Manual_NC_software__bhb)
203 iTNC_530/340_49x-04 200 201 206 202 210 203 204 212 213 205 211 208 256 (p318 User_s_Manual_NC_software__533)
203 iTNC_530/340_49x-05 200 201 206 202 210 203 204 212 213 208 205 211 256 (p82 Cycle_programming__670_388-20.)
203 iTNC_530/340_49x-06 200 201 206 202 210 203 204 212 213 208 205 211 256 (p86 Cycle_programming__670_388-21.)
203 iTNC_530/340_49x-07 200 201 206 202 210 203 204 212 213 208 205 211 256 (p88 Cycle_programming__670_388-22.)
203 iTNC_530/60642x-03 200 201 206 202 210 203 204 212 213 208 205 211 256 (p90 Cycle_programming__670388-23_S)
203 iTNC_530/60642x-04 200 201 206 202 210 203 204 212 213 208 205 211 256 395 (p84 Cycle_programming__670388-24.t)
203 iTNC_530/60642x-04_SP8 200 201 206 202 210 203 204 212 213 208 205 211 256 395 (p86 Cycle_programming__670388-25.t)
203 iTNC_530/606_42x-01 200 201 206 202 210 203 204 212 213 208 205 211 256 (p86 Cycle_programming__670_388-21.)
203 iTNC_530/606_42x-02 200 201 206 202 210 203 204 212 213 208 205 211 256 (p88 Cycle_programming__670_388-22.)
203 TNC_620/34056x-04 200 201 206 202 211 203 204 212 213 205 208 256 (p75 Cycle_programming__679295-22.t)
203 TNC_620/340_56x-01 200 201 206 202 210 203 204 212 213 205 211 208 256 (p234 User_s_Manual_NC_software__636)
203 TNC_620/340_56x-02 200 201 206 202 210 203 204 212 213 208 205 211 256 (p72 Cycle_programming__679_295-20.)
203 TNC_620/340_56x-03 200 206 202 210 203 204 212 213 208 205 211 256 (p74 Cycle_programming__679_295-21.)
203 TNC_620/73498x-01 200 206 202 210 203 204 212 213 208 205 211 256 (p74 Cycle_programming__679_295-21.)
203 TNC_620/73498x-02 200 201 206 202 211 203 204 212 213 205 208 256 (p75 Cycle_programming__679295-22.t)
203 TNC_620/81760x-01 200 201 206 202 211 203 204 212 213 205 208 256 395 (p77 Cycle_programming__1096886-20.)
203 TNC_620/81760x-02 200 201 206 202 211 203 204 212 213 205 208 256 395 (p79 Cycle_programming__1096886-21.)
203 TNC_620/81760x-03 200 201 206 202 211 203 204 212 213 205 208 256 395 (p81 Cycle_programming__1096886-22.)
203 TNC_620/81760x-04 200 201 206 202 211 203 204 212 213 205 208 256 395 (p81 Cycle_programming__1096886-23.)
203 TNC_620/81760x-05 200 201 206 202 211 203 204 212 213 205 208 256 395 (p84 Cycle_programming__1096886-24.)
203 TNC_620/81760x-06 200 201 206 202 211 203 204 212 213 205 208 256 395 (p86 Cycle_programming__1096886-25.)
203 TNC_620/81760x-07 200 201 206 202 211 203 204 212 213 205 208 256 395 (p86 Cycle_programming__1096886-26.)
203 TNC_620/81760x-08 200 201 206 202 211 203 204 212 213 205 208 256 395 (p83 Machining_cycles__1303427-20.t)
203 TNC_620/81760x-16 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p84 Machining_cycles__1303427-21.t)
203 TNC_620/81760x-17 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p82 Machining_cycles__1303427-22.t)
203 TNC_620/81760x-18 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p83 Machining_cycles__1303427-23.t)
203 TNC_640/34059x-01 200 206 202 210 203 204 212 213 208 205 211 256 (p74 Cycle_programming__892905-20.t)
203 TNC_640/34059x-02 200 201 206 202 211 203 204 212 213 205 208 256 (p81 Cycle_programming__892905-21.t)
203 TNC_640/34059x-04 200 201 206 202 211 203 204 212 213 205 208 256 395 (p83 Cycle_programming__892905-22.t)
203 TNC_640/34059x-05 200 201 206 202 211 203 204 212 213 205 208 256 395 (p85 Cycle_programming__892905-23.t)
203 TNC_640/34059x-06 200 201 206 202 211 203 204 212 213 205 208 256 395 (p87 Cycle_programming__892905-24.t)
203 TNC_640/34059x-07 200 201 206 202 211 203 204 212 213 205 208 256 395 (p89 Cycle_programming__892905-25.t)
203 TNC_640/34059x-08 200 201 206 202 211 203 204 212 213 205 208 256 395 (p92 Cycle_programming__892905-26.t)
203 TNC_640/34059x-09 200 201 206 202 211 203 204 212 213 205 208 256 395 (p94 Cycle_programming__892905-27.t)
203 TNC_640/34059x-10 200 201 206 202 211 203 204 212 213 205 208 256 395 (p96 Cycle_programming__892905-28.t)
203 TNC_640/34059x-11 200 201 206 202 211 203 204 212 213 205 208 256 395 (p94 Machining_cycles__1303406-20.t)
203 TNC_640/34059x-16 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p96 Machining_cycles__1303406-21.t)
203 TNC_640/34059x-17 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p90 Machining_cycles__1303406-22.t)
203 TNC_640/34059x-18 200 201 206 202 210 203 204 212 213 205 211 208 256 395 (p93 Machining_cycles__1303406-23.t)
205 TNC7/81762x-17 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p115 Machining_cycles__1358775-21.t)
205 TNC7/81762x-18 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p209 Machining_cycles__1358775-22.t)
205 TNC7/81762x-19 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p202 Machining_cycles__1358775-23.t)
205 TNC7/81762x-20 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p220 Machining_cycles__1358775-24.t)
205 vTNC7/81762x-20_SP1 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p543 User_documentation__1505672_00)
205 TNC_320/34055x-06 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p77 Cycle_programming__679220-22.t)
205 TNC_320/340_551-01 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p194 User_s_Manual_NC_software__550)
205 TNC_320/340_551-02 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p200 User_s_Manual_NC_software__550)
205 TNC_320/340_55x-03 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p230 User_s_Manual_NC_software__550)
205 TNC_320/340_55x-04 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p80 Cycle_programming__679_220-20.)
205 TNC_320/340_55x-05 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p82 Cycle_programming__679_220-21.)
205 TNC_320/771851-18 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p103 Machining_cycles__1303429-20.t)
205 TNC_320/77185x-01 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p82 Cycle_programming__1096959-20.)
205 TNC_320/77185x-02 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p80 Cycle_programming__1096959-21.)
205 TNC_320/77185x-04 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p83 Cycle_programming__1096959-22.)
205 TNC_320/77185x-05 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p88 Cycle_programming__1096959-23.)
205 TNC_320/77185x-06 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p90 Cycle_programming__1096959-24.)
205 iTNC_530/34049x-08 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p98 Cycle_programming__670388-23_S)
205 iTNC_530/340_420 200 201 206 202 203 204 212 205 258 259 257 256 211 (p257 User_s_Manual_Date__360_994-23)
205 iTNC_530/340_422 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p245 User_s_Manual_Date__389_512-20)
205 iTNC_530/340_49x-01 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p262 User_s_Manual_NC_software__533)
205 iTNC_530/340_49x-02 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p319 User_s_Manual_NC_software__bhb)
205 iTNC_530/340_49x-03 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p348 User_s_Manual_NC_software__bhb)
205 iTNC_530/340_49x-04 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p323 User_s_Manual_NC_software__533)
205 iTNC_530/340_49x-05 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p90 Cycle_programming__670_388-20.)
205 iTNC_530/340_49x-06 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p94 Cycle_programming__670_388-21.)
205 iTNC_530/340_49x-07 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p96 Cycle_programming__670_388-22.)
205 iTNC_530/60642x-03 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p98 Cycle_programming__670388-23_S)
205 iTNC_530/60642x-04 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p92 Cycle_programming__670388-24.t)
205 iTNC_530/60642x-04_SP8 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p94 Cycle_programming__670388-25.t)
205 iTNC_530/606_42x-01 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p94 Cycle_programming__670_388-21.)
205 iTNC_530/606_42x-02 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p96 Cycle_programming__670_388-22.)
205 TNC_620/34056x-04 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p82 Cycle_programming__679295-22.t)
205 TNC_620/340_56x-01 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p239 User_s_Manual_NC_software__636)
205 TNC_620/340_56x-02 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p80 Cycle_programming__679_295-20.)
205 TNC_620/340_56x-03 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p82 Cycle_programming__679_295-21.)
205 TNC_620/73498x-01 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p82 Cycle_programming__679_295-21.)
205 TNC_620/73498x-02 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p82 Cycle_programming__679295-22.t)
205 TNC_620/81760x-01 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p84 Cycle_programming__1096886-20.)
205 TNC_620/81760x-02 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p86 Cycle_programming__1096886-21.)
205 TNC_620/81760x-03 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p88 Cycle_programming__1096886-22.)
205 TNC_620/81760x-04 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p89 Cycle_programming__1096886-23.)
205 TNC_620/81760x-05 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p92 Cycle_programming__1096886-24.)
205 TNC_620/81760x-06 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p94 Cycle_programming__1096886-25.)
205 TNC_620/81760x-07 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p94 Cycle_programming__1096886-26.)
205 TNC_620/81760x-08 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p91 Machining_cycles__1303427-20.t)
205 TNC_620/81760x-16 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p94 Machining_cycles__1303427-21.t)
205 TNC_620/81760x-17 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p92 Machining_cycles__1303427-22.t)
205 TNC_620/81760x-18 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p93 Machining_cycles__1303427-23.t)
205 TNC_640/34059x-01 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p82 Cycle_programming__892905-20.t)
205 TNC_640/34059x-02 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 (p88 Cycle_programming__892905-21.t)
205 TNC_640/34059x-04 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p90 Cycle_programming__892905-22.t)
205 TNC_640/34059x-05 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p92 Cycle_programming__892905-23.t)
205 TNC_640/34059x-06 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p94 Cycle_programming__892905-24.t)
205 TNC_640/34059x-07 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p97 Cycle_programming__892905-25.t)
205 TNC_640/34059x-08 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p100 Cycle_programming__892905-26.t)
205 TNC_640/34059x-09 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p102 Cycle_programming__892905-27.t)
205 TNC_640/34059x-10 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p104 Cycle_programming__892905-28.t)
205 TNC_640/34059x-11 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 (p102 Machining_cycles__1303406-20.t)
205 TNC_640/34059x-16 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p105 Machining_cycles__1303406-21.t)
205 TNC_640/34059x-17 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p99 Machining_cycles__1303406-22.t)
205 TNC_640/34059x-18 200 201 206 202 203 204 212 205 258 259 257 256 211 379 253 208 395 373 (p103 Machining_cycles__1303406-23.t)
207 TNC7/81762x-17 200 201 239 203 204 (p143 Machining_cycles__1358775-21.t)
207 TNC7/81762x-18 200 201 239 203 204 (p243 Machining_cycles__1358775-22.t)
207 TNC7/81762x-19 200 201 239 203 204 (p233 Machining_cycles__1358775-23.t)
207 TNC7/81762x-20 200 201 239 203 204 (p251 Machining_cycles__1358775-24.t)
207 vTNC7/81762x-20_SP1 200 201 239 203 204 (p574 User_documentation__1505672_00)
207 TNC_320/34055x-06 200 201 239 203 204 (p92 Cycle_programming__679220-22.t)
207 TNC_320/340_551-01 200 201 239 203 204 (p200 User_s_Manual_NC_software__550)
207 TNC_320/340_551-02 200 201 239 203 204 (p206 User_s_Manual_NC_software__550)
207 TNC_320/340_55x-03 200 201 239 203 204 (p236 User_s_Manual_NC_software__550)
207 TNC_320/340_55x-04 200 201 239 203 204 (p97 Cycle_programming__679_220-20.)
207 TNC_320/340_55x-05 200 239 203 204 (p99 Cycle_programming__679_220-21.)
207 TNC_320/771851-18 200 201 239 203 204 (p137 Machining_cycles__1303429-20.t)
207 TNC_320/77185x-01 200 201 239 203 204 (p98 Cycle_programming__1096959-20.)
207 TNC_320/77185x-02 200 201 239 203 204 (p97 Cycle_programming__1096959-21.)
207 TNC_320/77185x-04 200 201 239 203 204 (p99 Cycle_programming__1096959-22.)
207 TNC_320/77185x-05 200 201 239 203 204 (p116 Cycle_programming__1096959-23.)
207 TNC_320/77185x-06 200 201 239 203 204 (p119 Cycle_programming__1096959-24.)
207 iTNC_530/34049x-08 200 201 239 203 204 (p115 Cycle_programming__670388-23_S)
207 iTNC_530/340_420 200 201 256 239 203 204 (p247 User_s_Manual_Date__369_475-21)
207 iTNC_530/340_422 216 217 244 245 246 247 241 200 203 204 (p323 User_s_Manual_Date__389_512-20)
207 iTNC_530/340_49x-01 200 201 256 239 203 204 (p271 User_s_Manual_NC_software__533)
207 iTNC_530/340_49x-02 200 201 256 239 203 204 257 336 (p297 User_s_Manual_NC_software__din)
207 iTNC_530/340_49x-03 200 256 201 239 203 204 257 336 (p316 User_s_Manual_NC_software__din)
207 iTNC_530/340_49x-04 200 201 256 239 203 204 257 336 403 (p332 User_s_Manual_NC_software__533)
207 iTNC_530/340_49x-05 200 201 239 203 204 (p107 Cycle_programming__670_388-20.)
207 iTNC_530/340_49x-06 200 201 239 203 204 (p111 Cycle_programming__670_388-21.)
207 iTNC_530/340_49x-07 200 201 239 203 204 (p113 Cycle_programming__670_388-22.)
207 iTNC_530/60642x-03 200 201 239 203 204 (p115 Cycle_programming__670388-23_S)
207 iTNC_530/60642x-04 200 201 239 203 204 (p109 Cycle_programming__670388-24.t)
207 iTNC_530/60642x-04_SP8 200 201 239 203 204 (p111 Cycle_programming__670388-25.t)
207 iTNC_530/606_42x-01 200 201 239 203 204 (p111 Cycle_programming__670_388-21.)
207 iTNC_530/606_42x-02 200 201 239 203 204 (p113 Cycle_programming__670_388-22.)
207 TNC_620/34056x-04 200 201 239 203 204 (p100 Cycle_programming__679295-22.t)
207 TNC_620/340_56x-01 200 201 239 203 204 (p245 User_s_Manual_NC_software__636)
207 TNC_620/340_56x-02 200 201 239 203 204 (p97 Cycle_programming__679_295-20.)
207 TNC_620/340_56x-03 200 239 203 204 (p99 Cycle_programming__679_295-21.)
207 TNC_620/73498x-01 200 239 203 204 (p99 Cycle_programming__679_295-21.)
207 TNC_620/73498x-02 200 201 239 203 204 (p100 Cycle_programming__679295-22.t)
207 TNC_620/81760x-01 200 201 239 203 204 (p102 Cycle_programming__1096886-20.)
207 TNC_620/81760x-02 200 201 239 203 204 (p104 Cycle_programming__1096886-21.)
207 TNC_620/81760x-03 200 201 239 203 204 (p106 Cycle_programming__1096886-22.)
207 TNC_620/81760x-04 200 201 239 203 204 (p108 Cycle_programming__1096886-23.)
207 TNC_620/81760x-05 200 201 239 203 204 (p120 Cycle_programming__1096886-24.)
207 TNC_620/81760x-06 200 201 239 203 204 (p123 Cycle_programming__1096886-25.)
207 TNC_620/81760x-07 200 201 239 203 204 (p124 Cycle_programming__1096886-26.)
207 TNC_620/81760x-08 200 201 239 203 204 (p118 Machining_cycles__1303427-20.t)
207 TNC_620/81760x-16 200 201 239 203 204 (p126 Machining_cycles__1303427-21.t)
207 TNC_620/81760x-17 200 201 239 203 204 (p124 Machining_cycles__1303427-22.t)
207 TNC_620/81760x-18 200 201 239 203 204 (p127 Machining_cycles__1303427-23.t)
207 TNC_640/34059x-01 200 239 203 204 (p99 Cycle_programming__892905-20.t)
207 TNC_640/34059x-02 200 201 239 203 204 (p106 Cycle_programming__892905-21.t)
207 TNC_640/34059x-04 200 201 239 203 204 (p108 Cycle_programming__892905-22.t)
207 TNC_640/34059x-05 200 201 239 203 204 (p110 Cycle_programming__892905-23.t)
207 TNC_640/34059x-06 200 201 239 203 204 (p112 Cycle_programming__892905-24.t)
207 TNC_640/34059x-07 200 201 239 203 204 (p116 Cycle_programming__892905-25.t)
207 TNC_640/34059x-08 200 201 239 203 204 (p128 Cycle_programming__892905-26.t)
207 TNC_640/34059x-09 200 201 239 203 204 (p131 Cycle_programming__892905-27.t)
207 TNC_640/34059x-10 200 201 239 203 204 (p134 Cycle_programming__892905-28.t)
207 TNC_640/34059x-11 200 201 239 203 204 (p130 Machining_cycles__1303406-20.t)
207 TNC_640/34059x-16 200 201 239 203 204 (p138 Machining_cycles__1303406-21.t)
207 TNC_640/34059x-17 200 201 239 203 204 (p132 Machining_cycles__1303406-22.t)
207 TNC_640/34059x-18 200 201 239 203 204 (p137 Machining_cycles__1303406-23.t)
209 TNC7/81762x-17 200 201 239 203 204 257 256 336 403 (p148 Machining_cycles__1358775-21.t)
209 TNC7/81762x-18 200 201 239 203 204 257 256 336 403 (p248 Machining_cycles__1358775-22.t)
209 TNC7/81762x-19 200 201 239 203 204 257 256 336 403 (p238 Machining_cycles__1358775-23.t)
209 TNC7/81762x-20 200 201 239 203 204 257 256 336 403 (p256 Machining_cycles__1358775-24.t)
209 vTNC7/81762x-20_SP1 200 201 239 203 204 257 256 336 403 (p579 User_documentation__1505672_00)
209 TNC_320/34055x-06 200 201 239 203 204 257 256 336 403 (p95 Cycle_programming__679220-22.t)
209 TNC_320/340_551-01 200 201 256 239 203 204 257 336 (p202 User_s_Manual_NC_software__550)
209 TNC_320/340_551-02 200 201 256 239 203 204 257 336 (p208 User_s_Manual_NC_software__550)
209 TNC_320/340_55x-03 256 200 201 239 203 204 257 336 403 (p239 User_s_Manual_NC_software__550)
209 TNC_320/340_55x-04 200 201 239 203 204 256 257 336 403 (p100 Cycle_programming__679_220-20.)
209 TNC_320/340_55x-05 200 239 203 204 256 257 336 403 (p102 Cycle_programming__679_220-21.)
209 TNC_320/771851-18 200 201 239 203 204 257 256 336 403 (p142 Machining_cycles__1303429-20.t)
209 TNC_320/77185x-01 200 201 239 203 204 257 256 336 403 (p101 Cycle_programming__1096959-20.)
209 TNC_320/77185x-02 200 201 239 203 204 257 256 336 403 (p100 Cycle_programming__1096959-21.)
209 TNC_320/77185x-04 200 201 239 203 204 257 256 336 403 (p102 Cycle_programming__1096959-22.)
209 TNC_320/77185x-05 200 201 239 203 204 257 256 336 403 (p120 Cycle_programming__1096959-23.)
209 TNC_320/77185x-06 200 201 239 203 204 257 256 336 403 (p124 Cycle_programming__1096959-24.)
209 iTNC_530/34049x-08 200 201 239 203 204 257 256 336 403 (p118 Cycle_programming__670388-23_S)
209 iTNC_530/340_420 216 217 244 245 246 247 241 200 203 204 (p294 User_s_Manual_Date__369_475-21)
209 iTNC_530/340_422 216 217 244 245 246 247 241 200 203 204 365 (p323 User_s_Manual_Date__389_512-20)
209 iTNC_530/340_49x-01 216 217 244 245 246 247 241 200 203 204 365 (p334 User_s_Manual_NC_software__533)
209 iTNC_530/340_49x-02 216 217 244 245 246 247 241 200 203 204 365 (p359 User_s_Manual_NC_software__din)
209 iTNC_530/340_49x-03 216 217 244 245 246 247 241 200 203 204 365 (p377 User_s_Manual_NC_software__din)
209 iTNC_530/340_49x-04 216 217 244 245 246 247 241 200 203 204 365 (p390 User_s_Manual_NC_software__533)
209 iTNC_530/340_49x-05 200 201 239 203 204 257 256 336 403 (p110 Cycle_programming__670_388-20.)
209 iTNC_530/340_49x-06 200 201 239 203 204 257 256 336 403 (p114 Cycle_programming__670_388-21.)
209 iTNC_530/340_49x-07 200 201 239 203 204 257 256 336 403 (p116 Cycle_programming__670_388-22.)
209 iTNC_530/60642x-03 200 201 239 203 204 257 256 336 403 (p118 Cycle_programming__670388-23_S)
209 iTNC_530/60642x-04 200 201 239 203 204 257 256 336 403 (p112 Cycle_programming__670388-24.t)
209 iTNC_530/60642x-04_SP8 200 201 239 203 204 257 256 336 403 (p114 Cycle_programming__670388-25.t)
209 iTNC_530/606_42x-01 200 201 239 203 204 257 256 336 403 (p114 Cycle_programming__670_388-21.)
209 iTNC_530/606_42x-02 200 201 239 203 204 257 256 336 403 (p116 Cycle_programming__670_388-22.)
209 TNC_620/34056x-04 200 201 239 203 204 257 256 336 403 (p103 Cycle_programming__679295-22.t)
209 TNC_620/340_56x-01 256 200 201 239 203 204 257 336 403 (p248 User_s_Manual_NC_software__636)
209 TNC_620/340_56x-02 200 201 239 203 204 256 257 336 403 (p100 Cycle_programming__679_295-20.)
209 TNC_620/340_56x-03 200 239 203 204 256 257 336 403 (p102 Cycle_programming__679_295-21.)
209 TNC_620/73498x-01 200 239 203 204 256 257 336 403 (p102 Cycle_programming__679_295-21.)
209 TNC_620/73498x-02 200 201 239 203 204 257 256 336 403 (p103 Cycle_programming__679295-22.t)
209 TNC_620/81760x-01 200 201 239 203 204 257 256 336 403 (p105 Cycle_programming__1096886-20.)
209 TNC_620/81760x-02 200 201 239 203 204 257 256 336 403 (p107 Cycle_programming__1096886-21.)
209 TNC_620/81760x-03 200 201 239 203 204 257 256 336 403 (p109 Cycle_programming__1096886-22.)
209 TNC_620/81760x-04 200 201 239 203 204 257 256 336 403 (p111 Cycle_programming__1096886-23.)
209 TNC_620/81760x-05 200 201 239 203 204 257 256 336 403 (p124 Cycle_programming__1096886-24.)
209 TNC_620/81760x-06 200 201 239 203 204 257 256 336 403 (p128 Cycle_programming__1096886-25.)
209 TNC_620/81760x-07 200 201 239 203 204 257 256 336 403 (p129 Cycle_programming__1096886-26.)
209 TNC_620/81760x-08 200 201 239 203 204 257 256 336 403 (p122 Machining_cycles__1303427-20.t)
209 TNC_620/81760x-16 200 201 239 203 204 257 256 336 403 (p131 Machining_cycles__1303427-21.t)
209 TNC_620/81760x-17 200 201 239 203 204 257 256 336 403 (p129 Machining_cycles__1303427-22.t)
209 TNC_620/81760x-18 200 201 239 203 204 257 256 336 403 (p132 Machining_cycles__1303427-23.t)
209 TNC_640/34059x-01 200 239 203 204 256 257 336 403 (p102 Cycle_programming__892905-20.t)
209 TNC_640/34059x-02 200 201 239 203 204 257 256 336 403 (p109 Cycle_programming__892905-21.t)
209 TNC_640/34059x-04 200 201 239 203 204 257 256 336 403 (p111 Cycle_programming__892905-22.t)
209 TNC_640/34059x-05 200 201 239 203 204 257 256 336 403 (p113 Cycle_programming__892905-23.t)
209 TNC_640/34059x-06 200 201 239 203 204 257 256 336 403 (p115 Cycle_programming__892905-24.t)
209 TNC_640/34059x-07 200 201 239 203 204 257 256 336 403 (p119 Cycle_programming__892905-25.t)
209 TNC_640/34059x-08 200 201 239 203 204 257 256 336 403 (p132 Cycle_programming__892905-26.t)
209 TNC_640/34059x-09 200 201 239 203 204 257 256 336 403 (p136 Cycle_programming__892905-27.t)
209 TNC_640/34059x-10 200 201 239 203 204 257 256 336 403 (p139 Cycle_programming__892905-28.t)
209 TNC_640/34059x-11 200 201 239 203 204 257 256 336 403 (p134 Machining_cycles__1303406-20.t)
209 TNC_640/34059x-16 200 201 239 203 204 257 256 336 403 (p143 Machining_cycles__1303406-21.t)
209 TNC_640/34059x-17 200 201 239 203 204 257 256 336 403 (p137 Machining_cycles__1303406-22.t)
209 TNC_640/34059x-18 200 201 239 203 204 257 256 336 403 (p142 Machining_cycles__1303406-23.t)
```

---

Section C (per control, every NC software version) is in [`differences-by-version.md`](differences-by-version.md).
