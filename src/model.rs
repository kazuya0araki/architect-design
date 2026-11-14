//! Model v2: fitted-region resolution and structural validation.
//! Models stay as `serde_json::Value` so unknown fields round-trip untouched.
use crate::error::{err, Error, Result};
use crate::util::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub fn connection_label(c: &Value) -> String {
    if let Some(label) = c.get("label") {
        return py_str(label);
    }
    ["data", "method", "frequency"]
        .iter()
        .filter(|k| truthy(c.get(**k)))
        .map(|k| py_str(&c[*k]))
        .collect::<Vec<_>>()
        .join(" / ")
}

/// Resolve fitted regions inside-out. Stages and ownership remain independent.
pub fn resolve(model: &Value) -> Result<Value> {
    let mut model = model.clone();
    let region_index: HashMap<String, usize> = items(&model, "regions")
        .iter()
        .enumerate()
        .filter_map(|(i, r)| str_of(r, "id").map(|id| (id.to_string(), i)))
        .collect();
    let nodes: HashMap<String, Value> = items(&model, "nodes")
        .iter()
        .filter_map(|n| str_of(n, "id").map(|id| (id.to_string(), n.clone())))
        .collect();
    let regions: Vec<Value> = items(&model, "regions").to_vec();
    let mut geometries: HashMap<String, [f64; 4]> = HashMap::new();
    let mut active: HashSet<String> = HashSet::new();

    struct Ctx<'a> {
        regions: &'a [Value],
        index: &'a HashMap<String, usize>,
        nodes: &'a HashMap<String, Value>,
    }

    fn geometry(
        id: &str,
        ctx: &Ctx,
        done: &mut HashMap<String, [f64; 4]>,
        active: &mut HashSet<String>,
    ) -> Result<[f64; 4]> {
        if active.contains(id) {
            return err(format!("region cycle: {id}"));
        }
        if let Some(g) = done.get(id) {
            return Ok(*g);
        }
        let Some(&i) = ctx.index.get(id) else {
            return err(format!("unknown region: {id}"));
        };
        let r = &ctx.regions[i];
        active.insert(id.to_string());
        if let Some(parent) = str_of(r, "parent_id").filter(|p| !p.is_empty()) {
            // Check the ownership tree independently of content fitting.
            let mut seen = HashSet::from([id.to_string()]);
            let mut parent = Some(parent.to_string());
            while let Some(p) = parent.filter(|p| !p.is_empty()) {
                if !seen.insert(p.clone()) {
                    return err(format!("region cycle: {id}"));
                }
                let Some(&pi) = ctx.index.get(&p) else {
                    return err(format!("unknown region: {p}"));
                };
                parent = str_of(&ctx.regions[pi], "parent_id").map(String::from);
            }
        }
        let result = if let Some(fit) = r.get("fit") {
            let members = items(fit, "members");
            if members.is_empty() {
                return err(format!("empty fitted region: {id}"));
            }
            let mut rects = Vec::new();
            for member in members {
                let name = member.as_str().unwrap_or_default();
                if let Some(n) = ctx.nodes.get(name) {
                    match rect_of(n.get("geometry")) {
                        Some(g) => rects.push(g),
                        None => return err(format!("invalid geometry: {name}")),
                    }
                } else if ctx.index.contains_key(name) {
                    rects.push(geometry(name, ctx, done, active)?);
                } else {
                    return err(format!("unknown fit member: {name}"));
                }
            }
            let padding = num_of(fit, "padding", 24.0);
            let header = num_of(r, "header_height", 48.0);
            let left = rects.iter().map(|g| g[0]).fold(f64::INFINITY, f64::min) - padding;
            let top = rects.iter().map(|g| g[1]).fold(f64::INFINITY, f64::min) - padding - header;
            let right = rects.iter().map(|g| g[0] + g[2]).fold(f64::NEG_INFINITY, f64::max) + padding;
            let bottom = rects.iter().map(|g| g[1] + g[3]).fold(f64::NEG_INFINITY, f64::max) + padding;
            [left, top, right - left, bottom - top]
        } else {
            match rect_of(r.get("geometry")) {
                Some(g) => g,
                None if r.get("geometry").is_some() => return err(format!("invalid geometry: {id}")),
                None => return err(format!("missing region geometry: {id}")),
            }
        };
        active.remove(id);
        done.insert(id.to_string(), result);
        Ok(result)
    }

    let ctx = Ctx { regions: &regions, index: &region_index, nodes: &nodes };
    for r in &regions {
        if let Some(id) = str_of(r, "id") {
            geometry(id, &ctx, &mut geometries, &mut active)?;
        }
    }
    if let Some(list) = model.get_mut("regions").and_then(Value::as_array_mut) {
        for r in list.iter_mut() {
            let Some(id) = str_of(r, "id").map(String::from) else { continue };
            if r.get("fit").is_some() {
                let g = geometries[&id];
                r["geometry"] = Value::Array(g.iter().map(|v| num_value(*v)).collect());
            }
        }
    }
    Ok(model)
}

fn on_edge(r: [f64; 4], p: (f64, f64)) -> bool {
    let [x, y, w, h] = r;
    let (px, py) = p;
    (((px - x).abs() < 0.1 || (px - x - w).abs() < 0.1) && y - 0.1 <= py && py <= y + h + 0.1)
        || (((py - y).abs() < 0.1 || (py - y - h).abs() < 0.1) && x - 0.1 <= px && px <= x + w + 0.1)
}

pub fn validate(model: &Value, tokens: &Value) -> Result<()> {
    let mut errors: Vec<String> = Vec::new();
    if model.get("schema_version").and_then(Value::as_i64) != Some(2) {
        errors.push("expected SVG model schema_version 2".into());
    }
    let level = str_of(model, "view_level");
    let diagram_type = str_of(model, "diagram_type");
    if opt(model, "view_level").is_some() && !matches!(level, Some("context" | "pipeline" | "detail")) {
        errors.push("invalid view_level".into());
    }
    if opt(model, "diagram_type").is_some()
        && !matches!(diagram_type, Some("context" | "data_flow" | "deployment" | "component_catalog"))
    {
        errors.push("invalid diagram_type".into());
    }
    if diagram_type == Some("context") && !matches!(level, None | Some("context")) {
        errors.push("context diagram requires context view_level".into());
    }
    if diagram_type == Some("deployment") && !matches!(level, None | Some("detail")) {
        errors.push("deployment diagram requires detail view_level".into());
    }
    let regions = items(model, "regions");
    let nodes = items(model, "nodes");
    let all: Vec<(&Value, bool)> = regions.iter().map(|r| (r, true)).chain(nodes.iter().map(|n| (n, false))).collect();
    let mut ids: Vec<&str> = Vec::new();
    for (item, _) in &all {
        ids.push(req_str(item, "id")?);
    }
    if ids.len() != ids.iter().collect::<HashSet<_>>().len() {
        errors.push("duplicate region/node ID".into());
    }
    let by_id: HashMap<&str, &Value> = all.iter().map(|(v, _)| (req_str(v, "id").unwrap_or_default(), *v)).collect();
    let region_by_id: HashMap<&str, &Value> = regions.iter().map(|r| (req_str(r, "id").unwrap_or_default(), r)).collect();
    let canvas: Vec<f64> = model
        .get("canvas")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default();
    if canvas.len() != 2 {
        return err("missing or invalid canvas");
    }
    let (cw, ch) = (canvas[0], canvas[1]);
    let margin = num_of(model, "outer_margin", 48.0);
    let inner_canvas = [margin, margin, cw - 2.0 * margin, ch - 2.0 * margin];
    let region_styles = &tokens["region_styles"];

    for (item, is_region) in &all {
        let id = req_str(item, "id")?;
        let Some(geometry) = rect_of(item.get("geometry")).filter(|g| g[2] > 0.0 && g[3] > 0.0) else {
            errors.push(format!("invalid geometry: {id}"));
            continue;
        };
        if !contains(inner_canvas, geometry, 0.1) {
            errors.push(format!("outside outer margin: {id}"));
        }
        let parent = str_of(item, "parent_id").filter(|p| !p.is_empty()).or_else(|| str_of(item, "region_id"));
        if let Some(parent) = parent.filter(|p| !p.is_empty()) {
            match region_by_id.get(parent) {
                None => errors.push(format!("unknown parent: {id} -> {parent}")),
                Some(p) => {
                    if rect_of(p.get("geometry")).is_some_and(|pg| !contains(pg, geometry, 0.1)) {
                        errors.push(format!("outside parent: {id} -> {parent}"));
                    }
                }
            }
        }
        if let Some(stage) = str_of(item, "stage_id").filter(|s| !s.is_empty()) {
            match region_by_id.get(stage) {
                Some(s) if str_of(s, "kind") == Some("stage") => {
                    if rect_of(s.get("geometry")).is_some_and(|sg| !contains(sg, geometry, 0.1)) {
                        errors.push(format!("outside stage: {id} -> {stage}"));
                    }
                }
                _ => errors.push(format!("invalid stage: {id} -> {stage}")),
            }
        }
        if *is_region {
            let kind = req_str(item, "kind")?;
            if region_styles.get(kind).is_none() {
                errors.push(format!("unknown region kind: {id}"));
            }
            if kind == "network" {
                if diagram_type != Some("deployment") {
                    errors.push(format!("network boundary requires deployment diagram: {id}"));
                }
                if !str_of(item, "scope").is_some_and(|s| !s.trim().is_empty()) {
                    errors.push(format!("network boundary requires explicit scope: {id}"));
                }
            }
        } else {
            let representation = opt(item, "representation").and_then(Value::as_str).unwrap_or("logical");
            if !matches!(representation, "icon" | "horizontal" | "logical" | "data") {
                errors.push(format!("unknown representation: {id}"));
            }
        }
    }

    let connections = items(model, "connections");
    let mut connection_ids: Vec<&str> = Vec::new();
    for c in connections {
        connection_ids.push(req_str(c, "id")?);
    }
    let unique: HashSet<&str> = connection_ids.iter().copied().collect();
    if connection_ids.len() != unique.len() || connection_ids.iter().any(|c| ids.contains(c)) {
        errors.push("duplicate connection ID".into());
    }
    for c in connections {
        let cid = req_str(c, "id")?;
        let (source, target) = (req_str(c, "source")?, req_str(c, "target")?);
        let (Some(src), Some(dst)) = (by_id.get(source), by_id.get(target)) else {
            errors.push(format!("unknown edge endpoint: {cid}"));
            continue;
        };
        let kind = req_str(c, "kind")?;
        if !matches!(kind, "data" | "control") {
            errors.push(format!("invalid connection kind: {cid}"));
        }
        let interaction = opt(c, "interaction");
        let interaction_name = interaction.and_then(Value::as_str);
        if interaction.is_some() && !matches!(interaction_name, Some("transfer" | "query" | "result" | "metadata")) {
            errors.push(format!("invalid interaction: {cid}"));
        }
        if interaction_name == Some("query") && kind != "control" {
            errors.push(format!("query must be control: {cid}"));
        }
        if let Some(i @ ("transfer" | "result" | "metadata")) = interaction_name {
            if kind != "data" {
                errors.push(format!("{i} must be data: {cid}"));
            }
        }
        if let Some(mode) = opt(c, "transfer_mode") {
            if interaction_name != Some("transfer") || !matches!(mode.as_str(), Some("batch" | "streaming" | "cdc")) {
                errors.push(format!("transfer_mode requires transfer and batch/streaming/cdc: {cid}"));
            }
        }
        let points: Vec<(f64, f64)> = items(c, "points").iter().filter_map(point_of).collect();
        if items(c, "points").len() < 2 {
            errors.push(format!("missing path: {cid}"));
            continue;
        }
        if points.len() != items(c, "points").len() {
            errors.push(format!("invalid path: {cid}"));
            continue;
        }
        if points.iter().any(|&(x, y)| !(0.0 <= x && x <= cw && 0.0 <= y && y <= ch)) {
            errors.push(format!("path outside canvas: {cid}"));
        }
        for (item_id, item, point) in [(source, src, points[0]), (target, dst, points[points.len() - 1])] {
            if let Some(g) = rect_of(item.get("geometry")) {
                if !on_edge(g, point) {
                    errors.push(format!("path port outside endpoint: {cid} -> {item_id}"));
                }
            }
        }
        if kind == "data" && str_of(model, "flow_direction") == Some("left-to-right") && !truthy(c.get("return_path")) {
            if let (Some(s), Some(t)) = (rect_of(src.get("geometry")), rect_of(dst.get("geometry"))) {
                if s[0] + s[2] > t[0] + 0.1 || points.windows(2).any(|w| w[0].0 > w[1].0) {
                    errors.push(format!("data flow doubles back: {cid}"));
                }
            }
        }
        if truthy(c.get("return_path")) && !truthy(c.get("purpose")) {
            errors.push(format!("return path requires purpose: {cid}"));
        }
        if !connection_label(c).is_empty() && !truthy(c.get("label_position")) {
            errors.push(format!("label requires label_position: {cid}"));
        }
        for j in items(c, "junctions") {
            let Some((px, py)) = point_of(j) else {
                errors.push(format!("junction outside path: {cid}"));
                continue;
            };
            let on_path = points.windows(2).any(|w| {
                let (a, b) = (w[0], w[1]);
                ((b.0 - a.0) * (py - a.1) - (b.1 - a.1) * (px - a.0)).abs() < 0.1
                    && a.0.min(b.0) - 0.1 <= px
                    && px <= a.0.max(b.0) + 0.1
                    && a.1.min(b.1) - 0.1 <= py
                    && py <= a.1.max(b.1) + 0.1
            });
            if !on_path {
                errors.push(format!("junction outside path: {cid}"));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error(errors.join("\n")))
    }
}
