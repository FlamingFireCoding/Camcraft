//! Provenance: original machine reservation grammar tests from issue #2604.
//! Gate map (M4.4): covers camcraft_post::machine::tooling,
//! camcraft_post::machine::parse, camcraft_post::machine::def, camcraft_post::machine::reservations.

use camcraft_post::machine::{Machine, parse_mch};
use serde_json::{Value, json};

fn file(reserved: Value) -> String {
    json!({"tooling": {"reserved": reserved}}).to_string()
}

fn refuses(reserved: Value) {
    let error = parse_mch(&file(reserved)).expect_err("malformed reservation must refuse");
    assert!(error.contains("tooling.reserved"), "{error}");
}

#[test]
fn malformed_reserved_tool_numbers_refuse_by_key() {
    for number in [json!(0), json!(100_000), json!(-1), json!(1.5), json!("1"), json!(null), json!(true)] {
        refuses(json!([{"number": number, "use": "probe"}]));
    }
}

#[test]
fn malformed_reserved_tool_entries_refuse_by_key() {
    for entry in [
        json!({"use": "probe"}),
        json!({"number": 1}),
        json!({"number": 1, "use": "cutter"}),
        json!({"number": 1, "use": 1}),
        json!({"number": 1, "use": "probe", "typo": true}),
        json!({"name": "", "use": "probe"}),
        json!({"name": "   ", "use": "probe"}),
        json!({"name": 1, "use": "probe"}),
        json!(null), json!(1), json!("probe"),
    ] {
        refuses(json!([entry]));
    }
    for wrong_list in [json!({}), json!("probe"), json!(1), json!(true)] {
        refuses(wrong_list);
    }
}

#[test]
fn duplicate_reserved_tool_numbers_and_names_refuse_by_key() {
    refuses(json!([{"number": 1, "use": "probe"}, {"number": 1, "use": "probe"}]));
    refuses(json!([{"name": "PROBE", "use": "probe"}, {"name": "probe", "use": "probe"}]));
}

#[test]
fn reserved_tools_survive_stored_machine_round_trip() {
    let entries = json!([{"number": 1, "use": "probe"}, {"name": "PROBE", "use": "probe"}]);
    let machine = parse_mch(&file(entries.clone())).expect("valid reservations");
    let stored = serde_json::to_value(&machine).expect("stored machine");
    assert_eq!(stored.get("reserved_tools"), Some(&entries));
    let restored: Machine = serde_json::from_value(stored).expect("restore machine");
    assert_eq!(restored, machine);
    assert!(restored.is_sane());
}

#[test]
fn an_old_machine_has_no_reserved_tools() {
    let old = parse_mch("{}").expect("old machine");
    let stored = serde_json::to_value(&old).expect("stored machine");
    assert!(stored.get("reserved_tools").is_none_or(|v| v == &json!([])));
    assert!(old.is_sane());
}

#[test]
fn invalid_stored_reservations_make_machine_insane() {
    for entries in [
        json!([{"number": 0, "use": "probe"}]),
        json!([{"number": 100_000, "use": "probe"}]),
        json!([{"number": 1, "use": "probe"}, {"number": 1, "use": "probe"}]),
        json!([{"name": "PROBE", "use": "probe"}, {"name": "probe", "use": "probe"}]),
        json!([{"name": "   ", "use": "probe"}]),
    ] {
        let machine: Machine = serde_json::from_value(json!({"reserved_tools": entries}))
            .expect("stored shape can deserialize before semantic validation");
        assert!(!machine.is_sane(), "invalid stored reservations: {machine:?}");
    }
}

#[test]
fn shipped_tormach_mills_reserve_t99_for_the_probe() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/machines");
    for stem in [
        "tormach-770m", "tormach-770m-plus", "tormach-770mx", "tormach-770mx-ml",
        "tormach-770mx-microarc4", "tormach-1100m", "tormach-1100m-plus", "tormach-1100mx",
        "tormach-1500mx", "tormach-pcnc440", "tormach-pcnc770", "tormach-pcnc770-s3",
        "tormach-pcnc1100-s1", "tormach-pcnc1100-s2", "tormach-pcnc1100-s3",
        "tormach-24r", "tormach-xstech", "tormach-xstech-pro",
    ] {
        let text = std::fs::read_to_string(directory.join(format!("{stem}.mch"))).expect("shipped machine");
        let machine = parse_mch(&text).expect("valid shipped machine");
        let stored = serde_json::to_value(machine).expect("stored machine");
        let reserved = stored.get("reserved_tools").and_then(Value::as_array);
        assert!(reserved.is_some_and(|entries| entries.contains(&json!({"number": 99, "use": "probe"}))),
            "{stem} must reserve T99 for the probe");
    }
}
