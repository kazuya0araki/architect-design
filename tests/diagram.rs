//! Behavioural checks for shared SVG layout/graph invariants (migrated from the Python suite).
use architect_design::audit::audit_svg;
use architect_design::error::Error;
use architect_design::model::{resolve, validate};
use architect_design::render::render;
use architect_design::text::wrap_text;
use architect_design::util::{find_root, tokens};
use architect_design::xml::{write_file, Element};
use base64::Engine;
use serde_json::{json, Value};
use std::path::PathBuf;

fn root() -> PathBuf {
    find_root()
}

fn check(model: &Value) -> Result<(), Error> {
    validate(model, &tokens(&root())?)
}

fn fixture() -> Value {
    json!({
        "schema_version": 2, "id": "test", "title": "Test", "canvas": [800, 480], "outer_margin": 48, "flow_direction": "left-to-right",
        "regions": [
            {"id": "left", "label": "Input", "kind": "stage", "geometry": [48, 64, 272, 320]},
            {"id": "right", "label": "Output", "kind": "stage", "geometry": [384, 64, 368, 320]}],
        "nodes": [
            {"id": "a", "name": "Source", "role": "Input data", "geometry": [80, 160, 208, 112], "region_id": "left", "stage_id": "left", "representation": "logical"},
            {"id": "b", "name": "Target", "role": "Output data", "geometry": [408, 160, 240, 112], "region_id": "right", "stage_id": "right", "representation": "logical"}],
        "connections": [{"id": "ab", "source": "a", "target": "b", "kind": "data", "points": [[288, 216], [408, 216]]}]
    })
}

fn rejected(model: &Value, needle: &str) {
    let e = check(model).expect_err(needle);
    assert!(e.0.contains(needle), "{needle:?} not in {:?}", e.0);
}

fn rendered(model: &Value) -> Element {
    render(model, &root(), &tokens(&root()).unwrap()).unwrap()
}

#[test]
fn fits_members_without_mutating_input() {
    let mut m = fixture();
    m["regions"].as_array_mut().unwrap().push(json!({"id": "product", "label": "Product", "kind": "product", "parent_id": "left", "fit": {"members": ["a"], "padding": 24}}));
    let result = resolve(&m).unwrap();
    assert_eq!(result["regions"][2]["geometry"], json!([56, 88, 256, 208]));
    assert!(m["regions"][2].get("geometry").is_none());
    check(&result).unwrap();
}

#[test]
fn nested_fit_and_cycle_rejection() {
    let mut m = fixture();
    m["regions"].as_array_mut().unwrap().extend([
        json!({"id": "inside", "label": "Inside", "kind": "functional", "fit": {"members": ["a"], "padding": 8}}),
        json!({"id": "outside", "label": "Outside", "kind": "functional", "fit": {"members": ["inside"], "padding": 8}}),
    ]);
    let result = resolve(&m).unwrap();
    assert!(result["regions"][3]["geometry"][0].as_f64() < result["regions"][2]["geometry"][0].as_f64());
    m["regions"][2]["fit"]["members"] = json!(["outside"]);
    assert!(resolve(&m).unwrap_err().0.contains("cycle"));
}

#[test]
fn unknown_members_and_ownership_cycle() {
    let mut m = fixture();
    m["regions"][0]["fit"] = json!({"members": ["missing"]});
    assert!(resolve(&m).unwrap_err().0.contains("unknown fit member"));
    let mut m = fixture();
    m["regions"][0]["parent_id"] = json!("right");
    m["regions"][1]["parent_id"] = json!("left");
    assert!(resolve(&m).unwrap_err().0.contains("cycle"));
}

#[test]
fn wrong_parent_and_port_are_rejected() {
    let mut m = fixture();
    m["nodes"][0]["region_id"] = json!("right");
    rejected(&m, "outside parent");
    let mut m = fixture();
    m["connections"][0]["points"][0] = json!([200, 216]);
    rejected(&m, "port");
}

#[test]
fn bent_route_advances_but_folded_route_fails() {
    let mut m = fixture();
    m["connections"][0]["points"] = json!([[288, 216], [344, 216], [344, 240], [408, 240]]);
    check(&m).unwrap();
    m["connections"][0]["points"] = json!([[288, 216], [360, 216], [340, 240], [408, 240]]);
    rejected(&m, "doubles back");
}

#[test]
fn real_return_flow_requires_purpose() {
    let mut m = fixture();
    m["connections"] = json!([{"id": "return", "source": "b", "target": "a", "kind": "data", "points": [[408, 216], [288, 216]], "return_path": true}]);
    rejected(&m, "requires purpose");
    m["connections"][0]["purpose"] = json!("reverse_etl");
    check(&m).unwrap();
}

#[test]
fn svg_matches_graph_and_keeps_lines_behind_labels() {
    let m = fixture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("diagram.svg");
    let mut svg = rendered(&m);
    write_file(&svg, &path).unwrap();
    audit_svg(&resolve(&m).unwrap(), &path).unwrap();
    let ids: Vec<_> = svg.children.iter().filter_map(|e| e.id()).collect();
    let at = |id| ids.iter().position(|x| *x == id).unwrap();
    assert!(at("regions") < at("connections") && at("connections") < at("nodes") && at("nodes") < at("labels"));
    svg.find_by_id_mut("connections").unwrap().children.clear();
    write_file(&svg, &path).unwrap();
    assert!(audit_svg(&resolve(&m).unwrap(), &path).unwrap_err().0.contains("connections differ"));
}

#[test]
fn shared_region_does_not_replace_stage_membership() {
    let mut m = fixture();
    m["nodes"][0]["stage_id"] = json!("right");
    rejected(&m, "outside stage");
}

#[test]
fn official_icon_is_embedded_without_changing_asset() {
    let mut m = fixture();
    let asset = "assets/icons/material-symbols/person.svg";
    let before = std::fs::read(root().join(asset)).unwrap();
    m["nodes"][0]["icons"] = json!([{"asset": asset}]);
    m["nodes"][0]["representation"] = json!("horizontal");
    let svg = rendered(&m);
    let image = svg.iter().into_iter().find(|e| e.tag == "image").unwrap();
    assert_eq!(image.attr("data-asset"), Some(asset));
    assert_eq!(std::fs::read(root().join(asset)).unwrap(), before);
}

#[test]
fn official_png_is_embedded_without_conversion() {
    let mut m = fixture();
    let asset = "assets/icons/apache/iceberg/iceberg-logo-icon.png";
    m["nodes"][0]["icons"] = json!([{"asset": asset}]);
    m["nodes"][0]["representation"] = json!("horizontal");
    let svg = rendered(&m);
    let href = svg.iter().into_iter().find(|e| e.tag == "image").unwrap().attr("href").unwrap();
    let payload = href.strip_prefix("data:image/png;base64,").expect("PNG MIME type");
    let decoded = base64::engine::general_purpose::STANDARD.decode(payload).unwrap();
    assert_eq!(decoded, std::fs::read(root().join(asset)).unwrap());
}

#[test]
fn connection_metadata_is_displayed_without_inventing_values() {
    let mut m = fixture();
    m["connections"][0]["method"] = json!("API");
    m["connections"][0]["frequency"] = json!("日次");
    rejected(&m, "label_position");
    m["connections"][0]["label_position"] = json!([348, 132]);
    let svg = rendered(&m);
    let text: String = svg.iter().into_iter().filter(|e| e.tag == "text").filter_map(|e| e.text.as_deref()).collect();
    assert!(text.contains("API / 日次"));
    m["connections"][0]["junctions"] = json!([[700, 400]]);
    rejected(&m, "junction outside");
}

#[test]
fn view_levels_and_network_boundaries() {
    let mut m = fixture();
    m["view_level"] = json!("pipeline");
    m["diagram_type"] = json!("context");
    rejected(&m, "context diagram");
    m["view_level"] = json!("detail");
    m["diagram_type"] = json!("deployment");
    m["regions"][0]["kind"] = json!("network");
    // Nodes in this fixture are also classified by stage; remove that axis.
    for n in m["nodes"].as_array_mut().unwrap() {
        n.as_object_mut().unwrap().remove("stage_id");
    }
    rejected(&m, "explicit scope");
    m["regions"][0]["scope"] = json!("Proposed private network");
    check(&m).unwrap();
    m["diagram_type"] = json!("data_flow");
    rejected(&m, "requires deployment");
    let mut m = fixture();
    m["view_level"] = json!("unknown");
    rejected(&m, "invalid view_level");
}

#[test]
fn query_result_and_transfer_mode_semantics() {
    let mut m = fixture();
    m["connections"][0]["interaction"] = json!("query");
    rejected(&m, "query must be control");
    m["connections"][0]["kind"] = json!("control");
    check(&m).unwrap();
    m["connections"][0]["transfer_mode"] = json!("batch");
    rejected(&m, "transfer_mode requires");
    m["connections"][0].as_object_mut().unwrap().extend([("kind".into(), json!("data")), ("interaction".into(), json!("transfer")), ("transfer_mode".into(), json!("cdc"))]);
    check(&m).unwrap();
    m["connections"][0]["transfer_mode"] = json!("unknown");
    rejected(&m, "transfer_mode requires");
    m["connections"][0].as_object_mut().unwrap().remove("transfer_mode");
    m["connections"][0]["interaction"] = json!("result");
    m["connections"][0]["kind"] = json!("control");
    rejected(&m, "result must be data");
    m["connections"][0]["kind"] = json!("data");
    check(&m).unwrap();
}

#[test]
fn svg_connection_semantics_cannot_go_stale() {
    let mut m = fixture();
    m["connections"][0]["interaction"] = json!("transfer");
    m["connections"][0]["transfer_mode"] = json!("cdc");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("diagram.svg");
    let mut svg = rendered(&m);
    write_file(&svg, &path).unwrap();
    audit_svg(&resolve(&m).unwrap(), &path).unwrap();
    svg.find_by_id_mut("ab").unwrap().set("data-transfer-mode", "batch");
    write_file(&svg, &path).unwrap();
    assert!(audit_svg(&resolve(&m).unwrap(), &path).unwrap_err().0.contains("semantics differ"));
}

#[test]
fn word_wrap_preserves_product_words_and_explicit_lines() {
    assert_eq!(wrap_text("Great Expectations", 176.0, 22.0), ["Great", "Expectations"]);
    assert_eq!(wrap_text("Cloud Storage", 176.0, 24.0), ["Cloud Storage"]);
    assert_eq!(wrap_text("Raw\nMart", 176.0, 24.0), ["Raw", "Mart"]);
}
