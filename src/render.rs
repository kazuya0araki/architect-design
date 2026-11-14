//! SVG generation from a resolved, validated v2 model.
use crate::error::{err, Result};
use crate::model::{connection_label, resolve, validate};
use crate::text::{text_width, wrap_text};
use crate::util::*;
use crate::xml::Element;
use base64::Engine;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Component, Path};

struct Ctx<'a> {
    root: &'a Path,
    colors: &'a Value,
    line_gap: f64,
    text_boxes: Vec<(String, [f64; 4])>,
}

fn color<'a>(colors: &'a Value, name: &'a str) -> &'a str {
    colors.get(name).and_then(Value::as_str).unwrap_or(name)
}

fn push(parent: &mut Element, tag: &str, attrs: Vec<(&str, String)>, value: Option<&str>) {
    parent.children.push(el(tag, attrs, value));
}

fn el(tag: &str, attrs: Vec<(&str, String)>, value: Option<&str>) -> Element {
    let mut e = Element::new(tag);
    e.attrs = attrs.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    e.text = value.map(String::from);
    e
}

fn n(v: f64) -> String {
    fmt_num(v)
}

impl Ctx<'_> {
    #[allow(clippy::too_many_arguments)]
    fn text(
        &mut self,
        parent: &mut Element,
        id: &str,
        x: f64,
        baseline: f64,
        value: &str,
        size: f64,
        width: f64,
        centered: bool,
        secondary: bool,
        bold: bool,
    ) -> f64 {
        let lines = wrap_text(value, width, size);
        for (i, line) in lines.iter().enumerate() {
            let y = baseline + i as f64 * (size + self.line_gap);
            let measured = text_width(line, size);
            let left = if centered { x - measured / 2.0 } else { x };
            let bx = [left, y - size, measured, size + 2.0];
            let tid = format!("{id}_{i}");
            self.text_boxes.push((tid.clone(), bx));
            let fill = color(self.colors, if secondary { "secondary" } else { "text" });
            push(
                parent,
                "text",
                vec![
                    ("id", tid),
                    ("x", n(x)),
                    ("y", n(y)),
                    ("font-size", n(size)),
                    ("font-weight", if bold { "600" } else { "400" }.into()),
                    ("text-anchor", if centered { "middle" } else { "start" }.into()),
                    ("style", format!("fill:{fill}")),
                    ("data-text-box", py_json(&json!(bx.iter().map(|v| num_value(*v)).collect::<Vec<_>>()))),
                ],
                Some(line),
            );
        }
        baseline + lines.len() as f64 * (size + self.line_gap)
    }

    fn icon(&self, parent: &mut Element, id: &str, asset: &str, x: f64, y: f64, size: f64) -> Result<()> {
        let path = Path::new(asset);
        if path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
            return err(format!("asset path must stay inside the repository: {asset}"));
        }
        let bytes = std::fs::read(self.root.join(path)).map_err(|e| crate::error::Error(format!("{asset}: {e}")))?;
        let data = base64::engine::general_purpose::STANDARD.encode(bytes);
        let png = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("png"));
        let mime = if png { "image/png" } else { "image/svg+xml" };
        push(
            parent,
            "image",
            vec![
                ("id", id.into()),
                ("x", n(x)),
                ("y", n(y)),
                ("width", n(size)),
                ("height", n(size)),
                ("preserveAspectRatio", "xMidYMid meet".into()),
                ("href", format!("data:{mime};base64,{data}")),
                ("data-asset", asset.into()),
            ],
            None,
        );
        Ok(())
    }
}

fn depth(id: &str, regions: &HashMap<&str, &Value>) -> usize {
    match regions.get(id).and_then(|r| str_of(r, "parent_id")).filter(|p| !p.is_empty()) {
        Some(p) => 1 + depth(p, regions),
        None => 0,
    }
}

/// Resolve, validate and draw `model`; returns the SVG tree.
pub fn render(model: &Value, root: &Path, tokens: &Value) -> Result<Element> {
    let model = resolve(model)?;
    validate(&model, tokens)?;
    let (colors, font, spacing) = (&tokens["color"], &tokens["font"], &tokens["spacing"]);
    let cv = |v: &Value, k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    let line_gap = cv(spacing, "line_gap");
    let canvas = model["canvas"].as_array().cloned().unwrap_or_default();
    let (w, h) = (canvas[0].as_f64().unwrap_or(0.0), canvas[1].as_f64().unwrap_or(0.0));
    let mut svg = el(
        "svg",
        vec![
            ("width", n(w)),
            ("height", n(h)),
            ("viewBox", format!("0 0 {} {}", n(w), n(h))),
            ("role", "img".into()),
            ("aria-labelledby", "title description".into()),
            ("data-schema-version", "2".into()),
        ],
        None,
    );
    push(&mut svg, "title", vec![("id", "title".into())], Some(req_str(&model, "title")?));
    let purpose = str_of(&model, "purpose").unwrap_or("");
    let notice = str_of(&model, "brand_notice").unwrap_or("");
    push(&mut svg, "desc", vec![("id", "description".into())], Some(&format!("{purpose} {notice}")));
    let get = |k: &str| model.get(k).cloned().unwrap_or(Value::Null);
    let metadata = json!({
        "id": get("id"), "status": get("status"), "source": get("source"),
        "omitted": model.get("omitted").cloned().unwrap_or(json!([])),
        "view_level": get("view_level"), "diagram_type": get("diagram_type"),
    });
    push(&mut svg, "metadata", vec![("id", "model_metadata".into())], Some(&py_json(&metadata)));
    let mut defs = Element::new("defs");
    let mut marker = el(
        "marker",
        vec![
            ("id", "arrow".into()),
            ("viewBox", "0 0 10 10".into()),
            ("refX", "9".into()),
            ("refY", "5".into()),
            ("markerWidth", "6".into()),
            ("markerHeight", "6".into()),
            ("orient", "auto".into()),
        ],
        None,
    );
    push(
        &mut marker,
        "path",
        vec![("d", "M0 0L10 5L0 10Z".into()), ("fill", color(colors, "connector").into())],
        None,
    );
    defs.children.push(marker);
    svg.children.push(defs);
    let family = font["family"].as_str().unwrap_or_default();
    push(&mut svg, "style", vec![], Some(&format!("text{{font-family:{family};fill:{}}}", color(colors, "text"))));
    push(&mut svg, "rect", vec![("width", n(w)), ("height", n(h)), ("fill", color(colors, "canvas").into())], None);

    let mut frames = el("g", vec![("id", "regions".into())], None);
    let mut edges = el("g", vec![("id", "connections".into())], None);
    let mut content = el("g", vec![("id", "nodes".into())], None);
    let mut labels = el("g", vec![("id", "labels".into())], None);
    let mut ctx = Ctx { root, colors, line_gap, text_boxes: Vec::new() };

    let regions: HashMap<&str, &Value> = items(&model, "regions").iter().filter_map(|r| str_of(r, "id").map(|i| (i, r))).collect();
    let mut ordered: Vec<&Value> = Vec::new();
    for r in items(&model, "regions") {
        // Later duplicates replace earlier ones, as in a dict keyed by ID.
        let id = req_str(r, "id")?;
        if std::ptr::eq(regions[id], r) {
            ordered.push(r);
        }
    }
    ordered.sort_by_key(|r| depth(str_of(r, "id").unwrap_or_default(), &regions));
    for r in ordered {
        let id = req_str(r, "id")?;
        let kind = req_str(r, "kind")?;
        let style = &tokens["region_styles"][kind];
        let g = rect_of(r.get("geometry")).unwrap_or_default();
        let [x, y, rw, rh] = g;
        let header = num_of(r, "header_height", cv(spacing, "header"));
        let sv = |k: &str| style.get(k).and_then(Value::as_str).unwrap_or_default();
        let mut attrs = vec![
            ("id", id.to_string()),
            ("data-region-kind", kind.to_string()),
            ("data-parent", str_of(r, "parent_id").unwrap_or("").to_string()),
            ("data-geometry", py_json(&r["geometry"])),
            ("x", n(x)),
            ("y", n(y)),
            ("width", n(rw)),
            ("height", n(rh)),
            ("fill", color(colors, sv("fill")).into()),
            ("stroke", color(colors, sv("stroke")).into()),
            ("stroke-width", "1".into()),
            ("rx", "4".into()),
        ];
        if kind == "network" {
            attrs.push(("data-scope", req_str(r, "scope")?.to_string()));
        }
        if let Some(dash) = style.get("dash").and_then(Value::as_str) {
            attrs.push(("stroke-dasharray", dash.into()));
        }
        push(&mut frames, "rect", attrs, None);
        if sv("header") == "bar" {
            push(
                &mut frames,
                "rect",
                vec![
                    ("x", n(x + 0.5)),
                    ("y", n(y + 0.5)),
                    ("width", n(rw - 1.0)),
                    ("height", n(header - 1.0)),
                    ("fill", color(colors, "heading").into()),
                ],
                None,
            );
            push(
                &mut frames,
                "path",
                vec![("d", format!("M{} {}H{}", n(x), n(y + header), n(x + rw))), ("stroke", color(colors, "border").into()), ("fill", "none".into())],
                None,
            );
        }
        let mut tx = x + 16.0;
        if let Some(asset) = str_of(r, "asset").filter(|a| !a.is_empty()) {
            ctx.icon(&mut content, &format!("{id}_icon"), asset, x + 12.0, y + 12.0, 36.0)?;
            tx = x + 60.0;
        }
        let stage = kind == "stage";
        let size = num_of(r, "label_size", cv(font, if stage { "heading" } else { "region" }));
        ctx.text(&mut labels, &format!("{id}_label"), tx, y + 32.0, req_str(r, "label")?, size, rw - (tx - x) - 12.0, false, false, stage);
    }

    let role_size = cv(font, "role");
    for c in items(&model, "connections") {
        let cid = req_str(c, "id")?;
        let kind = req_str(c, "kind")?;
        let mut g = el(
            "g",
            vec![
                ("id", cid.into()),
                ("data-source", req_str(c, "source")?.into()),
                ("data-target", req_str(c, "target")?.into()),
                ("data-kind", kind.into()),
                ("data-purpose", str_of(c, "purpose").unwrap_or("").into()),
                ("data-interaction", str_of(c, "interaction").unwrap_or("").into()),
                ("data-transfer-mode", str_of(c, "transfer_mode").unwrap_or("").into()),
            ],
            None,
        );
        let points: Vec<(f64, f64)> = items(c, "points").iter().filter_map(point_of).collect();
        let d = format!("M{}", points.iter().map(|(x, y)| format!("{} {}", n(*x), n(*y))).collect::<Vec<_>>().join(" L"));
        let mut attrs = vec![
            ("d", d),
            ("fill", "none".into()),
            ("stroke", color(colors, "connector").into()),
            ("stroke-width", "1.5".into()),
            ("marker-end", "url(#arrow)".into()),
        ];
        if kind == "control" {
            attrs.push(("stroke-dasharray", "6 5".into()));
        }
        push(&mut g, "path", attrs, None);
        for j in items(c, "junctions") {
            if let Some((x, y)) = point_of(j) {
                push(&mut g, "circle", vec![("cx", n(x)), ("cy", n(y)), ("r", "2.5".into()), ("fill", color(colors, "connector").into())], None);
            }
        }
        edges.children.push(g);
        let label = connection_label(c);
        if !label.is_empty() {
            let (x, y) = point_of(&c["label_position"]).unwrap_or_default();
            let label_width = num_of(c, "label_width", 192.0);
            let lines = wrap_text(&label, label_width, role_size);
            let measured = lines.iter().map(|s| text_width(s, role_size)).fold(0.0, f64::max);
            push(
                &mut labels,
                "rect",
                vec![
                    ("x", n(x - measured / 2.0 - 4.0)),
                    ("y", n(y - role_size - 2.0)),
                    ("width", n(measured + 8.0)),
                    ("height", n(lines.len() as f64 * (role_size + 6.0))),
                    ("fill", color(colors, "canvas").into()),
                ],
                None,
            );
            ctx.text(&mut labels, &format!("{cid}_label"), x, y, &label, role_size, label_width, true, true, false);
        }
    }

    for node in items(&model, "nodes") {
        let id = req_str(node, "id")?;
        let [x, y, nw, nh] = rect_of(node.get("geometry")).unwrap_or_default();
        let basis = node.get("basis").or_else(|| model.get("status")).map(py_str).unwrap_or_else(|| "unspecified".into());
        let mut group = el(
            "g",
            vec![
                ("id", id.into()),
                ("data-kind", "node".into()),
                ("data-region", str_of(node, "region_id").unwrap_or("").into()),
                ("data-stage", str_of(node, "stage_id").unwrap_or("").into()),
                ("data-geometry", py_json(&node["geometry"])),
                ("data-basis", basis),
            ],
            None,
        );
        let name_raw = req_str(node, "name")?;
        let role_raw = str_of(node, "role").unwrap_or("");
        push(&mut group, "title", vec![], Some(&format!("{name_raw} / {role_raw}")));
        let name = str_of(node, "display_name").unwrap_or(name_raw);
        let role = str_of(node, "display_role").unwrap_or(role_raw);
        let icons: Vec<&str> = items(node, "icons").iter().filter_map(|a| str_of(a, "asset")).collect();
        let default_layout = if truthy(node.get("icons")) { "icon" } else { "logical" };
        let layout = str_of(node, "representation").unwrap_or(default_layout);
        let size = num_of(node, "icon_size", cv(spacing, "icon"));
        let name_size = num_of(node, "name_size", cv(font, "name"));
        let role_size = num_of(node, "role_size", cv(font, "role"));
        match layout {
            "icon" => {
                let count = icons.len() as f64;
                let row_width = count * size + (count - 1.0).max(0.0) * 12.0;
                for (i, a) in icons.iter().enumerate() {
                    ctx.icon(&mut group, &format!("{id}_icon_{i}"), a, x + (nw - row_width) / 2.0 + i as f64 * (size + 12.0), y, size)?;
                }
                let baseline = y + size + name_size + 12.0;
                let baseline = ctx.text(&mut labels, &format!("{id}_name"), x + nw / 2.0, baseline, name, name_size, nw + 16.0, true, false, false);
                if !role.is_empty() {
                    ctx.text(&mut labels, &format!("{id}_role"), x + nw / 2.0, baseline + 2.0, role, role_size, nw + 16.0, true, true, false);
                }
            }
            "horizontal" => {
                for (i, a) in icons.iter().enumerate() {
                    ctx.icon(&mut group, &format!("{id}_icon_{i}"), a, x + i as f64 * (size + 12.0), y, size)?;
                }
                let left = x + icons.len() as f64 * (size + 12.0);
                let baseline = ctx.text(&mut labels, &format!("{id}_name"), left, y + name_size, name, name_size, nw - (left - x), false, false, false);
                if !role.is_empty() {
                    ctx.text(&mut labels, &format!("{id}_role"), left, baseline + 2.0, role, role_size, nw - (left - x), false, true, false);
                }
            }
            _ => {
                if layout == "data" {
                    let d = format!(
                        "M{x} {y1}C{x} {ym4} {x2} {ym4} {x2} {y1}V{yb}C{x2} {yp4} {x} {yp4} {x} {yb}Z M{x} {y1}C{x} {y28} {x2} {y28} {x2} {y1}",
                        x = n(x), x2 = n(x + nw), y1 = n(y + 12.0), ym4 = n(y - 4.0), yb = n(y + nh - 12.0), yp4 = n(y + nh + 4.0), y28 = n(y + 28.0)
                    );
                    push(&mut group, "path", vec![("d", d), ("fill", color(colors, "surface").into()), ("stroke", color(colors, "inner_border").into()), ("stroke-width", "1".into())], None);
                } else {
                    push(
                        &mut group,
                        "rect",
                        vec![("x", n(x)), ("y", n(y)), ("width", n(nw)), ("height", n(nh)), ("fill", color(colors, "surface").into()), ("stroke", color(colors, "inner_border").into()), ("stroke-width", "1".into()), ("rx", "3".into())],
                        None,
                    );
                }
                let pad = 8.0;
                let names = wrap_text(name, nw - 2.0 * pad, name_size);
                let roles = if role.is_empty() { vec![] } else { wrap_text(role, nw - 2.0 * pad, role_size) };
                let height = names.len() as f64 * (name_size + 6.0) + roles.len() as f64 * (role_size + 6.0);
                let baseline = y + (nh - height) / 2.0 + name_size;
                let baseline = ctx.text(&mut labels, &format!("{id}_name"), x + nw / 2.0, baseline, name, name_size, nw - 2.0 * pad, true, false, false);
                if !role.is_empty() {
                    ctx.text(&mut labels, &format!("{id}_role"), x + nw / 2.0, baseline + 2.0, role, role_size, nw - 2.0 * pad, true, true, false);
                }
            }
        }
        content.children.push(group);
    }
    // Label extents are included in machine checks and remain inspectable in SVG.
    for (id, bx) in &ctx.text_boxes {
        if !contains([0.0, 0.0, w, h], *bx, 0.1) {
            return err(format!("text outside canvas: {id}"));
        }
    }
    svg.children.extend([frames, edges, content, labels]);
    Ok(svg)
}
