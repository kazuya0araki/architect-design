//! Reuse must preserve graph meaning and isolate each instance.
use architect_design::components::instantiate;
use architect_design::model::{resolve, validate};
use architect_design::util::{find_root, read_json, tokens};
use serde_json::{json, Value};
use std::collections::HashMap;

fn no_labels() -> HashMap<String, String> {
    HashMap::new()
}

#[test]
fn repeated_instances_have_distinct_ids_and_valid_paths() {
    let root = find_root();
    let fragment = read_json(&root.join("examples/fragments/file-receipt.json")).unwrap();
    let before = fragment.clone();
    let first = instantiate(&fragment, "first", (64.0, 64.0), &HashMap::from([("file".to_string(), "売上CSV".to_string())])).unwrap();
    let second = instantiate(&fragment, "second", (64.0, 320.0), &no_labels()).unwrap();
    let cat = |key: &str| -> Vec<Value> { [&first, &second].iter().flat_map(|p| p[key].as_array().unwrap().clone()).collect() };
    let model = json!({"schema_version": 2, "canvas": [960, 640], "regions": [], "nodes": cat("nodes"), "connections": cat("connections"), "flow_direction": "left-to-right"});
    validate(&resolve(&model).unwrap(), &tokens(&root).unwrap()).unwrap();
    assert_eq!(fragment, before);
    assert_eq!(first["ports"]["output"], "first_storage");
    assert_eq!(first["nodes"][0]["name"], "受領ファイル");
    assert_eq!(first["nodes"][0]["display_name"], "売上CSV");
    assert!(second["nodes"][0].get("display_name").is_none());
    assert_eq!(first["connections"][0]["points"][0], json!([256, 176]));
}

#[test]
fn nested_fit_and_connection_annotations_move_together() {
    let fragment = json!({
        "regions": [{"id": "r", "kind": "product", "label": "Store", "fit": {"members": ["a"], "padding": 24}}],
        "nodes": [{"id": "a", "name": "A", "role": "", "geometry": [0, 80, 192, 96], "region_id": "r"}],
        "connections": [{"id": "c", "source": "a", "target": "r", "kind": "control", "points": [[0, 100], [0, 200]], "label_position": [0, 50], "junctions": [[0, 150]]}],
        "ports": {"store": "r"}
    });
    let result = instantiate(&fragment, "one", (64.0, 64.0), &no_labels()).unwrap();
    assert_eq!(result["regions"][0]["fit"]["members"], json!(["one_a"]));
    assert_eq!(result["nodes"][0]["region_id"], "one_r");
    assert_eq!(result["connections"][0]["target"], "one_r");
    assert_eq!(result["connections"][0]["label_position"], json!([64, 114]));
    assert_eq!(result["connections"][0]["junctions"], json!([[64, 214]]));
    assert!(resolve(&result).unwrap()["regions"][0].get("geometry").is_some());
}

#[test]
fn invalid_local_references_and_overrides_are_rejected() {
    let mut fragment = json!({"regions": [], "nodes": [{"id": "a", "name": "A", "role": "", "geometry": [0, 0, 192, 96]}], "connections": [], "ports": {"output": "missing"}});
    let message = |f: &Value, id: &str, labels: &HashMap<String, String>| instantiate(f, id, (0.0, 0.0), labels).unwrap_err().0;
    assert!(message(&fragment, "one", &no_labels()).contains("unknown fragment reference"));
    fragment["ports"] = json!({});
    assert!(message(&fragment, "one", &HashMap::from([("missing".to_string(), "X".to_string())])).contains("unknown label override"));
    assert!(message(&fragment, "../bad", &no_labels()).contains("instance_id"));
    let duplicate = fragment["nodes"][0].clone();
    fragment["nodes"].as_array_mut().unwrap().push(duplicate);
    assert!(message(&fragment, "one", &no_labels()).contains("duplicate fragment ID"));
}
