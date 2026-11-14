//! Diagram checks: v2 model discovery, model/SVG parity, estimated text bounds and line crossings.
use crate::error::{err, Error, Result};
use crate::locale::{self, Config, LocaleDef};
use crate::model::{resolve, validate};
use crate::util::*;
use roxmltree::Document;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Sorted files below `examples` and `architectures` whose extension (or name suffix) matches.
pub fn files_under(root: &Path, accept: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for base in ["examples", "architectures"] {
        walk_files(&root.join(base), &mut out);
    }
    out.retain(|p| accept(p));
    out.sort();
    out
}

pub fn is_model_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    name == "model.json" || name.ends_with(".model.json")
}

pub fn model_files(root: &Path) -> Result<Vec<(PathBuf, Value)>> {
    let mut found = Vec::new();
    for path in files_under(root, is_model_file) {
        let model = read_json(&path)?;
        if model.get("schema_version").and_then(Value::as_i64) == Some(2) {
            found.push((path, model));
        }
    }
    Ok(found)
}

pub fn svg_path(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    if name.ends_with(".model.json") {
        return path.with_file_name(name.replace(".model.json", ".svg"));
    }
    let manaslu = path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) == Some("manaslu-v2");
    path.with_file_name(if manaslu { "overview.svg" } else { "diagram.svg" })
}

fn attr<'a>(e: roxmltree::Node<'a, '_>, name: &str) -> Option<&'a str> {
    e.attribute(name).filter(|v| !v.is_empty())
}

/// Output language the SVG must declare: tag and font stack.
pub struct Expect<'a> {
    pub lang: &'a str,
    pub font_family: &'a str,
}

const JAPANESE_FONTS: [&str; 4] = ["Noto Sans JP", "Hiragino", "Yu Gothic", "Meiryo"];
const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

pub fn audit_svg(model: &Value, path: &Path) -> Result<()> {
    audit_svg_with(model, path, None)
}

fn ascii_words(text: &str) -> Vec<&str> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'))).filter(|w| !w.is_empty()).collect()
}

/// Displayed strings with the ID prefix of their `<text>` lines.
fn displayed(model: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for r in items(model, "regions") {
        out.push((format!("{}_label", str_of(r, "id").unwrap_or_default()), str_of(r, "label").unwrap_or_default().to_string()));
    }
    for n in items(model, "nodes") {
        let id = str_of(n, "id").unwrap_or_default();
        out.push((format!("{id}_name"), str_of(n, "display_name").or(str_of(n, "name")).unwrap_or_default().to_string()));
        out.push((format!("{id}_role"), str_of(n, "display_role").or(str_of(n, "role")).unwrap_or_default().to_string()));
    }
    for c in items(model, "connections") {
        out.push((format!("{}_label", str_of(c, "id").unwrap_or_default()), crate::model::connection_label(c)));
    }
    out
}

pub fn audit_svg_with(model: &Value, path: &Path, expect: Option<&Expect>) -> Result<()> {
    let text = std::fs::read_to_string(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
    let doc = Document::parse_with_options(&text, roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() })?;
    let elements: Vec<_> = doc.descendants().filter(|n| n.is_element()).collect();
    let mut errors: Vec<String> = Vec::new();
    let ids: Vec<&str> = elements.iter().filter_map(|e| attr(*e, "id")).collect();
    if ids.len() != ids.iter().collect::<HashSet<_>>().len() {
        errors.push("duplicate SVG ID".into());
    }
    let nodes: HashMap<&str, &Value> = items(model, "nodes").iter().filter_map(|n| str_of(n, "id").map(|i| (i, n))).collect();
    let regions: HashMap<&str, &Value> = items(model, "regions").iter().filter_map(|r| str_of(r, "id").map(|i| (i, r))).collect();
    let actual_nodes: HashSet<&str> = elements.iter().filter(|e| e.attribute("data-kind") == Some("node")).filter_map(|e| e.attribute("id")).collect();
    if actual_nodes != nodes.keys().copied().collect::<HashSet<_>>() {
        errors.push("SVG nodes differ from model".into());
    }
    type Edge<'a> = (Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>);
    let edge_elements: Vec<_> = elements.iter().filter(|e| attr(**e, "data-source").is_some()).collect();
    let actual_edges: HashSet<Edge> = edge_elements
        .iter()
        .map(|e| (e.attribute("id"), e.attribute("data-source"), e.attribute("data-target"), e.attribute("data-kind")))
        .collect();
    let expected_edges: HashSet<Edge> = items(model, "connections")
        .iter()
        .map(|c| (str_of(c, "id"), str_of(c, "source"), str_of(c, "target"), str_of(c, "kind")))
        .collect();
    if actual_edges != expected_edges {
        errors.push("SVG connections differ from model".into());
    }
    let edge_by_id: HashMap<&str, roxmltree::Node> = edge_elements.iter().filter_map(|e| e.attribute("id").map(|i| (i, **e))).collect();
    for c in items(model, "connections") {
        let Some(element) = str_of(c, "id").and_then(|i| edge_by_id.get(i)) else { continue };
        for key in ["interaction", "transfer_mode"] {
            if let Some(expected) = c.get(key) {
                if Some(py_str(expected).as_str()) != element.attribute(format!("data-{}", key.replace('_', "-")).as_str()) {
                    errors.push(format!("SVG connection semantics differ from model: {}", str_of(c, "id").unwrap_or_default()));
                }
            }
        }
    }
    let region_elements: HashMap<&str, roxmltree::Node> = elements
        .iter()
        .filter(|e| attr(**e, "data-region-kind").is_some())
        .filter_map(|e| e.attribute("id").map(|i| (i, *e)))
        .collect();
    for (id, region) in &regions {
        if str_of(region, "kind") == Some("network") {
            if let Some(element) = region_elements.get(id) {
                if element.attribute("data-scope") != str_of(region, "scope") {
                    errors.push(format!("SVG network scope differs from model: {id}"));
                }
            }
        }
    }
    if region_elements.keys().copied().collect::<HashSet<_>>() != regions.keys().copied().collect::<HashSet<_>>() {
        errors.push("SVG regions differ from model".into());
    }

    if let Some(expect) = expect {
        let root = doc.root_element();
        if root.attribute("lang") != Some(expect.lang) || root.attribute((XML_NS, "lang")) != Some(expect.lang) {
            errors.push(format!("SVG lang and xml:lang must be {}", expect.lang));
        }
        let style: String = elements.iter().filter(|e| e.tag_name().name() == "style").filter_map(|e| e.text()).collect();
        if !style.contains(&format!("font-family:{}", expect.font_family)) {
            errors.push("SVG font-family differs from the locale".into());
        }
        if expect.lang.to_ascii_lowercase().starts_with("ja") && !JAPANESE_FONTS.iter().any(|f| expect.font_family.contains(f)) {
            errors.push("Japanese output requires a Japanese font in the font stack".into());
        }
    }
    // Escaped text must read back unchanged, and wrapping must neither lose text nor split Latin words.
    let title = elements.iter().find(|e| e.attribute("id") == Some("title")).and_then(|e| e.text());
    if title != str_of(model, "title") {
        errors.push("SVG title differs from model".into());
    }
    for n in items(model, "nodes") {
        let id = str_of(n, "id").unwrap_or_default();
        let element = elements.iter().find(|e| e.attribute("id") == Some(id) && e.attribute("data-kind") == Some("node"));
        let found = element.and_then(|e| e.children().find(|c| c.tag_name().name() == "title")).and_then(|c| c.text());
        let expected = format!("{} / {}", str_of(n, "name").unwrap_or_default(), str_of(n, "role").unwrap_or_default());
        if element.is_some() && found != Some(expected.as_str()) {
            errors.push(format!("SVG node title differs from model: {id}"));
        }
    }
    for (prefix, shown) in displayed(model) {
        let lines: Vec<&str> = elements
            .iter()
            .filter(|e| e.attribute("id").and_then(|i| i.strip_prefix(&format!("{prefix}_"))).is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())))
            .map(|e| e.text().unwrap_or_default())
            .collect();
        let squeeze = |s: &str| s.split_whitespace().collect::<String>();
        if lines.iter().map(|l| squeeze(l)).collect::<String>() != squeeze(&shown) {
            errors.push(format!("SVG text differs from model: {prefix}"));
        } else if ascii_words(&shown).iter().any(|w| !lines.iter().any(|l| l.contains(w))) {
            errors.push(format!("word split across lines: {prefix}"));
        }
    }

    let mut labels: Vec<(String, [f64; 4])> = Vec::new();
    let margin = num_of(model, "outer_margin", 48.0);
    let canvas: Vec<f64> = model["canvas"].as_array().map(|a| a.iter().filter_map(Value::as_f64).collect()).unwrap_or_default();
    if canvas.len() != 2 {
        return err("missing or invalid canvas");
    }
    let (w, h) = (canvas[0], canvas[1]);
    for e in &elements {
        let Some(raw) = attr(*e, "data-text-box") else { continue };
        let id = e.attribute("id").unwrap_or_default().to_string();
        let Some(bx) = serde_json::from_str::<Value>(raw).ok().and_then(|v| rect_of(Some(&v))) else {
            errors.push(format!("invalid text box: {id}"));
            continue;
        };
        labels.push((id.clone(), bx));
        if !contains([margin - 8.0, margin - 8.0, w - 2.0 * margin + 16.0, h - 2.0 * margin + 16.0], bx, 0.1) {
            errors.push(format!("text outside outer margin: {id}"));
        }
        for (node_id, n) in &nodes {
            if id.starts_with(&format!("{node_id}_name_")) || id.starts_with(&format!("{node_id}_role_")) {
                if let Some([x, y, nw, nh]) = rect_of(n.get("geometry")) {
                    if !contains([x - 8.0, y - 2.0, nw + 16.0, nh + 4.0], bx, 0.1) {
                        errors.push(format!("text outside node: {id}"));
                    }
                }
            }
        }
        for (region_id, r) in &regions {
            if id.starts_with(&format!("{region_id}_label_")) && rect_of(r.get("geometry")).is_some_and(|g| !contains(g, bx, 0.1)) {
                errors.push(format!("text outside region: {id}"));
            }
        }
    }
    for (i, (id, bx)) in labels.iter().enumerate() {
        for (other, rect) in &labels[i + 1..] {
            if intersect(*bx, *rect, 1.0) {
                errors.push(format!("overlapping text: {id} / {other}"));
            }
        }
    }
    for c in items(model, "connections") {
        let cid = str_of(c, "id").unwrap_or_default();
        let points: Vec<(f64, f64)> = items(c, "points").iter().filter_map(point_of).collect();
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let line = if a.0 == b.0 {
                [a.0 - 0.5, a.1.min(b.1), 1.0, (a.1 - b.1).abs()]
            } else if a.1 == b.1 {
                [a.0.min(b.0), a.1 - 0.5, (a.0 - b.0).abs(), 1.0]
            } else {
                continue;
            };
            for (id, bx) in &labels {
                if !id.starts_with(&format!("{cid}_label_")) && intersect(line, *bx, 0.2) {
                    errors.push(format!("edge crosses text: {cid} / {id}"));
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error(errors.join("\n")))
    }
}

/// Language-neutral structure of an SVG: everything with an ID except text and its localized scope.
fn structure(path: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
    let doc = Document::parse_with_options(&text, roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() })?;
    let mut out: Vec<String> = doc
        .descendants()
        .filter(|e| e.is_element() && !matches!(e.tag_name().name(), "text" | "title" | "desc") && e.attribute("id").is_some())
        .map(|e| {
            let mut attrs: Vec<String> = e.attributes().filter(|a| !matches!(a.name(), "data-text-box" | "data-scope")).map(|a| format!("{}={}", a.name(), a.value())).collect();
            attrs.sort();
            format!("{} {}", e.tag_name().name(), attrs.join(" "))
        })
        .collect();
    out.sort();
    Ok(out)
}

fn check_locale(path: &Path, raw: &Value, def: &LocaleDef, config: &Config, tokens: &Value) -> Result<Vec<String>> {
    let localized = locale::load_localized(path, raw, def, config)?;
    let model = resolve(&localized)?;
    validate(&model, tokens)?;
    let svg = locale::output_path(&svg_path(path), def);
    let family = def.font_family.as_deref().or(tokens["font"]["family"].as_str()).unwrap_or_default();
    audit_svg_with(&model, &svg, Some(&Expect { lang: &def.lang, font_family: family })).map_err(|e| Error(format!("{}:\n{}", svg.display(), e.0)))?;
    for format in crate::export::declared(raw)? {
        check_export(&model, &locale::output_path(&svg_path(path).with_extension(format.extension()), def), format)?;
    }
    structure(&svg)
}

/// A declared Mermaid / draw.io export must exist and carry every node, region and connection of the model.
fn check_export(model: &Value, path: &Path, format: crate::export::Format) -> Result<()> {
    let text = std::fs::read_to_string(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
    let ids = |key: &str| items(model, key).iter().filter_map(|i| str_of(i, "id")).collect::<Vec<_>>();
    let mut errors: Vec<String> = Vec::new();
    match format {
        crate::export::Format::Drawio => {
            let doc = Document::parse(&text)?;
            let cells: HashSet<&str> = doc.descendants().filter(|n| n.has_tag_name("mxCell")).filter_map(|n| n.attribute("id")).collect();
            for id in ["regions", "nodes", "connections"].iter().flat_map(|k| ids(k)) {
                if !cells.contains(id) {
                    errors.push(format!("draw.io cell missing: {id}"));
                }
            }
        }
        _ => {
            for id in ids("regions").into_iter().chain(ids("nodes")) {
                if !text.lines().any(|l| l.trim_start().starts_with(&format!("{id}[")) || l.trim_start().starts_with(&format!("subgraph {id}["))) {
                    errors.push(format!("Mermaid element missing: {id}"));
                }
            }
            let arrows = text.lines().filter(|l| l.contains("-->") || l.contains("-.->")).count();
            if arrows != items(model, "connections").len() {
                errors.push("Mermaid connections differ from model".into());
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error(format!("{}:\n{}", path.display(), errors.join("\n"))))
    }
}

/// Validate every maintained v2 model and its SVG in every configured locale; returns the
/// number of diagrams checked. Nodes, connections, ownership and icons must match across locales.
pub fn validate_diagrams(root: &Path, tokens: &Value) -> Result<usize> {
    let config = Config::load(root)?;
    let mut count = 0;
    for (path, raw) in model_files(root)? {
        let rel = path.strip_prefix(root).unwrap_or(&path).display().to_string();
        let mut reference: Option<Vec<String>> = None;
        let source = config.source_def()?;
        for def in std::iter::once(source).chain(config.locales.iter().filter(|l| *l != source)) {
            let found = check_locale(&path, &raw, def, &config, tokens).map_err(|e| Error(format!("{rel} [{}]:\n{e}", def.code)))?;
            match &reference {
                None => reference = Some(found),
                Some(expected) if *expected != found => return err(format!("{rel} [{}]: SVG structure differs from {}", def.code, config.source)),
                Some(_) => {}
            }
        }
        count += 1;
    }
    // A locale file must belong to a model beside it and to a configured locale.
    for file in files_under(root, |p| p.parent().and_then(|d| d.file_name()).is_some_and(|n| n == "locales") && p.extension().is_some_and(|e| e == "json")) {
        let suffix = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        let owned = file.parent().and_then(|d| d.parent()).is_some_and(|d| d.join("model.json").is_file());
        if !owned || !config.locales.iter().any(|l| !l.suffix.is_empty() && l.suffix == suffix) {
            return err(format!("{}: locale file has no matching model or configured locale", file.strip_prefix(root).unwrap_or(&file).display()));
        }
    }
    Ok(count)
}
