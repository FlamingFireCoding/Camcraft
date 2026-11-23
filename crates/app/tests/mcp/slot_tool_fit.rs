//! #3475: `set_tool` refuses a tool wider than a plain Slot, by name with
//! both sizes, and changes nothing; a tool that fits is set.
//! Provenance: our own operation and its MCP contract.
//!
//! Gate map: covers app::mcp (set_tool).

use super::Client;
use serde_json::json;

#[test]
fn set_tool_refuses_a_tool_wider_than_the_slot_without_mutation() {
    let mut c = Client::new();
    c.ok("set_stock", json!({ "box": { "dx": 60.0, "dy": 50.0, "dz": 10.0 } }));
    c.ok("add_tool", json!({ "catalog": "EM-0400-2FL" }));
    c.ok("add_tool", json!({ "catalog": "EM-0600-4FL" }));
    for (name, width) in [("S4", 4.0), ("S6", 6.0)] {
        c.ok("add_operation", json!({
            "kind": "Slot", "tool": "EM-0400-2FL", "name": name, "params": { "width": width, "stock_to_leave": 0.0 }
        }));
    }
    let before = c.ok("get_job", json!({}));
    let msg = c.refused("set_tool", json!({ "operation": "S4", "tool": "EM-0600-4FL" }));
    assert_eq!(msg, "set_tool: invalid parameters: the tool 'EM-0600-4FL' is 6 mm wide and the slot is 4 mm wide: choose a tool of 4 mm or less");
    assert_eq!(c.ok("get_job", json!({})), before, "the refused set_tool changes nothing");
    // A tool of the slot's own width fits.
    let set = c.ok("set_tool", json!({ "operation": "S6", "tool": "EM-0600-4FL" }));
    assert_eq!(set["tool"], json!("EM-0600-4FL"), "{set}");
}
