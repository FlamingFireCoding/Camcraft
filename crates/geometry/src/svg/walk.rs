//! The SVG element tree walked into sketch primitives: groups and their transforms, `use`
//! references inside the file, layers from top-level groups, and the content that is skipped
//! with a note.
//! Provenance: the W3C SVG 1.1 structure chapter (g, defs, symbol, use, the `display`
//! property) and Inkscape's public `inkscape:label` layer convention; original implementation.

use super::emit::Sink;
use super::shapes::{self, Drawn};
use super::transform::{self, Affine};
use super::xml::{Node, Tree};
use super::{SvgError, invalid};
use crate::dxf::ImportSummary;

/// The deepest chain of `use` references followed.
const MAX_USE_DEPTH: usize = 16;

/// Never drawn and never noted: definitions and document furniture.
const SILENT: &[&str] = &[
    "defs", "symbol", "metadata", "title", "desc", "namedview", "linearGradient", "radialGradient",
    "filter", "RDF", "perspective", "font", "font-face", "cursor",
];

/// The layer of elements outside any top-level group.
pub(super) const DEFAULT_LAYER: &str = "0";

pub(super) struct Walk<'a> {
    pub tree: &'a Tree,
    pub sink: Sink,
    pub summary: ImportSummary,
    pub max_entities: usize,
    pub max_cmds: usize,
}

fn hidden(node: &Node) -> bool {
    let style_none = node.attr("style").is_some_and(|s| {
        s.split(';').any(|decl| {
            let mut kv = decl.splitn(2, ':').map(str::trim);
            matches!((kv.next(), kv.next()), (Some("display"), Some("none")))
        })
    });
    style_none || node.attr("display").is_some_and(|d| d.trim() == "none")
}

impl<'a> Walk<'a> {
    fn note(&mut self, kind: &str) -> Result<(), SvgError> {
        self.count()?;
        *self.summary.skipped.entry(kind.to_owned()).or_default() += 1;
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
        let children = self.node(0).map(|n| n.children.clone()).unwrap_or_default();
        for child in children {
            let layer = self.node(child).filter(|n| n.name == "g").map(|n| {
                n.attr("inkscape:label").or_else(|| n.attr("id")).unwrap_or(DEFAULT_LAYER).to_owned()
            });
            self.sink.layer = layer.unwrap_or_else(|| DEFAULT_LAYER.to_owned());
            self.element(child, root_map, 0)?;
        }
        Ok(())
    }

    /// Draw element `index` whose parent maps user space through `parent`.
    fn element(&mut self, index: usize, parent: Affine, uses: usize) -> Result<(), SvgError> {
        let Some(node) = self.node(index) else { return Ok(()) };
        let name = node.name.clone();
        if SILENT.contains(&name.as_str()) {
            return Ok(());
        }
        if hidden(node) {
            return self.note("hidden");
        }
        let map = match node.attr("transform") {
            Some(t) => parent.then(transform::parse(t)?),
            None => parent,
        };
        for attr in ["clip-path", "mask"] {
            if node.attr(attr).is_some() {
                self.note(attr)?;
            }
        }
        match name.as_str() {
            "g" | "a" | "switch" => {
                for child in node.children.clone() {
                    self.element(child, map, uses)?;
                }
                Ok(())
            }
            "use" => self.reference(node, map, uses),
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
    fn reference(&mut self, node: &'a Node, map: Affine, uses: usize) -> Result<(), SvgError> {
        let href = node.attr("href").or_else(|| node.attr("xlink:href")).unwrap_or("").trim();
        let tree = self.tree;
        let target = href.strip_prefix('#').and_then(|id| tree.nodes.iter().position(|n| n.attr("id") == Some(id)));
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
            // A symbol's content is drawn in place of the symbol.
            Some(sym) if sym.name == "symbol" => {
                for child in sym.children.clone() {
                    self.element(child, map, uses + 1)?;
                }
                Ok(())
            }
            _ => self.element(target, map, uses + 1),
        }
    }
}
