## #3450 - DXF import skips the CLASSES section

**Chosen.** The DXF reader skips a `CLASSES` section. It reads `CLASS` records to the matching `ENDSEC` and imports the rest of the file. A `CLASSES` section with no `ENDSEC` gives the named error `CLASSES: missing ENDSEC`. A record that is not `CLASS` gives `CLASSES: expected CLASS or ENDSEC`. Every other unknown section name stays refused with `unsupported SECTION`.

**Why.** `CLASSES` is a standard section of AC1015 and later files. It lists application object classes and holds no geometry. Refusing it rejected files that hold valid entities. Reading to `ENDSEC` with a strict record check cannot hang or panic.

**Dropped.** Skip all unknown sections: this would hide a real unsupported file. Parse the class records: the importer has no use for them.

**Cost to change.** Low. The change is one match arm and one function in `crates/geometry/src/dxf/reader.rs`.
