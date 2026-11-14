//! Instantiate local diagram fragments; IDs and routes are isolated per instance.
use crate::error::{err, Result};
use crate::util::*;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

fn shift(p: &Value, dx: f64, dy: f64) -> Value {
    match point_of(p) {
        Some((x, y)) => Value::Array(vec![num_value(x + dx), num_value(y + dy)]),
        None => p.clone(),
    }
}

/// Return model lists plus exported ports; the input is never mutated.
///
/// `labels` maps local node/region IDs to display labels. Ports identify nodes or regions;
/// callers explicitly draw connections to their geometry boundaries. Composition never
/// invents cross-fragment data flows or selects products.
pub fn instantiate(fragment: &Value, instance_id: &str, offset: (f64, f64), labels: &HashMap<String, String>) -> Result<Value> {
    let mut chars = instance_id.chars();
    let valid = chars.next().is_some_and(|c| c.is_ascii_lowercase()) && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid {
        return err("instance_id must be a lowercase identifier");
    }
    let mut result = fragment.clone();
    let sections = ["regions", "nodes", "connections"];
    let local_ids: Vec<String> = sections.iter().flat_map(|s| items(&result, s)).map(|i| req_str(i, "id").map(String::from)).collect::<Result<_>>()?;
    if local_ids.len() != local_ids.iter().collect::<HashSet<_>>().len() {
        return err("duplicate fragment ID");
    }
    let ids: HashMap<&str, String> = local_ids.iter().map(|k| (k.as_str(), format!("{instance_id}_{k}"))).collect();
    let (dx, dy) = offset;
    let reference = |key: &str| -> Result<String> {
        match ids.get(key) {
            Some(v) => Ok(v.clone()),
            None => err(format!("unknown fragment reference: {key}")),
        }
    };
    let region_ids: HashSet<String> = items(&result, "regions").iter().filter_map(|r| str_of(r, "id").map(String::from)).collect();
    let element_ids: HashSet<String> = ["regions", "nodes"].iter().flat_map(|s| items(&result, s)).filter_map(|i| str_of(i, "id").map(String::from)).collect();
    let ports = result.get("ports").and_then(Value::as_object).cloned().unwrap_or_default();
    if ports.values().any(|key| !key.as_str().is_some_and(|k| element_ids.contains(k))) {
        return err("unknown fragment reference in ports");
    }
    if labels.keys().any(|k| !element_ids.contains(k.as_str())) {
        return err("unknown label override");
    }
    for section in sections {
        let Some(list) = result.get_mut(section).and_then(Value::as_array_mut) else { continue };
        for item in list.iter_mut() {
            let local = req_str(item, "id")?.to_string();
            item["id"] = Value::String(ids[local.as_str()].clone());
            for key in ["parent_id", "region_id", "stage_id", "source", "target"] {
                if let Some(v) = str_of(item, key).filter(|v| !v.is_empty()) {
                    item[key] = Value::String(reference(v)?);
                }
            }
            if let Some(g) = rect_of(item.get("geometry")) {
                item["geometry"] = Value::Array(vec![num_value(g[0] + dx), num_value(g[1] + dy), num_value(g[2]), num_value(g[3])]);
            }
            if let Some(members) = item.get_mut("fit").and_then(|f| f.get_mut("members")).and_then(Value::as_array_mut) {
                for m in members.iter_mut() {
                    *m = Value::String(reference(m.as_str().unwrap_or_default())?);
                }
            }
            for key in ["points", "junctions"] {
                if let Some(points) = item.get_mut(key).and_then(Value::as_array_mut) {
                    points.iter_mut().for_each(|p| *p = shift(p, dx, dy));
                }
            }
            if let Some(p) = item.get("label_position") {
                item["label_position"] = shift(p, dx, dy);
            }
            if let Some(label) = labels.get(&local) {
                item[if region_ids.contains(&local) { "label" } else { "display_name" }] = Value::String(label.clone());
            }
        }
    }
    let mut mapped = Map::new();
    for (name, key) in &ports {
        mapped.insert(name.clone(), Value::String(reference(key.as_str().unwrap_or_default())?));
    }
    result["ports"] = Value::Object(mapped);
    Ok(result)
}
