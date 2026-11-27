//! #3898: what the component stack group derives from the stack that it
//! shows - the parts that "Add component" offers, the values that the
//! resolver gives for the stack (gauge length, cutting diameter, corner
//! radius), and for each row the parts that "Swap" offers: only the
//! replacements with which the whole stack still resolves. Frame cost:
//! the set is built when the tool, the document or the stack's links
//! change (`builds`); a still frame builds nothing (crates/ui/AGENTS.md
//! "Frame cost").
//! Provenance: our own cache over our own resolver call
//! (`camcraft_app::tool_assembly::derive`); no HSMWorks material involved.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use camcraft_app::schema::{AssemblyLink, DocumentComponent};
use camcraft_app::tool_assembly as ta;
use camcraft_app::tools::Tool;

/// The values a resolved stack gives its tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StackValues {
    /// From the cutting tip to the spindle gauge line (mm).
    pub gauge_len: f64,
    /// The cutting diameter (mm).
    pub diameter: f64,
    /// The corner radius (mm).
    pub corner_r: f64,
}

/// One replacement that "Swap" offers: its label and the whole new stack.
pub(crate) type SwapOffer = (String, Vec<AssemblyLink>);

/// What the stack group derives from one stack.
pub(crate) struct StackView {
    /// The parts that can go on the stack next (`stack::candidates`).
    pub offer: Vec<(String, AssemblyLink)>,
    /// The resolver's values, or its named refusal; `None` for no stack.
    pub values: Option<Result<StackValues, String>>,
    /// For each row, the replacements with which the stack resolves.
    pub swaps: Vec<Vec<SwapOffer>>,
}

#[derive(Clone, PartialEq)]
struct Key {
    tool: String,
    doc: u64,
    links: Vec<AssemblyLink>,
}

thread_local! {
    static KEPT: RefCell<Option<(Key, Rc<StackView>)>> = const { RefCell::new(None) };
    static BUILDS: Cell<u64> = const { Cell::new(0) };
}

/// The stack views built on this thread - a still frame adds none.
#[cfg(test)]
pub(crate) fn builds() -> u64 {
    BUILDS.with(Cell::get)
}

fn values(name: &str, tool: &Tool, links: &[AssemblyLink], table: &[DocumentComponent]) -> Result<StackValues, String> {
    ta::derive(name, tool, links, table).map(|d| StackValues { gauge_len: d.gauge_len, diameter: d.tool.diameter, corner_r: d.tool.corner_r })
}

/// The replacements for row `at`: each part that fits where the row's
/// part sits, put in its place with the row's turn and (when stated) its
/// stick-out kept, and kept only when the whole stack then resolves.
fn swaps_at(name: &str, tool: &Tool, links: &[AssemblyLink], at: usize, table: &[DocumentComponent]) -> Vec<SwapOffer> {
    let (Some(prefix), Some(old)) = (links.get(..at), links.get(at)) else {
        return Vec::new();
    };
    super::stack::candidates(prefix, table)
        .into_iter()
        .filter(|(_, c)| c.component != old.component)
        .filter_map(|(label, mut c)| {
            if let (Some(m), Some(was)) = (c.mount.as_mut(), old.mount.as_ref()) {
                m.z_rot_deg = was.z_rot_deg;
                m.projection_mm = was.projection_mm.or(m.projection_mm);
            }
            let mut next = links.to_vec();
            if let Some(slot) = next.get_mut(at) {
                *slot = c;
            }
            values(name, tool, &next, table).is_ok().then_some((label, next))
        })
        .collect()
}

/// The view of `links` on tool `name` (now `tool`) over `table`, for
/// document version `doc`.
pub(crate) fn view(name: &str, tool: Option<&Tool>, doc: u64, links: &[AssemblyLink], table: &[DocumentComponent]) -> Rc<StackView> {
    let key = Key { tool: name.to_owned(), doc, links: links.to_vec() };
    KEPT.with(|k| {
        let mut k = k.borrow_mut();
        if let Some((kk, v)) = k.as_ref()
            && *kk == key
        {
            return Rc::clone(v);
        }
        BUILDS.with(|b| b.set(b.get().saturating_add(1)));
        let offer = super::stack::candidates(links, table);
        let (values, swaps) = match tool {
            Some(t) if !links.is_empty() => (
                Some(values(name, t, links, table)),
                (0..links.len()).map(|i| swaps_at(name, t, links, i, table)).collect(),
            ),
            Some(_) => (None, Vec::new()),
            None => (Some(Err(format!("tool '{name}' is not in the document"))), Vec::new()),
        };
        let v = Rc::new(StackView { offer, values, swaps });
        *k = Some((key, Rc::clone(&v)));
        v
    })
}
