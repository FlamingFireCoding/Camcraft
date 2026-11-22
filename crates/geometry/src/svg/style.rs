//! Inline CSS `style` declaration scanning: comments, strings and character escapes.
//! Provenance: the W3C CSS 2.1 syntax rules (comments, character escapes, declaration
//! syntax, and `!important` rules) and CSS Syntax Level 3 string/comment boundaries;
//! original implementation.

pub(super) struct StyleDecl {
    pub property: String,
    pub value: String,
    pub has_bang: bool,
    pub importance: String,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum DeclPart {
    #[default]
    Property,
    Value,
    Importance,
}

#[derive(Default)]
struct DeclBuilder {
    property: String,
    value: String,
    importance: String,
    part: DeclPart,
    has_colon: bool,
    has_bang: bool,
    pending_ws: bool,
    depth: usize,
}

impl DeclBuilder {
    fn target(&mut self) -> &mut String {
        match self.part {
            DeclPart::Property => &mut self.property,
            DeclPart::Value => &mut self.value,
            DeclPart::Importance => &mut self.importance,
        }
    }

    fn push_char(&mut self, ch: char) {
        let pending = self.pending_ws;
        self.pending_ws = false;
        let out = self.target();
        if pending && !out.is_empty() {
            out.push(' ');
        }
        out.push(ch);
    }

    fn finish(&mut self, out: &mut Vec<StyleDecl>) {
        if self.has_colon {
            out.push(StyleDecl {
                property: std::mem::take(&mut self.property),
                value: std::mem::take(&mut self.value),
                has_bang: self.has_bang,
                importance: std::mem::take(&mut self.importance),
            });
        } else {
            self.property.clear();
            self.value.clear();
            self.importance.clear();
        }
        self.part = DeclPart::Property;
        self.has_colon = false;
        self.has_bang = false;
        self.pending_ws = false;
        self.depth = 0;
    }
}

/// Decode one CSS backslash escape starting immediately after `\`.
fn decode_css_escape(rest: &str) -> Option<(char, usize)> {
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if matches!(first, '\n' | '\r' | '\x0c') {
        return None;
    }
    if first.is_ascii_hexdigit() {
        let mut end = first.len_utf8();
        for (idx, ch) in chars.take(5) {
            if !ch.is_ascii_hexdigit() {
                break;
            }
            end = idx + ch.len_utf8();
        }
        let code = u32::from_str_radix(&rest[..end], 16).unwrap_or(0);
        let decoded = match code {
            0 => '\u{FFFD}',
            n => char::from_u32(n).unwrap_or('\u{FFFD}'),
        };
        // Per CSS2.1 §4.1.3, one trailing whitespace (or CRLF) ends a hex escape.
        let tail = &rest[end..];
        if tail.starts_with("\r\n") {
            end += 2;
        } else if tail
            .as_bytes()
            .first()
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c'))
        {
            end += 1;
        }
        return Some((decoded, end));
    }
    Some((first, first.len_utf8()))
}

fn consume_css_string(style: &str, mut i: usize, quote: char, out: &mut String) -> usize {
    out.push(quote);
    i += quote.len_utf8();
    while let Some(rest) = style.get(i..) {
        let Some(ch) = rest.chars().next() else { break };
        if matches!(ch, '\n' | '\r' | '\x0c') {
            break;
        }
        if ch == quote {
            out.push(quote);
            i += quote.len_utf8();
            break;
        }
        if ch == '\\' {
            i += 1;
            let after = style.get(i..).unwrap_or("");
            if let Some((decoded, n)) = decode_css_escape(after) {
                out.push(decoded);
                i += n;
            } else if after.starts_with("\r\n") {
                i += 2;
            } else if after.chars().next().is_some_and(|c| matches!(c, '\n' | '\r' | '\x0c')) {
                i += 1;
            } else {
                break;
            }
            continue;
        }
        out.push(ch);
        i += ch.len_utf8();
    }
    i
}

/// Split an inline `style` attribute into CSS declarations, stripping comments and decoding escapes.
pub(super) fn inline_declarations(style: &str) -> Vec<StyleDecl> {
    let mut decls = Vec::new();
    let mut b = DeclBuilder::default();
    let mut i = 0;
    while let Some(rest) = style.get(i..) {
        if let Some(after_start) = rest.strip_prefix("/*") {
            b.pending_ws = true;
            i = after_start.find("*/").map_or(style.len(), |end| i + 2 + end + 2);
            continue;
        }
        let Some(ch) = rest.chars().next() else { break };
        if matches!(ch, ' ' | '\t' | '\n' | '\r' | '\x0c') {
            b.pending_ws = true;
            i += 1;
            continue;
        }
        match ch {
            ';' if b.depth == 0 => {
                b.finish(&mut decls);
                i += 1;
            }
            ':' if b.part == DeclPart::Property && b.depth == 0 => {
                b.has_colon = true;
                b.part = DeclPart::Value;
                b.pending_ws = false;
                i += 1;
            }
            '!' if b.part == DeclPart::Value && b.depth == 0 => {
                b.has_bang = true;
                b.part = DeclPart::Importance;
                b.pending_ws = false;
                i += 1;
            }
            '"' | '\'' => {
                if b.pending_ws && !b.target().is_empty() {
                    b.target().push(' ');
                }
                b.pending_ws = false;
                i = consume_css_string(style, i, ch, b.target());
            }
            '\\' => {
                if let Some((decoded, n)) = decode_css_escape(&rest[1..]) {
                    b.push_char(decoded);
                    i += 1 + n;
                } else {
                    b.push_char('\\');
                    i += 1;
                }
            }
            '(' | '[' | '{' => {
                b.depth += 1;
                b.push_char(ch);
                i += 1;
            }
            ')' | ']' | '}' => {
                b.depth = b.depth.saturating_sub(1);
                b.push_char(ch);
                i += 1;
            }
            _ => {
                b.push_char(ch);
                i += ch.len_utf8();
            }
        }
    }
    b.finish(&mut decls);
    decls
}
