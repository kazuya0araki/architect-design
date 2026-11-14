//! Build and render the maintained examples. No network access.
use crate::components::instantiate;
use crate::error::Result;
use crate::export::{self, Format};
use crate::locale::{self, Config, LocaleDef};
use crate::render::{render_with, RenderOptions};
use crate::util::*;
use crate::xml::write_file;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

/// Which locales to draw: all configured ones, or one (explicit request, else per-case, else default).
pub enum Selection<'a> {
    All,
    One(Option<&'a str>),
}

/// Draw one model in one locale and format. `output` defaults to `diagram[.<suffix>].<ext>` beside the model.
pub fn render_model(root: &Path, tokens: &Value, config: &Config, model_path: &Path, def: &LocaleDef, format: Format, output: Option<&Path>) -> Result<PathBuf> {
    let raw = read_json(model_path)?;
    let model = locale::load_localized(model_path, &raw, def, config)?;
    let family = def.font_family.as_deref();
    let target = output.map(Path::to_path_buf).unwrap_or_else(|| locale::output_path(&model_path.with_file_name(format!("diagram.{}", format.extension())), def));
    match format {
        Format::Svg => write_file(&render_with(&model, root, tokens, &RenderOptions { lang: &def.lang, font_family: family })?, &target)?,
        Format::Mermaid | Format::Drawio => {
            let text = if format == Format::Mermaid { export::mermaid(&model, tokens)? } else { export::drawio(&model, root, tokens, family)? };
            if let Some(dir) = target.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(&target, text)?;
        }
    }
    Ok(target)
}

/// Locales a model is drawn in under `selection`.
pub fn locales_for<'a>(config: &'a Config, model_path: &Path, selection: &Selection) -> Result<Vec<&'a LocaleDef>> {
    match selection {
        Selection::All => Ok(config.locales.iter().collect()),
        Selection::One(explicit) => Ok(vec![config.select(*explicit, locale::case_default(model_path)?.as_deref())?]),
    }
}

/// Rebuild the catalogue, then re-render every maintained v2 example. Returns the written files.
pub fn render_examples(root: &Path, tokens: &Value, selection: &Selection) -> Result<Vec<String>> {
    let config = Config::load(root)?;
    build_catalog(root)?;
    let mut written = Vec::new();
    let mut paths = crate::audit::files_under(root, |p| p.file_name().is_some_and(|n| n == "model.json") && p.parent().and_then(|d| d.parent()) == Some(&root.join("examples")));
    paths.sort();
    for path in paths {
        if read_json(&path)?.get("schema_version").and_then(Value::as_i64) != Some(2) {
            continue;
        }
        let formats: Vec<Format> = std::iter::once(Format::Svg).chain(export::declared(&read_json(&path)?)?).collect();
        for def in locales_for(&config, &path, selection)? {
            for format in &formats {
                let target = render_model(root, tokens, &config, &path, def, *format, None)?;
                written.push(target.strip_prefix(root).unwrap_or(&target).display().to_string());
            }
        }
    }
    Ok(written)
}
