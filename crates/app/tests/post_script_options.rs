//! #258: document/setup per-post `PostScriptOptions` persistence, undo/redo,
//! post-switch isolation, verification disagreement check, CLI `--post-option`,
//! and MCP `get_post_options` / `set_post_options` integration.
//!
//! Provenance: our own fixtures over our own commands; no vendor material.

use std::collections::BTreeMap;
use std::path::PathBuf;

use flecs_ecs::prelude::World;
use serde_json::{json, Value};

use camcraft_app::commands::{
    AddTool, CreateOperation, CreateSetup, GenerateOperation, History, SelectPost, SetPostOptions,
    SetPostScriptOptions, SetSetupPostScriptOptions, SetStock,
};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::post::options::OptionValue;
use camcraft_app::schema::{BetweenSetups, PostOptions, PostRef, PostScriptOptions};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use camcraft_app::{cli, mcp::Server, posting, project, setup_post_options};

const TOOL: &str = "EM-0400-2FL";

fn face() -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    }
}

fn add_setup_with_face(world: &World, h: &mut History, op_name: &str) {
    h.execute(world, CreateSetup::new(world, None, None).expect("setup"));
    let stock = SetStock::new(world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Default::default());
    h.execute(world, stock.expect("stock"));
    let op = CreateOperation::new(world, op_name, TOOL, SrcRef::Stock, face());
    h.execute(world, op.expect("op"));
    h.execute(world, GenerateOperation::new(world, op_name).expect("gen"));
}

fn single_face_doc() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let stock = SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Default::default());
    h.execute(&world, stock.expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("tool");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    let op = CreateOperation::new(&world, "Face1", TOOL, SrcRef::Stock, face());
    h.execute(&world, op.expect("op"));
    h.execute(&world, GenerateOperation::new(&world, "Face1").expect("gen"));
    (world, h)
}

fn tmp(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!("camcraft-258-{tag}-{}.{ext}", std::process::id()))
}

#[test]
fn document_and_setup_script_options_undo_redo_and_keep_per_post_memory() {
    let (world, mut h) = single_face_doc();
    let fanuc = PostRef::Builtin;
    let dnm = PostRef::Named("Doosan DNM".into());

    // Invalid option key and bad value refuse cleanly without touching history.
    let before_done = h.done_len();
    let mut bad = BTreeMap::new();
    bad.insert("noSuchOption".into(), OptionValue::Bool(true));
    let err = SetPostScriptOptions::for_post(&world, &fanuc, bad).expect_err("unknown refused");
    assert!(err.to_string().contains("noSuchOption"), "{err}");
    assert_eq!(h.done_len(), before_done);

    // Set Fanuc option.
    let mut fanuc_ov = BTreeMap::new();
    fanuc_ov.insert("showSequenceNumbers".into(), OptionValue::Str("every-block".into()));
    fanuc_ov.insert("sequenceNumberStart".into(), OptionValue::Int(50));
    h.execute(
        &world,
        SetPostScriptOptions::for_post(&world, &fanuc, fanuc_ov.clone()).expect("fanuc set"),
    );

    // Post writes N50 sequence numbers!
    let nc = tmp("seq", "nc");
    let posted = posting::post_setup(&world, &nc).expect("post succeeds");
    assert!(posted.program.contains("N50 "), "expected N50 sequence lines: {}", posted.program);

    // Switch to Doosan DNM, set Doosan smoothingLevel without disturbing Fanuc's saved choices.
    h.execute(&world, SelectPost::new(&world, dnm.clone()).expect("select dnm"));
    let mut dnm_ov = BTreeMap::new();
    dnm_ov.insert("smoothingLevel".into(), OptionValue::Int(8));
    h.execute(
        &world,
        SetPostScriptOptions::for_post(&world, &dnm, dnm_ov.clone()).expect("dnm set"),
    );
    h.execute(&world, SelectPost::new(&world, fanuc.clone()).expect("back to fanuc"));

    let key_fanuc = setup_post_options::post_key(&fanuc);
    let key_dnm = setup_post_options::post_key(&dnm);
    let doc_opts = setup_post_options::script_document(&world);
    assert_eq!(doc_opts.for_post(&key_fanuc).get("showSequenceNumbers"), fanuc_ov.get("showSequenceNumbers"));
    assert_eq!(doc_opts.for_post(&key_dnm).get("smoothingLevel"), dnm_ov.get("smoothingLevel"));

    // Undo and redo round-trip cleanly.
    h.undo(&world);
    h.undo(&world);
    let after_undo = setup_post_options::script_document(&world);
    assert!(!after_undo.for_post(&key_dnm).contains_key("smoothingLevel"));
    h.redo(&world);
    let after_redo = setup_post_options::script_document(&world);
    assert_eq!(after_redo.for_post(&key_dnm).get("smoothingLevel"), dnm_ov.get("smoothingLevel"));

    let _ = std::fs::remove_file(&nc);
}

#[test]
fn project_round_trips_script_options_and_flags_multi_setup_disagreement() {
    let world = new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("tool");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    add_setup_with_face(&world, &mut h, "Face1");
    add_setup_with_face(&world, &mut h, "Face2");

    let fanuc = PostRef::Builtin;
    let mut doc_ov = BTreeMap::new();
    doc_ov.insert("writeToolList".into(), OptionValue::Bool(true));
    h.execute(
        &world,
        SetPostScriptOptions::for_post(&world, &fanuc, doc_ov).expect("doc set"),
    );

    let mut setup2_ov = BTreeMap::new();
    setup2_ov.insert("writeToolList".into(), OptionValue::Bool(false));
    setup2_ov.insert("showSequenceNumbers".into(), OptionValue::Str("every-block".into()));
    let mut next_setup2 = PostScriptOptions::default();
    next_setup2.set_for_post(&setup_post_options::post_key(&fanuc), setup2_ov);
    h.execute(
        &world,
        SetSetupPostScriptOptions::for_setup(&world, "Setup2", Some(next_setup2)).expect("setup2 set"),
    );

    // Setup1 still follows document (`None` own), Setup2 owns its overrides.
    assert!(setup_post_options::script_own(&world, Some("Setup1")).is_none());
    assert!(setup_post_options::script_own(&world, Some("Setup2")).is_some());

    // Combined single-file post must refuse by setup name because Setup1 and Setup2 disagree on Fanuc's script options.
    let problems = setup_post_options::program_problems(&world);
    assert!(!problems.is_empty(), "disagreeing script options must fail combined program check");

    // Project save/load preserves both document and Setup2 overrides.
    let text = project::save_text(&world).expect("save");
    let reloaded = new_world();
    project::open(&reloaded, &text).expect("open");
    assert_eq!(
        setup_post_options::script_document(&reloaded),
        setup_post_options::script_document(&world)
    );
    assert_eq!(
        setup_post_options::script_own(&reloaded, Some("Setup2")),
        setup_post_options::script_own(&world, Some("Setup2"))
    );

    // Switching to FilePerSetup resolves the multi-setup conflict and posts separate clean files.
    let cur = setup_post_options::document(&world);
    h.execute(
        &world,
        SetPostOptions::new(&world, PostOptions { between_setups: BetweenSetups::FilePerSetup, ..cur })
            .expect("file per setup"),
    );
    assert!(setup_post_options::program_problems(&world).is_empty());
}

#[test]
fn mcp_and_cli_expose_and_validate_declared_post_options() {
    let mut server = Server::new();
    let req = |s: &mut Server, id: i64, method: &str, params: Value| {
        s.handle(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
            .expect("reply")
    };
    req(&mut server, 1, "initialize", json!({ "protocolVersion": "2025-06-18" }));

    let got = req(&mut server, 2, "tools/call", json!({ "name": "get_post_options", "arguments": {} }));
    let sc = &got["result"]["structuredContent"];
    assert!(sc["script_schema"].is_array(), "missing script_schema: {sc}");
    assert!(sc["script_options"].is_object(), "missing script_options: {sc}");

    let set_ok = req(
        &mut server,
        3,
        "tools/call",
        json!({
            "name": "set_post_options",
            "arguments": { "script_options": { "showSequenceNumbers": "tool-change", "writeMachine": true } }
        }),
    );
    assert_ne!(set_ok["result"]["isError"], json!(true), "{set_ok}");
    assert_eq!(
        set_ok["result"]["structuredContent"]["script_options"]["showSequenceNumbers"],
        json!("tool-change")
    );
    assert_eq!(
        set_ok["result"]["structuredContent"]["script_options"]["writeMachine"],
        json!(true)
    );

    let set_bad = req(
        &mut server,
        4,
        "tools/call",
        json!({
            "name": "set_post_options",
            "arguments": { "script_options": { "noSuchOpt": 1 } }
        }),
    );
    assert_eq!(set_bad["result"]["isError"], json!(true));
    assert!(set_bad["result"]["content"][0]["text"].as_str().unwrap_or("").contains("noSuchOpt"));

    // CLI `process --post-option` check.
    let (world, _) = single_face_doc();
    let proj = tmp("cli-opt", "camproj");
    let out = tmp("cli-opt", "nc");
    project::save(&world, &proj).expect("save proj");

    let args: Vec<String> = [
        "process",
        &proj.display().to_string(),
        "--post-option",
        "showSequenceNumbers=every-block",
        "-o",
        &out.display().to_string(),
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let code = cli::run(&args);
    assert_eq!(code, 0, "cli process with --post-option must exit 0");
    let posted = std::fs::read_to_string(&out).expect("nc output");
    assert!(posted.contains("N10 "), "expected N10 from CLI --post-option: {posted}");

    let bad_args: Vec<String> = [
        "process",
        &proj.display().to_string(),
        "--post-option",
        "nonexistentOpt=true",
        "-o",
        &out.display().to_string(),
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let bad_code = cli::run(&bad_args);
    assert_ne!(bad_code, 0, "unknown --post-option must exit non-zero");

    let _ = std::fs::remove_file(&proj);
    let _ = std::fs::remove_file(&out);
}
