{
  "general": { "model": "Sodick AQ600L", "vendor": "Sodick", "kind": "wire_edm",
    "description": "Sodick AQ600L wire EDM with the LN2W control (#4045). Travels: X/Y 600 x 400 mm and U/V 120 x 120 mm, taper +-20 deg. These figures are NOT YET CHECKED against Sodick's published AQ600L specification (docs/decisions/4045-sodick-ln2w-dialect.md) - CHECK THEM against your machine's specification sheet; each travel is centred on zero. Stated by an owner of this machine (2026-11-25): automatic wire threading (T91 / T90), high flushing (T84) and submerged machining (T94) at each start. The taper rules (edm.taper_angle: a positive angle is G51, the upper guide left of travel; mitre corners) follow the Sodick LN1W / LN2W operation manual (2012) where it states them; the corner rule is not confirmed on a control. EXAMPLE STARTER SET: the conditions C001 / C002 / C903, the guide heights (used only to check the U/V travel of a taper) and edm.table_to_part_bottom (0: the work sits on the worktable) are examples so that a first Wire Contour generates and posts - they are not Sodick figures; replace them with the rows of your control's condition table, your machine's guide heights and your fixture's height before you cut. Wire diameters and codes have no public source and are left out" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "x", "name": "X", "type": "linear", "travel": { "min": -300.0, "max": 300.0 }, "parts": [
          { "id": "y", "name": "Y", "type": "linear", "travel": { "min": -200.0, "max": 200.0 } }
        ] },
        { "id": "u", "name": "U", "type": "linear", "travel": { "min": -60.0, "max": 60.0 }, "parts": [
          { "id": "v", "name": "V", "type": "linear", "travel": { "min": -60.0, "max": 60.0 } }
        ] },
        { "id": "head", "name": "guides", "type": "head" }
      ] }
    ] } },
  "controller": { "post": "Sodick Wire EDM (LN2W)" },
  "edm": {
    "lower_guide_z": 0.0, "upper_guide_z": 130.0,
    "max_taper_deg": 20.0,
    "taper_angle": { "positive": "top_left", "corner": "mitre" },
    "table_to_part_bottom": 0.0,
    "awt": true,
    "tank": "submerged", "flushing": "high", "skim_flushing": "low",
    "conditions": [
      { "id": "C001", "name": "First cut (example)", "role": "rough", "gap": 0.04, "wire_diameter": 0.25,
        "notes": "EXAMPLE - not a Sodick figure; replace with the row of your control's condition table" },
      { "id": "C002", "name": "Second cut (example)", "role": "skim", "gap": 0.02, "wire_diameter": 0.25,
        "notes": "EXAMPLE - not a Sodick figure; replace with the row of your control's condition table" },
      { "id": "C903", "name": "Third cut (example)", "role": "finish", "gap": 0.01, "wire_diameter": 0.25,
        "notes": "EXAMPLE - not a Sodick figure; replace with the row of your control's condition table" }
    ]
  }
}
