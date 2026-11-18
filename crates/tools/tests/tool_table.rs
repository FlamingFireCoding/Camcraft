//! #2566: pure unit/integration tests for Tormach PathPilot (.csv) and
//! Hurco WinMax (.TCM) tool tables in `camcraft_tools`.
//!
//! Provenance: our own tests over the documented PathPilot CSV and
//! Hurco WinMax .TCM formats.

use camcraft_geometry::geometry::units::Unit;
use camcraft_tools::cutting::ToolMaterial;
use camcraft_tools::tool_table::{
    ToolTableExportRow, ToolTableFormat, detect_format, export_controller_table,
    parse_controller_table, parse_unit,
};
use camcraft_tools::tools::{Tool, ToolKind};
use std::path::Path;

const PATHPILOT_SAMPLE: &str = "\
Mill Tool Table,2026-11-18 19:09:05 - v1,This must be the first row - do not edit\r\n\
Tool number,Description,Z offset,Diameter,Z offset wear,Diameter wear,Max RPM\r\n\
1,endmill [1/8] 2FL R:0.000 loc:0.375 carbide dia:0.125 [37368],1.0000000,0.1250000,0.0000000,0.0000000,0.0\r\n\
2,,0.0000000,0.0000000,0.0000000,0.0000000,0.0\r\n\
8,Slitting Saw [1/8] 30FL R:0.000 uncoated carbide dia:3.000 [34993],5.0000000,3.0000000,0.0000000,0.0000000,0.0\r\n\
10,drill dia:0.500,2.0000000,0.5000000,0.0000000,0.0000000,0.0\r\n\
99,,3.0000000,0.0000000,0.0000000,0.0000000,-2.0\r\n";

#[test]
fn detects_formats_by_extension_and_content() {
    assert_eq!(detect_format(Path::new("tools.csv"), None), Some(ToolTableFormat::PathPilot));
    assert_eq!(detect_format(Path::new("atc_mtc.TCM"), None), Some(ToolTableFormat::Hurco));
    assert_eq!(detect_format(Path::new("shop.camtools"), None), Some(ToolTableFormat::CamTools));
    assert_eq!(
        detect_format(Path::new("unknown.txt"), Some(PATHPILOT_SAMPLE)),
        Some(ToolTableFormat::PathPilot)
    );
    assert_eq!(
        detect_format(Path::new("unknown.xml"), Some("<Hurco_Document Version=\"1.1\"/>")),
        Some(ToolTableFormat::Hurco)
    );
    assert_eq!(parse_unit("inch"), Ok(Unit::Inches));
    assert_eq!(parse_unit("mm"), Ok(Unit::Millimeters));
    assert!(parse_unit("furlongs").is_err());
}

#[test]
fn pathpilot_parses_sample_and_ignores_placeholder_z_offset_and_negative_probe_marker() {
    let entries = parse_controller_table(ToolTableFormat::PathPilot, PATHPILOT_SAMPLE, Unit::Inches)
        .expect("sample parses");
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].number, 1);
    assert_eq!(entries[0].name, "endmill [1/8] 2FL R:0.000 loc:0.375 carbide dia:0.125 [37368]");
    assert_eq!(entries[0].tool.kind, ToolKind::EndMill);
    assert!((entries[0].tool.diameter - 3.175).abs() < 1e-9);
    assert!((entries[0].tool.flute_len - 0.375 * 25.4).abs() < 1e-9);
    assert_eq!(entries[0].tool.flutes, Some(2));
    assert_eq!(entries[0].tool.tool_material, Some(ToolMaterial::Carbide));
    assert_eq!(entries[0].tool.catalog.part_number.as_deref(), Some("37368"));
    // Z offset 1.0 in must not become stick-out (flute_len stays 0.375 in).
    assert!((entries[0].tool.flute_len - 25.4).abs() > 1.0);

    assert_eq!(entries[1].number, 8);
    assert_eq!(entries[1].tool.flutes, Some(30));
    assert!(!entries[1].tool.center_cutting);
    assert_eq!(entries[1].tool.catalog.part_number.as_deref(), Some("34993"));

    assert_eq!(entries[2].number, 10);
    assert_eq!(entries[2].tool.kind, ToolKind::Drill);
    assert!((entries[2].tool.diameter - 12.7).abs() < 1e-9);

    // A configured probe with a non-empty description and -2.0 Max RPM parses cleanly.
    let probe_csv = "\
Mill Tool Table,2026-11-18 - v1,This must be the first row - do not edit\n\
Tool number,Description,Z offset,Diameter,Z offset wear,Diameter wear,Max RPM\n\
99,Passive Probe,3.0000000,0.1574803,0.0,0.0,-2.0\n";
    let probe = parse_controller_table(ToolTableFormat::PathPilot, probe_csv, Unit::Inches)
        .expect("probe parses");
    assert_eq!(probe[0].tool.kind, ToolKind::Probe);
    assert!(probe[0].tool.rpm > 0.0, "-2.0 marker must not become negative rpm");
}

#[test]
fn pathpilot_refuses_bad_headers_bad_rows_and_non_mill_export() {
    let missing_marker = "Tool number,Description,Z offset,Diameter\n1,EM,0,0.25\n";
    assert!(parse_controller_table(ToolTableFormat::PathPilot, missing_marker, Unit::Inches).is_err());

    let bad_dia = "\
Mill Tool Table,x - v1,This must be the first row - do not edit\n\
Tool number,Description,Z offset,Diameter,Z offset wear,Diameter wear,Max RPM\n\
4,Bad Cutter,0,zero,0,0,0\n";
    let err = parse_controller_table(ToolTableFormat::PathPilot, bad_dia, Unit::Inches).unwrap_err();
    assert!(err.contains("Bad Cutter") && err.contains("T4"), "{err}");

    let zero_dia_named = "\
Mill Tool Table,x - v1,This must be the first row - do not edit\n\
Tool number,Description,Z offset,Diameter,Z offset wear,Diameter wear,Max RPM\n\
4,Zero Cutter,0,0.0,0,0,0\n";
    let err = parse_controller_table(ToolTableFormat::PathPilot, zero_dia_named, Unit::Inches).unwrap_err();
    assert!(err.contains("Zero Cutter"), "{err}");

    let mut lathe = camcraft_tools::tools::catalog().into_iter().find(|(_, t)| t.kind == ToolKind::Turning).unwrap().1;
    lathe.diameter = 12.0;
    let err = export_controller_table(
        ToolTableFormat::PathPilot,
        &[ToolTableExportRow { number: 1, name: "Turn1", tool: &lathe, gauge_len: None }],
        Unit::Inches,
    )
    .unwrap_err();
    assert!(err.contains("Turn1"), "{err}");
}

#[test]
fn hurco_maps_supported_types_and_refuses_unsupported_by_name() {
    let xml = "\u{FEFF}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
<Hurco_Document Version=\"1.1\">\r\n\
  <Tools>\r\n\
    <Tool Id=\"a\"><Name>Bull</Name><Number>1</Number><Diameter>12</Diameter><TipRadius>1.5</TipRadius><AbsoluteGaugeLength>90</AbsoluteGaugeLength><ToolType>0</ToolType></Tool>\r\n\
    <Tool Id=\"b\"><Name>Chamfer</Name><Number>2</Number><Diameter>10</Diameter><Angle>90</Angle><ToolType>2</ToolType></Tool>\r\n\
    <Tool Id=\"c\"><Name>Engrave</Name><Number>3</Number><Diameter>6</Diameter><Angle>60</Angle><ToolType>7</ToolType></Tool>\r\n\
    <Tool Id=\"d\"><Name>Drill</Name><Number>4</Number><Diameter>8.5</Diameter><ToolType>11</ToolType></Tool>\r\n\
    <Tool Id=\"e\"><Name>Tap</Name><Number>5</Number><Diameter>6</Diameter><ThreadPitch>1.0</ThreadPitch><ToolType>12</ToolType></Tool>\r\n\
    <Tool Id=\"f\"><Name>Bore</Name><Number>6</Number><Diameter>16</Diameter><ToolType>13</ToolType></Tool>\r\n\
    <Tool Id=\"g\"><Name>Face</Name><Number>7</Number><Diameter>50</Diameter><ToolType>15</ToolType></Tool>\r\n\
    <Tool Id=\"h\"><Name>TMMC</Name><Number>8</Number><Diameter>8</Diameter><FluteLength>16</FluteLength><ThreadPitch>1.25</ThreadPitch><ToolType>22</ToolType></Tool>\r\n\
    <Tool Id=\"i\"><Name>TMSC</Name><Number>9</Number><Diameter>8</Diameter><ThreadPitch>1.25</ThreadPitch><ToolType>26</ToolType></Tool>\r\n\
  </Tools>\r\n\
</Hurco_Document>\r\n";

    let entries = parse_controller_table(ToolTableFormat::Hurco, xml, Unit::Millimeters)
        .expect("supported Hurco types");
    assert_eq!(entries.len(), 9);
    let kinds: Vec<ToolKind> = entries.iter().map(|e| e.tool.kind).collect();
    assert_eq!(
        kinds,
        vec![
            ToolKind::BullNose,
            ToolKind::Taper,
            ToolKind::Taper,
            ToolKind::Drill,
            ToolKind::Tap,
            ToolKind::Bore,
            ToolKind::EndMill,
            ToolKind::ThreadMill,
            ToolKind::ThreadMillSingle,
        ]
    );
    assert!(!entries[6].tool.center_cutting, "face mill is not center cutting");
    assert_eq!(entries[0].tool.holder_len, 90.0);

    // Every Hurco type without a CamCraft kind must refuse by name, never guess.
    for (code, label) in [(1, "Center Drill"), (3, "Corner Round"), (6, "Dove Tail"), (8, "Keyseat"), (9, "Ream"), (17, "Back Spotface")] {
        let bad = format!(
            "<Hurco_Document Version=\"1.1\"><Tools><Tool Id=\"x\"><Name>Tool{code}</Name><Number>{code}</Number><Diameter>10</Diameter><ToolType>{code}</ToolType></Tool></Tools></Hurco_Document>"
        );
        let err = parse_controller_table(ToolTableFormat::Hurco, &bad, Unit::Millimeters).unwrap_err();
        assert!(err.contains(&format!("Tool{code}")) && err.contains(label), "{code}: {err}");
    }
}

#[test]
fn hurco_export_roundtrips_and_escapes_xml_without_control_state() {
    let mut tool = Tool {
        kind: ToolKind::EndMill,
        diameter: 12.0,
        corner_r: 0.0,
        tip_angle: 0.0,
        pitch: 0.0,
        pitch_min: 0.0,
        thread_hand: Default::default(),
        nose_radius: 0.0,
        width: 0.0,
        flute_len: 25.0,
        shank_d: 12.0,
        shank_len: 20.0,
        holder_d: 40.0,
        holder_len: 105.5,
        material_group: "aluminum".into(),
        rpm: 5000.0,
        feed: 1200.0,
        tool_material: None,
        flutes: Some(4),
        center_cutting: true,
        orientation: None,
        catalog: Default::default(),
        cutting: vec![],
        outline: vec![],
        turning: None,
    };
    tool.catalog.vendor = Some("A & B <Carbide>".into());
    let text = export_controller_table(
        ToolTableFormat::Hurco,
        &[ToolTableExportRow { number: 12, name: "1/2\" EM & More", tool: &tool, gauge_len: Some(110.25) }],
        Unit::Millimeters,
    )
    .expect("export");
    assert!(text.contains("&amp;") && text.contains("&quot;"));
    assert!(text.contains("<AbsoluteGaugeLength>110.25</AbsoluteGaugeLength>"));
    assert!(!text.contains("UsedTime") && !text.contains("<Flags>"));

    let back = parse_controller_table(ToolTableFormat::Hurco, &text, Unit::Millimeters).expect("re-parse");
    assert_eq!(back[0].number, 12);
    assert_eq!(back[0].name, "1/2\" EM & More");
    assert_eq!(back[0].tool.holder_len, 110.25);
    assert_eq!(back[0].tool.catalog.vendor.as_deref(), Some("A & B <Carbide>"));
}
