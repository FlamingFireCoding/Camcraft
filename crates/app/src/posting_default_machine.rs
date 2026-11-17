//! The default machine file (`assets/machines/3axis-vmc.mch`) read as the
//! post's machine for a mount with none, or refused by the file's name
//! (#2319). Provenance: our own machine-file format (docs/MACHINE-ONBOARDING.md).

use serde_json::Value;
use std::sync::OnceLock;

use super::files::{default_mch_text, DEFAULT_MCH_FILE};

/// The default machine: a plain 3-axis VMC with flood coolant. For the
/// readers that only time or show a machine (the sim's traverse clock):
/// a missing or malformed file reads as `Value::Null` here. The post
/// never reads this - it reads [`default_machine_checked`], which refuses
/// by the file's name (#2319).
pub fn default_machine() -> Value {
    default_machine_checked().unwrap_or(Value::Null)
}

/// The default machine, or the refusal that names its file (#2319). A
/// good file is parsed once (#2282). A refusal is not kept: the next post
/// reads the file again, so a repaired file posts without a restart.
pub fn default_machine_checked() -> Result<Value, String> {
    static M: OnceLock<Value> = OnceLock::new();
    if let Some(v) = M.get() {
        return Ok(v.clone());
    }
    match parse_default_machine(default_mch_text()) {
        Ok(v) => Ok(M.get_or_init(|| v).clone()),
        Err(why) => {
            tracing::error!("{why}");
            Err(why)
        }
    }
}

/// The default machine file's text read as the post's machine, or a
/// plain-English refusal that names the file (#2319). Refused: a file that
/// does not read, is not JSON, is not a JSON object, or is an object that
/// is not a machine - no `name` (the program header) or no
/// `kinematics.static` object (the limits and the coolant words). Such a
/// machine would post with no machine name and no coolant words.
pub fn parse_default_machine(text: Result<&str, String>) -> Result<Value, String> {
    let file = DEFAULT_MCH_FILE.0;
    let advice = "mount a machine on the setup, or reinstall the file";
    let text = text.map_err(|why| format!("the default machine file {file} cannot be read ({why}) - {advice}"))?;
    let v = serde_json::from_str::<Value>(text)
        .map_err(|e| format!("the default machine file {file} is malformed ({e}) - {advice}"))?;
    if !v.is_object() {
        return Err(format!("the default machine file {file} is malformed (not a machine object) - {advice}"));
    }
    let named = v.get("name").and_then(Value::as_str).is_some_and(|n| !n.trim().is_empty());
    if !named {
        return Err(format!("the default machine file {file} is malformed (it has no machine name) - {advice}"));
    }
    if !v.pointer("/kinematics/static").is_some_and(Value::is_object) {
        return Err(format!("the default machine file {file} is malformed (it has no kinematics.static section) - {advice}"));
    }
    Ok(v)
}
