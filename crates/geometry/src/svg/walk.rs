//! The SVG element tree walked into sketch primitives: groups and their transforms, `use`
//! references inside the file, layers from top-level groups, and the content that is skipped
//! with a note.
//! Provenance: the W3C SVG 1.1 structure chapter (g, defs, symbol, use, the `display`
//! property), its painting chapter (`visibility`), and Inkscape's public `inkscape:label`
//! layer convention; original implementation.

use super::emit::Sink;
use super::shapes::{self, Drawn};
use super::facts::Facts;
use super::transform::Affine;
use super::xml::{Node, Tree};
use super::{SvgError, invalid};
use crate::dxf::{ImportSummary, Limits};
use std::collections::HashMap;

/// The deepest chain of `use` references followed.
const MAX_USE_DEPTH: usize = 16;
/// Bound the active call stack across both groups and reference expansion.
const MAX_ACTIVE_DEPTH: usize = 256;

/// Never drawn and never noted: definitions and document furniture.
/// Stylesheets are counted once across the entire tree in `Walk::root`.
const SILENT: &[&str] = &[
    "defs", "symbol", "style", "metadata", "title", "desc", "namedview", "linearGradient",
    "radialGradient", "filter", "RDF", "perspective", "font", "font-face", "cursor",
];

/// The layer of elements outside any top-level group.
pub(super) const DEFAULT_LAYER: &str = "0";

/// The work budget in multiples of the byte budget. One pass over a file reads each attribute
/// one time; `use` references can read an element again, and this bounds the total.
const WORK_PER_BYTE: usize = 4;

pub(super) struct Walk<'a> {
    pub tree: &'a Tree,
    pub sink: Sink,
    pub summary: ImportSummary,
    max_entities: usize,
    max_cmds: usize,
    visits: usize,
    /// The attribute bytes that the visits have read, and their limit.
    work: usize,
    max_work: usize,
    /// The facts of each node, read at its first visit.
    facts: Vec<Option<Facts>>,
    /// The first element of each `id`: a search of all nodes for each `use` is quadratic.
    ids: HashMap<&'a str, usize>,
}

impl<'a> Walk<'a> {
    pub fn new(tree: &'a Tree, sink: Sink, limits: &Limits) -> Self {
        let mut ids = HashMap::new();
        for (index, node) in tree.nodes.iter().enumerate() {
            if let Some(id) = node.attr("id") {
                ids.entry(id).or_insert(index);
            }
        }
        Walk {
            tree,
            sink,
            summary: ImportSummary::default(),
            max_entities: limits.max_entities,
            max_cmds: limits.max_vertices,
            visits: 0,
            work: 0,
            max_work: limits.max_bytes.saturating_mul(WORK_PER_BYTE),
            facts: tree.nodes.iter().map(|_| None).collect(),
            ids,
        }
    }

    /// The facts of node `index`, read from its attributes at the first call only.
    fn facts(&mut self, index: usize, node: &Node) -> Facts {
        if let Some(Some(known)) = self.facts.get(index) {
            return known.clone();
        }
        let read = Facts::read(node, index == 0);
        if let Some(slot) = self.facts.get_mut(index) {
            *slot = Some(read.clone());
        }
        read
    }

    /// Charge one visit for the attribute bytes that it reads again.
    fn charge(&mut self, bytes: usize) -> Result<(), SvgError> {
        self.work = self.work.saturating_add(bytes);
        if self.work > self.max_work {
            return Err(invalid(format!(
                "references read more than {} bytes of element data: reference expansion exceeds the import budget",
                self.max_work
            )));
        }
        Ok(())
    }

    fn note(&mut self, kind: &str) -> Result<(), SvgError> {
        self.count()?;
        *self.summary.skipped.entry(kind.to_owned()).or_default() += 1;
        Ok(())
    }

    fn skip_visibility_or_hidden(&mut self, facts: &Facts) -> Result<bool, SvgError> {
        // Visibility is inherited and permits child overrides. Refuse the whole
        // subtree until those rules can be evaluated without adding hidden cuts.
        if facts.visibility {
            self.note("visibility")?;
            return Ok(true);
        }
        if facts.hidden {
            self.note("hidden")?;
            return Ok(true);
        }
        Ok(false)
    }

    fn note_unsupported_effects(&mut self, facts: &Facts) -> Result<(), SvgError> {
        for (attr, present) in [("clip-path", facts.clip_path), ("mask", facts.mask)] {
            if present {
                self.note(attr)?;
            }
        }
        Ok(())
    }

    fn count(&mut self) -> Result<(), SvgError> {
        if self.summary.entities >= self.max_entities {
            return Err(invalid(format!(
                "more than {} elements: the file is larger than the import budget",
                self.max_entities
            )));
        }
        self.summary.entities += 1;
        Ok(())
    }

    fn node(&self, index: usize) -> Option<&'a Node> {
        self.tree.nodes.get(index)
    }

    /// Walk the children of the root element under the root transform.
    pub fn root(&mut self, root_map: Affine) -> Result<(), SvgError> {
        if let Some(root) = self.node(0) {
            let facts = self.facts(0, root);
            if self.skip_visibility_or_hidden(&facts)? {
                return Ok(());
            }
            if let Some(refusal) = facts.refusal.clone() {
                return Err(refusal);
            }
            self.note_unsupported_effects(&facts)?;
        }
        // Stylesheets inside <defs> or other silent elements still govern the file,
        // so count every <style> element once across the whole tree.
        let style_count = self.tree.nodes.iter().filter(|n| n.name == "style").count();
        for _ in 0..style_count {
            self.note("style")?;
        }
        let children = self.node(0).map(|n| n.children.clone()).unwrap_or_default();
        for child in children {
            let layer = self.node(child).filter(|n| n.name == "g").map(|n| {
                n.attr("inkscape:label").or_else(|| n.attr("id")).unwrap_or(DEFAULT_LAYER).to_owned()
            });
            self.sink.layer = layer.unwrap_or_else(|| DEFAULT_LAYER.to_owned());
            self.element(child, root_map, 0, 0)?;
        }
        Ok(())
    }

    /// Draw element `index` whose parent maps user space through `parent`.
    fn element(&mut self, index: usize, parent: Affine, uses: usize, depth: usize) -> Result<(), SvgError> {
        if depth >= MAX_ACTIVE_DEPTH {
            return Err(invalid(format!(
                "active traversal depth exceeds {MAX_ACTIVE_DEPTH} elements across groups and references"
            )));
        }
        if self.visits >= self.max_entities {
            return Err(invalid(format!(
                "traversal visits more than {} elements: reference expansion exceeds the import budget",
                self.max_entities
            )));
        }
        self.visits += 1;
        let Some(node) = self.node(index) else { return Ok(()) };
        let name = node.name.clone();
        if SILENT.contains(&name.as_str()) {
            return Ok(());
        }
        let facts = self.facts(index, node);
        if self.skip_visibility_or_hidden(&facts)? {
            return Ok(());
        }
        if let Some(refusal) = facts.refusal.clone() {
            return Err(refusal);
        }
        let map = match facts.transform.clone()? {
            Some(t) => parent.then(t),
            None => parent,
        };
        self.charge(facts.reread)?;
        self.note_unsupported_effects(&facts)?;
        match name.as_str() {
            "g" | "a" => {
                for child in node.children.clone() {
                    self.element(child, map, uses, depth + 1)?;
                }
                Ok(())
            }
            "use" => self.reference(node, map, uses, depth),
            "path" | "rect" | "circle" | "ellipse" | "line" | "polyline" | "polygon" => {
                self.count()?;
                let saved = std::mem::replace(&mut self.sink.map, map);
                let drawn = shapes::draw(node, &mut self.sink, self.max_cmds);
                self.sink.map = saved;
                match drawn? {
                    Drawn::Shape => Ok(()),
                    Drawn::Empty => {
                        self.summary.entities -= 1;
                        self.note(&format!("{name} (empty)"))
                    }
                }
            }
            // Text, images, nested viewports, clipping and masking content, styles and the
            // like are not cut; each is counted so the caller can say what was left out.
            _ => self.note(&name),
        }
    }

    /// A `use` of an element in this file: drawn at its x, y; any other reference is noted.
    fn reference(&mut self, node: &'a Node, map: Affine, uses: usize, depth: usize) -> Result<(), SvgError> {
        let href = node.attr("href").or_else(|| node.attr("xlink:href")).unwrap_or("").trim();
        let target = href.strip_prefix('#').and_then(|id| self.ids.get(id).copied());
        let Some(target) = target else { return self.note("use") };
        if uses >= MAX_USE_DEPTH {
            return Err(invalid(format!("<use> references nest more than {MAX_USE_DEPTH} deep (a reference loop?)")));
        }
        let at = Affine::translate(
            super::number::user_length(node.attr("x").unwrap_or("0"), "<use> x")?,
            super::number::user_length(node.attr("y").unwrap_or("0"), "<use> y")?,
        );
        let map = map.then(at);
        match self.node(target) {
            // A symbol needs its own viewport, including the dimensions of this use.
            Some(sym) if sym.name == "symbol" => Err(invalid("<symbol> viewport references are not supported")),
            _ => self.element(target, map, uses + 1, depth + 1),
        }
    }
}
