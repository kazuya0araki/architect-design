//! Localization: English source, ID-keyed translations, locale selection and checks.
use architect_design::locale::{case_default, localize, output_path, Config, LocaleDef};
use serde_json::{json, Value};
use std::path::Path;

fn ja() -> LocaleDef {
    LocaleDef { code: "ja-JP".into(), suffix: "ja_jp".into(), lang: "ja-JP".into(), font_family: None }
}

fn model() -> Value {
    json!({
        "title": "Platform", "purpose": "Shows a platform",
        "regions": [{"id": "r", "label": "Scope", "kind": "functional", "geometry": [0, 0, 10, 10]}],
        "nodes": [
            {"id": "a", "name": "Source", "role": "Provides data", "geometry": [0, 0, 1, 1]},
            {"id": "b", "name": "BigQuery", "role": "", "geometry": [0, 0, 1, 1]}],
        "connections": [{"id": "ab", "source": "a", "target": "b", "kind": "data", "label": "Transfer"}]
    })
}

fn locale() -> Value {
    json!({
        "locale": "ja-JP", "title": "基盤", "purpose": "基盤を示す",
        "regions": {"r": {"label": "対象"}},
        "nodes": {"a": {"name": "接続元", "role": "データを提供"}},
        "connections": {"ab": {"label": "転送"}},
        "untranslated": ["nodes.b.name"]
    })
}

fn rejected(locale: Value, needle: &str) {
    let e = localize(&model(), &locale, &ja()).expect_err(needle);
    assert!(e.0.contains(needle), "{needle:?} not in {:?}", e.0);
}

#[test]
fn applies_translations_by_id_and_keeps_structure() {
    let result = localize(&model(), &locale(), &ja()).unwrap();
    assert_eq!(result["title"], "基盤");
    assert_eq!(result["nodes"][0]["name"], "接続元");
    assert_eq!(result["nodes"][1]["name"], "BigQuery");
    assert_eq!(result["connections"][0]["label"], "転送");
    for key in ["nodes", "regions", "connections"] {
        let strip = |m: &Value| -> Vec<Value> { m[key].as_array().unwrap().iter().map(|i| json!({"id": i["id"], "geometry": i["geometry"], "source": i["source"], "target": i["target"]})).collect() };
        assert_eq!(strip(&result), strip(&model()));
    }
}

#[test]
fn rejects_missing_unknown_and_stale_entries() {
    let mut l = locale();
    l["nodes"].as_object_mut().unwrap().remove("a");
    rejected(l, "missing translation: nodes.a.name");
    let mut l = locale();
    l["nodes"]["ghost"] = json!({"name": "幽霊"});
    rejected(l, "unknown nodes ID: ghost");
    let mut l = locale();
    l["nodes"]["a"]["colour"] = json!("赤");
    rejected(l, "unknown translatable field: nodes.a.colour");
    let mut l = locale();
    l["nodes"]["b"] = json!({"role": "役割"});
    rejected(l, "translation for absent source text: nodes.b.role");
    let mut l = locale();
    l["stray"] = json!(1);
    rejected(l, "unknown locale key: stray");
    let mut l = locale();
    l["untranslated"] = json!(["nodes.b.name", "nodes.b.role"]);
    rejected(l, "untranslated entry matches no source text");
    let mut l = locale();
    l["untranslated"] = json!(["nodes.b.name", "title"]);
    rejected(l, "translated text listed as untranslated: title");
}

#[test]
fn untranslated_must_be_explicit_and_translations_must_be_japanese() {
    let mut l = locale();
    l["untranslated"] = json!([]);
    rejected(l, "missing translation: nodes.b.name");
    let mut l = locale();
    l["untranslated"] = json!([]);
    l["nodes"]["b"] = json!({"name": "BigQuery"});
    rejected(l, "unchanged text must be listed under untranslated");
    let mut l = locale();
    l["nodes"]["a"]["role"] = json!("Provides data!");
    rejected(l, "no Japanese in translation");
    let mut l = locale();
    l["locale"] = json!("fr");
    rejected(l, "locale must be");
}

#[test]
fn adjustments_are_limited_to_presentation() {
    let mut l = locale();
    l["adjustments"] = json!({"nodes": {"a": {"name_size": 20}}, "connections": {"ab": {"label_width": 120}}});
    let result = localize(&model(), &l, &ja()).unwrap();
    assert_eq!(result["nodes"][0]["name_size"], 20);
    let mut l = locale();
    l["adjustments"] = json!({"nodes": {"a": {"geometry": [9, 9, 9, 9]}}});
    rejected(l, "adjustment not allowed: nodes.a.geometry");
    let mut l = locale();
    l["adjustments"] = json!({"nodes": {"zzz": {"name_size": 20}}});
    rejected(l, "unknown nodes ID in adjustments: zzz");
}

#[test]
fn locale_selection_and_file_names() {
    let root = architect_design::util::find_root();
    let config = Config::load(&root).unwrap();
    assert_eq!(config.source, "en");
    assert_eq!(config.select(None, None).unwrap().code, "ja-JP");
    assert_eq!(config.select(Some("en"), Some("ja-JP")).unwrap().code, "en");
    assert_eq!(config.select(None, Some("en")).unwrap().code, "en");
    assert_eq!(config.get("ja_jp").unwrap().code, "ja-JP");
    assert!(config.get("fr").unwrap_err().0.contains("unknown locale"));
    let svg = Path::new("examples/x/diagram.svg");
    assert_eq!(output_path(svg, config.get("ja-JP").unwrap()), Path::new("examples/x/diagram.ja_jp.svg"));
    assert_eq!(output_path(svg, config.get("en").unwrap()), svg);
    assert_eq!(case_default(Path::new("/nonexistent/model.json")).unwrap(), None);
}
