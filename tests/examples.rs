//! Regression: maintained examples, manifest and catalogue stay consistent with the renderer.
use architect_design::audit::{model_files, svg_path, validate_diagrams};
use architect_design::assets::validate_assets;
use architect_design::export::{drawio, mermaid, Format};
use architect_design::locale::{load_localized, output_path, Config};
use architect_design::render::{render_with, RenderOptions};
use architect_design::util::{find_root, tokens};
use architect_design::xml::to_string;

#[test]
fn every_example_svg_is_up_to_date_and_valid() {
    let root = find_root();
    let tokens = tokens(&root).unwrap();
    let models = model_files(&root).unwrap();
    assert!(models.len() >= 12);
    let config = Config::load(&root).unwrap();
    for (path, model) in &models {
        for def in &config.locales {
            let localized = load_localized(path, model, def, &config).unwrap();
            let options = RenderOptions { lang: &def.lang, font_family: def.font_family.as_deref() };
            let expected = to_string(&render_with(&localized, &root, &tokens, &options).unwrap());
            let target = output_path(&svg_path(path), def);
            let actual = std::fs::read_to_string(&target).unwrap();
            assert_eq!(expected, actual, "{} is stale; run `cargo run -- examples --all-locales`", target.display());
            for format in architect_design::export::declared(model).unwrap() {
                let text = match format {
                    Format::Mermaid => mermaid(&localized, &tokens).unwrap(),
                    _ => drawio(&localized, &root, &tokens, def.font_family.as_deref()).unwrap(),
                };
                let target = output_path(&svg_path(path).with_extension(format.extension()), def);
                let stale = format!("{} is stale; run `cargo run -- examples --all-locales`", target.display());
                assert_eq!(text, std::fs::read_to_string(&target).unwrap(), "{stale}");
            }
        }
    }
    assert_eq!(validate_diagrams(&root, &tokens).unwrap(), models.len());
}

#[test]
fn asset_manifest_is_consistent() {
    assert!(validate_assets(&find_root()).unwrap() > 0);
}
