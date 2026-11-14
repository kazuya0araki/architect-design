//! Local icon inventory and example references; never fetches updates.
use crate::audit::{files_under, is_model_file};
use crate::error::{Error, Result};
use crate::util::*;
use base64::Engine;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

const SVG_TAG: &str = "http://www.w3.org/2000/svg";

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn ext(path: &Path) -> String {
    path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase()
}

fn parse_xml(text: &str) -> std::result::Result<roxmltree::Document<'_>, roxmltree::Error> {
    roxmltree::Document::parse_with_options(text, roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() })
}

fn strict_base64(s: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}

/// Returns the number of registered icons, or every finding joined by newlines.
pub fn validate_assets(root: &Path) -> Result<usize> {
    let manifest = read_json(&root.join("assets/icons/manifest.json"))?;
    let mut entries: HashMap<String, &Value> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    for item in items(&manifest, "assets") {
        let relative = req_str(item, "path")?;
        if entries.insert(relative.to_string(), item).is_some() {
            errors.push(format!("duplicate manifest path: {relative}"));
        }
        let file = root.join(relative);
        if !file.is_file() {
            errors.push(format!("missing asset: {relative}"));
            continue;
        }
        let data = std::fs::read(&file)?;
        if sha256(&data) != req_str(item, "sha256")? {
            errors.push(format!("SHA-256 mismatch: {relative}"));
        }
        match ext(&file).as_str() {
            "svg" => match std::str::from_utf8(&data).ok().map(parse_xml) {
                Some(Ok(doc)) => {
                    let r = doc.root_element();
                    if r.tag_name().name() != "svg" || r.tag_name().namespace() != Some(SVG_TAG) {
                        errors.push(format!("not an SVG: {relative}"));
                    }
                }
                Some(Err(e)) => errors.push(format!("invalid SVG: {relative}: {e}")),
                None => errors.push(format!("invalid SVG: {relative}: not UTF-8")),
            },
            "png" => {
                let dims = (data.len() >= 24).then(|| {
                    (u32::from_be_bytes([data[16], data[17], data[18], data[19]]), u32::from_be_bytes([data[20], data[21], data[22], data[23]]))
                });
                if data.len() < 33 || data[..8] != *b"\x89PNG\r\n\x1a\n" || data[12..16] != *b"IHDR" || dims.is_none_or(|(w, h)| w.min(h) == 0) {
                    errors.push(format!("invalid PNG header: {relative}"));
                }
            }
            _ => errors.push(format!("unsupported icon format: {relative}")),
        }
    }
    let mut actual = Vec::new();
    crate::audit::walk_files(&root.join("assets/icons"), &mut actual);
    let actual: BTreeSet<String> = actual
        .iter()
        .filter(|p| matches!(ext(p).as_str(), "svg" | "png"))
        .filter_map(|p| p.strip_prefix(root).ok().map(|r| r.to_string_lossy().replace('\\', "/")))
        .collect();
    for p in actual.iter().filter(|p| !entries.contains_key(*p)) {
        errors.push(format!("unregistered asset: {p}"));
    }
    let google: Vec<&Value> = entries.values().copied().filter(|a| str_of(a, "provider") == Some("Google Cloud")).collect();
    for item in &google {
        let path = str_of(item, "path").unwrap_or_default();
        match str_of(item, "collection") {
            Some(c @ ("core" | "categories")) => {
                if !path.starts_with(&format!("assets/icons/google-cloud/{c}/")) {
                    errors.push(format!("noncanonical Google Cloud path: {path}"));
                }
            }
            _ => errors.push(format!("unsupported Google Cloud collection: {path}")),
        }
    }
    let collections: Vec<&Value> = items(&manifest, "collections").iter().filter(|c| str_of(c, "provider") == Some("Google Cloud")).collect();
    let names: HashSet<&str> = collections.iter().filter_map(|c| str_of(c, "collection")).collect();
    if names != HashSet::from(["core", "categories"]) {
        errors.push("Google Cloud collection inventory must contain core and categories".into());
    }
    for collection in &collections {
        let name = str_of(collection, "collection").unwrap_or_default();
        let members: Vec<_> = google.iter().filter(|a| str_of(a, "collection") == Some(name)).collect();
        if collection.get("svg_count").and_then(Value::as_u64) != Some(members.len() as u64) {
            errors.push(format!("collection count mismatch: {name}"));
        }
        if members.iter().any(|a| a.get("archive_sha256") != collection.get("archive_sha256")) {
            errors.push(format!("archive hash mismatch: {name}"));
        }
    }

    let check_reference = |relative: &str, errors: &mut Vec<String>| -> Option<Vec<u8>> {
        let file = root.join(relative);
        if !entries.contains_key(relative) || !file.is_file() {
            errors.push(format!("invalid example asset reference: {relative}"));
            return None;
        }
        std::fs::read(file).ok()
    };
    fn walk(value: &Value, check: &mut dyn FnMut(&str)) {
        match value {
            Value::Object(o) => {
                if let Some(asset) = o.get("asset") {
                    check(asset.as_str().unwrap_or_default());
                }
                o.values().for_each(|v| walk(v, check));
            }
            Value::Array(a) => a.iter().for_each(|v| walk(v, check)),
            _ => {}
        }
    }
    for file in files_under(root, is_model_file) {
        let model = read_json(&file)?;
        walk(&model, &mut |a| {
            check_reference(a, &mut errors);
        });
    }
    let rel = |p: &Path| p.strip_prefix(root).unwrap_or(p).display().to_string();
    for file in files_under(root, |p| ext(p) == "svg") {
        let text = std::fs::read_to_string(&file)?;
        let doc = parse_xml(&text).map_err(|e| Error(format!("{}: {e}", rel(&file))))?;
        for element in doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "image" && n.tag_name().namespace() == Some(SVG_TAG)) {
            let asset = element.attribute("data-asset").unwrap_or_default();
            let data = check_reference(asset, &mut errors);
            let mime = if ext(Path::new(asset)) == "png" { "image/png" } else { "image/svg+xml" };
            let href = element.attribute("href").unwrap_or_default();
            if !href.starts_with(&format!("data:{mime};base64,")) {
                errors.push(format!("wrong embedded image MIME type: {}", rel(&file)));
            }
            match href.split_once(',').and_then(|(_, payload)| strict_base64(payload)) {
                None => errors.push(format!("invalid SVG image embedding: {}: invalid base64 payload", rel(&file))),
                Some(payload) => {
                    if data.is_some_and(|d| d != payload) {
                        errors.push(format!("embedded SVG asset mismatch: {}", rel(&file)));
                    }
                }
            }
        }
    }
    let registered: HashSet<&str> = entries.values().filter_map(|i| str_of(i, "sha256")).collect();
    for file in files_under(root, |p| ext(p) == "drawio") {
        let text = std::fs::read_to_string(&file)?;
        let doc = parse_xml(&text).map_err(|e| Error(format!("{}: {e}", rel(&file))))?;
        for cell in doc.descendants().filter(|n| n.has_tag_name("mxCell")) {
            let style = cell.attribute("style").unwrap_or_default();
            let prefix = "image=data:image/svg+xml,";
            if let Some((_, rest)) = style.split_once(prefix) {
                let payload = rest.split(';').next().unwrap_or_default();
                match strict_base64(payload) {
                    Some(bytes) => {
                        if !registered.contains(sha256(&bytes).as_str()) {
                            errors.push(format!("unregistered draw.io image: {}", rel(&file)));
                        }
                    }
                    None => errors.push(format!("invalid draw.io image embedding: {}: invalid base64 payload", rel(&file))),
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(entries.len())
    } else {
        Err(Error(errors.join("\n")))
    }
}
