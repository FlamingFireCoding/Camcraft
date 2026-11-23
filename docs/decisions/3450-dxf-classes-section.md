## #3450 - DXF import skips the CLASSES, THUMBNAILIMAGE and ACDSDATA sections

**Chosen.** The DXF reader skips three inert sections that the issue names: `CLASSES`, `THUMBNAILIMAGE` and `ACDSDATA`. It reads their records to the matching `ENDSEC` and imports the rest of the file. The content of these sections is opaque. Group pairs right after the section name are allowed, because `THUMBNAILIMAGE` and `ACDSDATA` hold data there. A section with no `ENDSEC` (end of file or a new `SECTION` first) gives the named error `<SECTION>: missing ENDSEC`. Every other unknown section name stays refused with `unsupported SECTION`.

**CLASSES strictness.** `CLASSES` is opaque too. A strict check for `CLASS` records only could refuse a legal file, and no test of ours proves that every legal `CLASSES` section holds only `CLASS` records. The test `classes_section_holds_only_class_records` is dropped.

**Why.** These sections hold class definitions, a preview image and application data. They hold no geometry. Refusing them rejected files that hold valid entities. Reading to `ENDSEC` cannot hang or panic.

**Dropped.** Skip every unknown section: this would hide a real unsupported file. Parse the section content: the importer has no use for it. A strict `CLASS` record check: see above.

**Cost to change.** Low. The change is one `matches!` arm and one function in `crates/geometry/src/dxf/reader.rs`.
