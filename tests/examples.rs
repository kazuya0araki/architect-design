//! Regression: maintained examples, manifest and catalogue stay consistent with the renderer.
use architect_design::audit::{model_files, svg_path, validate_diagrams};
use architect_design::assets::validate_assets;
use architect_design::render::render;
use architect_design::util::{find_root, tokens};
use architect_design::xml::to_string;

#[test]
fn every_example_svg_is_up_to_date_and_valid() {
    let root = find_root();
    let tokens = tokens(&root).unwrap();
    let models = model_files(&root).unwrap();
    assert!(models.len() >= 12);
    for (path, model) in &models {
        let expected = to_string(&render(model, &root, &tokens).unwrap());
        let actual = std::fs::read_to_string(svg_path(path)).unwrap();
        assert_eq!(expected, actual, "{} is stale; run `cargo run -- examples`", path.display());
    }
    assert_eq!(validate_diagrams(&root, &tokens).unwrap(), models.len());
}

#[test]
fn asset_manifest_is_consistent() {
    assert!(validate_assets(&find_root()).unwrap() > 0);
}
