//! Diagram checks: v2 model discovery, model/SVG parity, estimated text bounds and line crossings.
use crate::error::{err, Error, Result};
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

pub fn audit_svg(model: &Value, path: &Path) -> Result<()> {
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

/// Validate every maintained v2 model and its SVG; returns the number of diagrams checked.
pub fn validate_diagrams(root: &Path, tokens: &Value) -> Result<usize> {
    let mut count = 0;
    for (path, raw) in model_files(root)? {
        let rel = path.strip_prefix(root).unwrap_or(&path).display().to_string();
        let result = resolve(&raw).and_then(|model| {
            validate(&model, tokens)?;
            audit_svg(&model, &svg_path(&path))
        });
        if let Err(e) = result {
            return err(format!("{rel}:\n{e}"));
        }
        count += 1;
    }
    Ok(count)
}
