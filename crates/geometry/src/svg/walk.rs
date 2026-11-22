//! The SVG element tree walked into sketch primitives: groups and their transforms, `use`
//! references inside the file, layers from top-level groups, and the content that is skipped
//! with a note.
//! Provenance: the W3C SVG 1.1 structure chapter (g, defs, symbol, use, the `display`
//! property), its painting chapter (`visibility`), and Inkscape's public `inkscape:label`
//! layer convention; original implementation.

use super::emit::Sink;
use super::shapes::{self, Drawn};
use super::style::{StyleDecl, inline_declarations};
use super::transform::{self, Affine};
use super::xml::{Node, Tree};
use super::{SvgError, invalid};
use crate::dxf::ImportSummary;

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

pub(super) struct Walk<'a> {
    pub tree: &'a Tree,
    pub sink: Sink,
    pub summary: ImportSummary,
    pub max_entities: usize,
    pub max_cmds: usize,
    pub visits: usize,
}

fn parse_node_style(node: &Node) -> Vec<StyleDecl> {
    node.attr("style").map(inline_declarations).unwrap_or_default()
}

fn hidden(node: &Node, style: &[StyleDecl]) -> bool {
    let style_none = style.iter().any(|decl| {
        // CSS property names and keywords are case-insensitive. An importance
        // suffix must never make a hidden contour eligible for cutting.
        decl.property.eq_ignore_ascii_case("display")
            && decl.value.eq_ignore_ascii_case("none")
            && (!decl.has_bang || decl.importance.eq_ignore_ascii_case("important"))
    });
    style_none || node.attr("display").is_some_and(|d| d.trim().eq_ignore_ascii_case("none"))
}

fn has_inline_property(style: &[StyleDecl], name: &str) -> bool {
    style.iter().any(|decl| decl.property.eq_ignore_ascii_case(name))
}

/// Recognize unsupported effects in presentation attributes and inline declarations.
fn has_property(node: &Node, style: &[StyleDecl], name: &str) -> bool {
    node.attr(name).is_some() || has_inline_property(style, name)
}

fn check_transform_sources(node: &Node, style: &[StyleDecl]) -> Result<(), SvgError> {
    if node.attr("transform-origin").is_some() {
        return Err(invalid(
            "transform-origin presentation attribute is not supported",
        ));
    }
    if has_inline_property(style, "transform-origin") {
        return Err(invalid("inline CSS transform-origin is not supported"));
    }
    if has_inline_property(style, "transform") {
        return Err(invalid("inline CSS transform is not supported"));
    }
    Ok(())
}

impl<'a> Walk<'a> {
    fn note(&mut self, kind: &str) -> Result<(), SvgError> {
        self.count()?;
        *self.summary.skipped.entry(kind.to_owned()).or_default() += 1;
        Ok(())
    }

    fn skip_visibility_or_hidden(&mut self, node: &Node, style: &[StyleDecl]) -> Result<bool, SvgError> {
        // Visibility is inherited and permits child overrides. Refuse the whole
        // subtree until those rules can be evaluated without adding hidden cuts.
        if has_property(node, style, "visibility") {
            self.note("visibility")?;
            return Ok(true);
        }
        if hidden(node, style) {
            self.note("hidden")?;
            return Ok(true);
        }
        Ok(false)
    }

    fn note_unsupported_effects(&mut self, node: &Node, style: &[StyleDecl]) -> Result<(), SvgError> {
        for attr in ["clip-path", "mask"] {
            if has_property(node, style, attr) {
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
            let style = parse_node_style(root);
            if self.skip_visibility_or_hidden(root, &style)? {
                return Ok(());
            }
            check_transform_sources(root, &style)?;
            self.note_unsupported_effects(root, &style)?;
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
        let style = parse_node_style(node);
        if self.skip_visibility_or_hidden(node, &style)? {
            return Ok(());
        }
        check_transform_sources(node, &style)?;
        let map = match node.attr("transform") {
            Some(t) => parent.then(transform::parse(t)?),
            None => parent,
        };
        self.note_unsupported_effects(node, &style)?;
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
            // A symbol needs its own viewport, including the dimensions of this use.
            Some(sym) if sym.name == "symbol" => Err(invalid("<symbol> viewport references are not supported")),
            _ => self.element(target, map, uses + 1, depth + 1),
        }
    }
}
