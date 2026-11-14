//! Minimal SVG element tree, serialised like Python's ElementTree with `indent(space="  ")`.
use crate::error::Result;
use std::path::Path;

pub const SVG_NS: &str = "http://www.w3.org/2000/svg";

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Element {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub text: Option<String>,
    pub children: Vec<Element>,
}

impl Element {
    pub fn new(tag: &str) -> Self {
        Element { tag: tag.to_string(), ..Default::default() }
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    pub fn set(&mut self, name: &str, value: impl Into<String>) {
        let value = value.into();
        match self.attrs.iter_mut().find(|(k, _)| k == name) {
            Some(slot) => slot.1 = value,
            None => self.attrs.push((name.to_string(), value)),
        }
    }

    pub fn id(&self) -> Option<&str> {
        self.attr("id")
    }

    /// This element and all descendants, document order.
    pub fn iter(&self) -> Vec<&Element> {
        let mut out = vec![self];
        for child in &self.children {
            out.extend(child.iter());
        }
        out
    }

    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut Element> {
        if self.id() == Some(id) {
            return Some(self);
        }
        self.children.iter_mut().find_map(|c| c.find_by_id_mut(id))
    }
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_text(s).replace('"', "&quot;").replace('\r', "&#13;").replace('\n', "&#10;").replace('\t', "&#09;")
}

fn write(e: &Element, depth: usize, root_ns: Option<&str>, out: &mut String) {
    let pad = "  ".repeat(depth);
    out.push_str(&pad);
    out.push('<');
    out.push_str(&e.tag);
    if let Some(ns) = root_ns {
        out.push_str(&format!(" xmlns=\"{ns}\""));
    }
    for (k, v) in &e.attrs {
        out.push_str(&format!(" {k}=\"{}\"", escape_attr(v)));
    }
    match (&e.text, e.children.is_empty()) {
        (Some(t), true) if !t.is_empty() => out.push_str(&format!(">{}</{}>", escape_text(t), e.tag)),
        (_, true) => out.push_str(" />"),
        (_, false) => {
            out.push_str(">\n");
            for c in &e.children {
                write(c, depth + 1, None, out);
                out.push('\n');
            }
            out.push_str(&pad);
            out.push_str(&format!("</{}>", e.tag));
        }
    }
}

pub fn to_string(root: &Element) -> String {
    let mut out = String::from("<?xml version='1.0' encoding='utf-8'?>\n");
    write(root, 0, Some(SVG_NS), &mut out);
    out
}

/// Serialise a document without a default namespace (draw.io `mxfile`).
pub fn to_string_plain(root: &Element) -> String {
    let mut out = String::from("<?xml version='1.0' encoding='utf-8'?>\n");
    write(root, 0, None, &mut out);
    out
}

pub fn write_file(root: &Element, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, to_string(root))?;
    Ok(())
}
