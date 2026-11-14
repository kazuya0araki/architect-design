//! Localization: English source models, per-diagram translation files keyed by ID.
//!
//! The model keeps IDs, connections, ownership, assets and coordinates in one language-neutral
//! structure. A locale file only overrides display text and, where unavoidable, local text sizing.
use crate::error::{err, Error, Result};
use crate::util::*;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Translatable fields per model section. Other strings are structure or non-rendered metadata.
const FIELDS: [(&str, &[&str]); 3] = [
    ("regions", &["label", "scope"]),
    ("nodes", &["name", "role", "display_name", "display_role"]),
    ("connections", &["label", "data", "method", "frequency"]),
];
const TOP_FIELDS: [&str; 3] = ["title", "purpose", "brand_notice"];
/// Local presentation adjustments a locale file may make; geometry and graph fields are never listed.
const ADJUSTMENTS: [(&str, &[&str]); 3] = [
    ("regions", &["label_size"]),
    ("nodes", &["name_size", "role_size"]),
    ("connections", &["label_position", "label_width"]),
];
const LOCALE_KEYS: [&str; 9] = ["locale", "title", "purpose", "brand_notice", "regions", "nodes", "connections", "untranslated", "adjustments"];

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleDef {
    pub code: String,
    /// File-name suffix (`diagram.<suffix>.svg`, `locales/<suffix>.json`); empty for the source locale.
    pub suffix: String,
    /// BCP 47 tag for the SVG `lang` / `xml:lang` attributes.
    pub lang: String,
    pub font_family: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub source: String,
    pub default: String,
    pub locales: Vec<LocaleDef>,
}

fn normalize(code: &str) -> String {
    code.trim().replace('_', "-").to_ascii_lowercase()
}

impl Config {
    pub fn load(root: &Path) -> Result<Config> {
        let path = root.join("config/diagram.json");
        let raw = read_json(&path)?;
        let at = |m: String| Error(format!("{}: {m}", path.display()));
        let source = req_str(&raw, "source_locale").map_err(|e| at(e.0))?.to_string();
        let default = req_str(&raw, "default_output_locale").map_err(|e| at(e.0))?.to_string();
        let table = raw.get("locales").and_then(Value::as_object).ok_or_else(|| at("missing locales".into()))?;
        let mut locales = Vec::new();
        for (code, def) in table {
            let lang = req_str(def, "lang").map_err(|e| at(format!("{code}: {}", e.0)))?.to_string();
            let suffix = str_of(def, "suffix").unwrap_or_default().to_string();
            if !suffix.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
                return Err(at(format!("{code}: suffix must be lowercase letters, digits and underscores")));
            }
            locales.push(LocaleDef { code: code.clone(), suffix, lang, font_family: str_of(def, "font_family").map(String::from) });
        }
        let config = Config { source, default, locales };
        let source_def = config.get(&config.source).map_err(|e| at(e.0))?;
        if !source_def.suffix.is_empty() {
            return Err(at("the source locale must have an empty suffix".into()));
        }
        config.get(&config.default).map_err(|e| at(e.0))?;
        let mut seen = HashSet::new();
        if !config.locales.iter().all(|l| seen.insert(l.suffix.clone())) {
            return Err(at("duplicate locale suffix".into()));
        }
        Ok(config)
    }

    /// Look up a locale by code, ignoring case and `_` / `-` (`ja_jp` selects `ja-JP`).
    pub fn get(&self, code: &str) -> Result<&LocaleDef> {
        let wanted = normalize(code);
        match self.locales.iter().find(|l| normalize(&l.code) == wanted) {
            Some(def) => Ok(def),
            None => err(format!("unknown locale: {code} (available: {})", self.locales.iter().map(|l| l.code.as_str()).collect::<Vec<_>>().join(", "))),
        }
    }

    pub fn source_def(&self) -> Result<&LocaleDef> {
        self.get(&self.source)
    }

    /// Precedence: explicit request > per-case setting > repository default.
    pub fn select(&self, explicit: Option<&str>, case: Option<&str>) -> Result<&LocaleDef> {
        self.get(explicit.or(case).unwrap_or(&self.default))
    }
}

/// Per-case default (`<model dir>/locale.json`: `{"default_output_locale": "en"}`), if present.
pub fn case_default(model_path: &Path) -> Result<Option<String>> {
    let path = model_path.with_file_name("locale.json");
    if !path.is_file() {
        return Ok(None);
    }
    Ok(str_of(&read_json(&path)?, "default_output_locale").map(String::from))
}

/// `diagram.svg` -> `diagram.ja_jp.svg` (also `.mmd`, `.drawio`); the source locale keeps the plain name.
pub fn output_path(base: &Path, def: &LocaleDef) -> PathBuf {
    if def.suffix.is_empty() {
        return base.to_path_buf();
    }
    let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let ext = base.extension().and_then(|s| s.to_str()).unwrap_or("svg");
    base.with_file_name(format!("{stem}.{}.{ext}", def.suffix))
}

pub fn locale_file(model_path: &Path, def: &LocaleDef) -> PathBuf {
    model_path.with_file_name("locales").join(format!("{}.json", def.suffix))
}

fn is_japanese(c: char) -> bool {
    matches!(c, '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{ff00}'..='\u{ffef}' | '\u{3000}'..='\u{303f}')
}

/// Every translatable `(path, text)` of a model; `path` is `title` or `section.id.field`.
fn texts(model: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for key in TOP_FIELDS {
        if let Some(s) = str_of(model, key).filter(|s| !s.is_empty()) {
            out.push((key.to_string(), s.to_string()));
        }
    }
    for (section, fields) in FIELDS {
        for item in items(model, section) {
            let id = str_of(item, "id").unwrap_or_default();
            for field in fields {
                if let Some(s) = str_of(item, field).filter(|s| !s.is_empty()) {
                    out.push((format!("{section}.{id}.{field}"), s.to_string()));
                }
            }
        }
    }
    out
}

/// The source text must be English so a translation never hides a mixed-language base.
pub fn check_source(model: &Value) -> Result<()> {
    let errors: Vec<String> = texts(model)
        .into_iter()
        .filter(|(_, s)| s.chars().any(is_japanese))
        .map(|(path, _)| format!("source text must be English: {path}"))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error(errors.join("\n")))
    }
}

/// Return `model` with the locale's display text and adjustments applied.
///
/// Fails on missing translations, unknown IDs/fields/keys, unchanged text that is not listed
/// under `untranslated`, and `untranslated` entries that are stale or actually translated.
pub fn localize(model: &Value, locale: &Value, def: &LocaleDef) -> Result<Value> {
    let mut errors: Vec<String> = Vec::new();
    let Some(object) = locale.as_object() else { return err("locale file must be an object") };
    if str_of(locale, "locale").map(normalize) != Some(normalize(&def.code)) {
        errors.push(format!("locale must be \"{}\"", def.code));
    }
    for key in object.keys() {
        if !LOCALE_KEYS.contains(&key.as_str()) {
            errors.push(format!("unknown locale key: {key}"));
        }
    }
    let source = texts(model);
    let paths: HashSet<&str> = source.iter().map(|(p, _)| p.as_str()).collect();
    let mut untranslated: HashSet<String> = HashSet::new();
    for entry in locale.get("untranslated").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
        match entry.as_str() {
            Some(p) if paths.contains(p) => {
                if !untranslated.insert(p.to_string()) {
                    errors.push(format!("duplicate untranslated entry: {p}"));
                }
            }
            Some(p) => errors.push(format!("untranslated entry matches no source text: {p}")),
            None => errors.push("untranslated entries must be strings".into()),
        }
    }

    let mut translated: Vec<(String, String)> = Vec::new();
    for key in TOP_FIELDS {
        if let Some(v) = opt(locale, key) {
            match v.as_str().filter(|s| !s.is_empty()) {
                Some(s) if paths.contains(key) => translated.push((key.to_string(), s.to_string())),
                Some(_) => errors.push(format!("translation for absent source text: {key}")),
                None => errors.push(format!("translation must be a non-empty string: {key}")),
            }
        }
    }
    for (section, fields) in FIELDS {
        let Some(table) = opt(locale, section) else { continue };
        let Some(table) = table.as_object() else {
            errors.push(format!("{section} must be an object keyed by ID"));
            continue;
        };
        let known: HashSet<&str> = items(model, section).iter().filter_map(|i| str_of(i, "id")).collect();
        for (id, entry) in table {
            if !known.contains(id.as_str()) {
                errors.push(format!("unknown {section} ID: {id}"));
                continue;
            }
            let Some(entry) = entry.as_object() else {
                errors.push(format!("{section}.{id} must be an object"));
                continue;
            };
            for (field, value) in entry {
                let path = format!("{section}.{id}.{field}");
                if !fields.contains(&field.as_str()) {
                    errors.push(format!("unknown translatable field: {path}"));
                } else if !paths.contains(path.as_str()) {
                    errors.push(format!("translation for absent source text: {path}"));
                } else if let Some(s) = value.as_str().filter(|s| !s.is_empty()) {
                    translated.push((path, s.to_string()));
                } else {
                    errors.push(format!("translation must be a non-empty string: {path}"));
                }
            }
        }
    }

    let japanese = normalize(&def.lang).starts_with("ja");
    for (path, text) in &source {
        let new = translated.iter().find(|(p, _)| p == path).map(|(_, s)| s);
        match (new, untranslated.contains(path)) {
            (None, false) => errors.push(format!("missing translation: {path}")),
            (Some(_), true) => errors.push(format!("translated text listed as untranslated: {path}")),
            (Some(s), false) => {
                if s == text {
                    errors.push(format!("unchanged text must be listed under untranslated: {path}"));
                } else if japanese && !s.chars().any(is_japanese) {
                    errors.push(format!("no Japanese in translation (list it under untranslated if intentional): {path}"));
                }
            }
            (None, true) => {}
        }
    }

    let mut result = model.clone();
    for (path, text) in translated {
        let parts: Vec<&str> = path.split('.').collect();
        match parts.as_slice() {
            [key] => result[*key] = Value::String(text),
            [section, id, field] => {
                if let Some(item) = result.get_mut(*section).and_then(Value::as_array_mut).and_then(|l| l.iter_mut().find(|i| str_of(i, "id") == Some(id))) {
                    item[*field] = Value::String(text);
                }
            }
            _ => {}
        }
    }
    apply_adjustments(&mut result, locale.get("adjustments"), &mut errors);
    if errors.is_empty() {
        Ok(result)
    } else {
        Err(Error(errors.join("\n")))
    }
}

fn apply_adjustments(model: &mut Value, adjustments: Option<&Value>, errors: &mut Vec<String>) {
    let Some(adjustments) = adjustments else { return };
    let Some(sections) = adjustments.as_object() else {
        errors.push("adjustments must be an object".into());
        return;
    };
    for (section, table) in sections {
        let Some((_, allowed)) = ADJUSTMENTS.iter().find(|(s, _)| s == section) else {
            errors.push(format!("unknown adjustment section: {section}"));
            continue;
        };
        for (id, fields) in table.as_object().unwrap_or(&Map::new()) {
            let Some(item) = model.get_mut(section.as_str()).and_then(Value::as_array_mut).and_then(|l| l.iter_mut().find(|i| str_of(i, "id") == Some(id))) else {
                errors.push(format!("unknown {section} ID in adjustments: {id}"));
                continue;
            };
            for (field, value) in fields.as_object().unwrap_or(&Map::new()) {
                let valid = match field.as_str() {
                    "label_position" => point_of(value).is_some(),
                    _ => value.as_f64().is_some_and(|n| n > 0.0),
                };
                if !allowed.contains(&field.as_str()) {
                    errors.push(format!("adjustment not allowed: {section}.{id}.{field}"));
                } else if !valid {
                    errors.push(format!("invalid adjustment value: {section}.{id}.{field}"));
                } else {
                    item[field.as_str()] = value.clone();
                }
            }
        }
    }
}

/// The model as it is drawn for `def`: the source model, or the source with its locale applied.
pub fn load_localized(model_path: &Path, model: &Value, def: &LocaleDef, config: &Config) -> Result<Value> {
    check_source(model).map_err(|e| Error(format!("{}:\n{}", model_path.display(), e.0)))?;
    if def == config.source_def()? {
        return Ok(model.clone());
    }
    let path = locale_file(model_path, def);
    if !path.is_file() {
        return err(format!("missing locale file: {}", path.display()));
    }
    localize(model, &read_json(&path)?, def).map_err(|e| Error(format!("{}:\n{}", path.display(), e.0)))
}
