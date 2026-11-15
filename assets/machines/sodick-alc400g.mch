{
  "general": { "model": "Sodick ALC400G", "vendor": "Sodick", "kind": "wire_edm" },
  "kinematics": { "default": {
    "units": { "length": "mm", "angle": "degrees" },
    "parts": [
      { "id": "base", "name": "table", "type": "table", "parts": [
        { "id": "x", "name": "X", "type": "linear", "travel": { "min": -200.0, "max": 200.0 }, "parts": [
          { "id": "y", "name": "Y", "type": "linear", "travel": { "min": -150.0, "max": 150.0 } }
        ] },
        { "id": "u", "name": "U", "type": "linear", "travel": { "min": -75.0, "max": 75.0 }, "parts": [
          { "id": "v", "name": "V", "type": "linear", "travel": { "min": -75.0, "max": 75.0 } }
        ] },
        { "id": "head", "name": "guides", "type": "head" }
      ] }
    ] } },
  "edm": {
    "max_taper_deg": 25.0
  }
}
