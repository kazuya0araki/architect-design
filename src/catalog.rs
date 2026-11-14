//! Build and render the maintained examples. No network access.
use crate::audit::model_files;
use crate::components::instantiate;
use crate::error::Result;
use crate::render::render;
use crate::util::*;
use crate::xml::write_file;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Rebuild the component catalogue from reusable JSON fragments.
pub fn build_catalog(root: &Path) -> Result<()> {
    let directory = root.join("examples");
    let path = directory.join("component-catalog/model.json");
    let mut model = read_json(&path)?;
    let mut parts = Vec::new();
    for (name, prefix, offset) in [("file-receipt", "receipt", (64.0, 112.0)), ("raw-mart", "warehouse", (64.0, 352.0)), ("consumers", "use", (944.0, 112.0))] {
        let fragment = read_json(&directory.join("fragments").join(format!("{name}.json")))?;
        parts.push(instantiate(&fragment, prefix, offset, &HashMap::new())?);
    }
    // Outer presentation frames are owned by the catalogue; internals by fragments.
    let mut regions: Vec<Value> = items(&model, "regions")
        .iter()
        .filter(|r| matches!(str_of(r, "id"), Some("receipt_group" | "consumers_group")))
        .cloned()
        .collect();
    for key in ["nodes", "connections"] {
        model[key] = Value::Array(parts.iter().flat_map(|p| items(p, key).iter().cloned()).collect());
    }
    regions.extend(parts.iter().flat_map(|p| items(p, "regions").iter().cloned()));
    model["regions"] = Value::Array(regions);
    std::fs::write(&path, serde_json::to_string_pretty(&model)? + "\n")?;
    Ok(())
}

/// Rebuild the catalogue, then re-render every maintained v2 example. Returns the example names.
pub fn render_examples(root: &Path, tokens: &Value) -> Result<Vec<String>> {
    build_catalog(root)?;
    let mut names = Vec::new();
    let mut paths = crate::audit::files_under(root, |p| p.file_name().is_some_and(|n| n == "model.json") && p.parent().and_then(|d| d.parent()) == Some(&root.join("examples")));
    paths.sort();
    for path in paths {
        let model = read_json(&path)?;
        if model.get("schema_version").and_then(Value::as_i64) == Some(2) {
            write_file(&render(&model, root, tokens)?, &path.with_file_name("diagram.svg"))?;
            names.push(path.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_default());
        }
    }
    let _ = model_files; // discovery for validation lives in `audit`
    Ok(names)
}
