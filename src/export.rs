//! Mermaid and draw.io exports of a localized, resolved model. The model stays the single source:
//! IDs, ownership, connections, icons and coordinates match the SVG; only the notation differs.
use crate::error::{err, Error, Result};
use crate::model::{connection_label, resolve, validate};
use crate::util::*;
use crate::xml::{to_string_plain, Element};
use base64::Engine;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Svg,
    Mermaid,
    Drawio,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Svg => "svg",
            Format::Mermaid => "mmd",
            Format::Drawio => "drawio",
        }
    }

    pub fn parse(name: &str) -> Result<Format> {
        match name {
            "svg" => Ok(Format::Svg),
            "mermaid" => Ok(Format::Mermaid),
            "drawio" => Ok(Format::Drawio),
            other => err(format!("unknown export format: {other} (svg, mermaid, drawio)")),
        }
    }
}

/// Extra formats a model declares (`"exports": ["mermaid", "drawio"]`); SVG is always produced.
pub fn declared(model: &Value) -> Result<Vec<Format>> {
    items(model, "exports").iter().map(|v| Format::parse(v.as_str().unwrap_or_default())).collect()
}

fn one_line(s: &str) -> String {
    s.split('\n').map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ")
}

fn texts(n: &Value) -> (String, String) {
    let name = str_of(n, "display_name").or(str_of(n, "name")).unwrap_or_default();
    let role = str_of(n, "display_role").or(str_of(n, "role")).unwrap_or_default();
    (name.to_string(), role.to_string())
}

/// Container of a region or node: direct ownership first, then the stage.
fn container<'a>(item: &'a Value, own: &str) -> Option<&'a str> {
    ["parent_id", "region_id", "stage_id"].iter().filter_map(|k| str_of(item, k)).find(|c| !c.is_empty() && *c != own)
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "#quot;"))
}

pub fn mermaid(model: &Value, tokens: &Value) -> Result<String> {
    let model = resolve(model)?;
    validate(&model, tokens)?;
    let colors = &tokens["color"];
    type Children<'a> = HashMap<Option<&'a str>, Vec<(&'a Value, bool)>>;
    let mut children: Children = HashMap::new();
    for (item, region) in items(&model, "regions").iter().map(|r| (r, true)).chain(items(&model, "nodes").iter().map(|n| (n, false))) {
        children.entry(container(item, str_of(item, "id").unwrap_or_default())).or_default().push((item, region));
    }
    fn emit(parent: Option<&str>, depth: usize, children: &Children, out: &mut String) {
        let pad = "    ".repeat(depth);
        for (item, region) in children.get(&parent).map(Vec::as_slice).unwrap_or(&[]) {
            let id = str_of(item, "id").unwrap_or_default();
            if *region {
                out.push_str(&format!("{pad}subgraph {id}[{}]\n", quote(&one_line(str_of(item, "label").unwrap_or_default()))));
                emit(Some(id), depth + 1, children, out);
                out.push_str(&format!("{pad}end\n"));
            } else {
                let (name, role) = texts(item);
                let label = [one_line(&name), one_line(&role)].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" / ");
                out.push_str(&format!("{pad}{id}[{}]\n", quote(&label)));
            }
        }
    }
    let mut out = String::from("flowchart LR\n");
    emit(None, 1, &children, &mut out);
    let mut control = Vec::new();
    for (index, c) in items(&model, "connections").iter().enumerate() {
        let (s, t) = (req_str(c, "source")?, req_str(c, "target")?);
        let label = one_line(&connection_label(c));
        let arrow = if req_str(c, "kind")? == "control" { "-.->" } else { "-->" };
        if arrow == "-.->" {
            control.push(index);
        }
        let label = if label.is_empty() { String::new() } else { format!("|{}|", quote(&label)) };
        out.push_str(&format!("    {s} {arrow}{label} {t}\n"));
    }
    let nodes: Vec<&str> = items(&model, "nodes").iter().filter_map(|n| str_of(n, "id")).collect();
    let c = |k: &str| colors.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    out.push_str(&format!("    classDef neutral fill:{},stroke:{},color:{}\n", c("surface"), c("inner_border"), c("text")));
    if !nodes.is_empty() {
        out.push_str(&format!("    class {} neutral\n", nodes.join(",")));
    }
    out.push_str(&format!("    linkStyle default stroke:{}\n", c("connector")));
    Ok(out)
}

fn cell(id: &str, value: &str, style: String, parent: &str, vertex: bool) -> Element {
    let mut e = Element::new("mxCell");
    e.attrs = vec![("id".into(), id.into()), ("value".into(), value.into()), ("style".into(), style)];
    e.attrs.push((if vertex { "vertex" } else { "edge" }.into(), "1".into()));
    e.attrs.push(("parent".into(), parent.into()));
    e
}

fn geometry(g: [f64; 4]) -> Element {
    let mut e = Element::new("mxGeometry");
    e.attrs = vec![("x".into(), fmt_num(g[0])), ("y".into(), fmt_num(g[1])), ("width".into(), fmt_num(g[2])), ("height".into(), fmt_num(g[3])), ("as".into(), "geometry".into())];
    e
}

fn point(tag: &str, p: (f64, f64), role: Option<&str>) -> Element {
    let mut e = Element::new(tag);
    e.attrs = vec![("x".into(), fmt_num(p.0)), ("y".into(), fmt_num(p.1))];
    if let Some(role) = role {
        e.attrs.push(("as".into(), role.into()));
    }
    e
}

fn image_cell(root: &Path, id: &str, asset: &str, g: [f64; 4]) -> Result<Element> {
    let bytes = std::fs::read(root.join(asset)).map_err(|e| Error(format!("{asset}: {e}")))?;
    let mime = if asset.to_ascii_lowercase().ends_with(".png") { "image/png" } else { "image/svg+xml" };
    let data = base64::engine::general_purpose::STANDARD.encode(bytes);
    let mut c = cell(id, "", format!("shape=image;imageAspect=1;aspect=fixed;image=data:{mime},{data};"), "1", true);
    c.children.push(geometry(g));
    Ok(c)
}

fn depth(id: &str, regions: &HashMap<&str, &Value>) -> usize {
    match regions.get(id).and_then(|r| str_of(r, "parent_id")).filter(|p| !p.is_empty()) {
        Some(p) => 1 + depth(p, regions),
        None => 0,
    }
}

pub fn drawio(model: &Value, root: &Path, tokens: &Value, font_family: Option<&str>) -> Result<String> {
    let model = resolve(model)?;
    validate(&model, tokens)?;
    let (colors, font, spacing) = (&tokens["color"], &tokens["font"], &tokens["spacing"]);
    let color = |k: &str| colors.get(k).and_then(Value::as_str).unwrap_or(k).to_string();
    let size = |k: &str| font.get(k).and_then(Value::as_f64).unwrap_or(16.0);
    let family = font_family.or(font["family"].as_str()).unwrap_or_default().split(',').next().unwrap_or_default().to_string();
    let canvas: Vec<f64> = model["canvas"].as_array().map(|a| a.iter().filter_map(Value::as_f64).collect()).unwrap_or_default();
    let mut cells = vec![{
        let mut c = Element::new("mxCell");
        c.attrs = vec![("id".into(), "0".into())];
        c
    }];
    cells.push({
        let mut c = Element::new("mxCell");
        c.attrs = vec![("id".into(), "1".into()), ("parent".into(), "0".into())];
        c
    });

    // Outer regions first so inner regions and nodes draw above them.
    let regions = items(&model, "regions");
    let by_id: HashMap<&str, &Value> = regions.iter().filter_map(|r| str_of(r, "id").map(|i| (i, r))).collect();
    let mut ordered: Vec<&Value> = regions.iter().collect();
    ordered.sort_by_key(|r| depth(str_of(r, "id").unwrap_or_default(), &by_id));
    for r in ordered {
        let id = req_str(r, "id")?;
        let kind = req_str(r, "kind")?;
        let style = &tokens["region_styles"][kind];
        let sv = |k: &str| style.get(k).and_then(Value::as_str).unwrap_or_default();
        let stroke = if sv("stroke") == "none" { "none".to_string() } else { color(sv("stroke")) };
        let dash = style.get("dash").and_then(Value::as_str).map(|d| format!("dashed=1;dashPattern={};", d.replace(' ', " "))).unwrap_or_default();
        let g = rect_of(r.get("geometry")).unwrap_or_default();
        let label_size = num_of(r, "label_size", if kind == "stage" { size("heading") } else { size("region") });
        let bold = if kind == "stage" { "fontStyle=1;" } else { "" };
        let mut c = cell(id, req_str(r, "label")?, format!("rounded=1;arcSize=2;whiteSpace=wrap;html=0;fillColor={};strokeColor={stroke};{dash}verticalAlign=top;align=left;spacingLeft=16;spacingTop=8;fontFamily={family};fontSize={};fontColor={};{bold}", color(sv("fill")), fmt_num(label_size), color("text")), "1", true);
        c.children.push(geometry(g));
        cells.push(c);
        if let Some(asset) = str_of(r, "asset").filter(|a| !a.is_empty()) {
            cells.push(image_cell(root, &format!("{id}_icon"), asset, [g[0] + 12.0, g[1] + 12.0, 36.0, 36.0])?);
        }
    }

    for n in items(&model, "nodes") {
        let id = req_str(n, "id")?;
        let g = rect_of(n.get("geometry")).unwrap_or_default();
        let [x, y, w, h] = g;
        let (name, role) = texts(n);
        let value = if role.is_empty() { name.clone() } else { format!("{name}\n{role}") };
        let icons: Vec<&str> = items(n, "icons").iter().filter_map(|a| str_of(a, "asset")).collect();
        let layout = str_of(n, "representation").unwrap_or(if icons.is_empty() { "logical" } else { "icon" });
        let icon = num_of(n, "icon_size", spacing.get("icon").and_then(Value::as_f64).unwrap_or(56.0));
        let name_size = num_of(n, "name_size", size("name"));
        let base = format!("html=0;whiteSpace=wrap;fontFamily={family};fontSize={};fontColor={};", fmt_num(name_size), color("text"));
        match layout {
            "icon" | "horizontal" => {
                let count = icons.len() as f64;
                let mut c;
                if layout == "icon" {
                    let row = count * icon + (count - 1.0).max(0.0) * 12.0;
                    for (i, a) in icons.iter().enumerate() {
                        cells.push(image_cell(root, &format!("{id}_icon_{i}"), a, [x + (w - row) / 2.0 + i as f64 * (icon + 12.0), y, icon, icon])?);
                    }
                    c = cell(id, &value, format!("text;{base}align=center;verticalAlign=top;"), "1", true);
                    c.children.push(geometry([x, y + icon + 4.0, w, (h - icon - 4.0).max(24.0)]));
                } else {
                    let left = count * (icon + 12.0);
                    for (i, a) in icons.iter().enumerate() {
                        cells.push(image_cell(root, &format!("{id}_icon_{i}"), a, [x + i as f64 * (icon + 12.0), y, icon, icon])?);
                    }
                    c = cell(id, &value, format!("text;{base}align=left;verticalAlign=top;"), "1", true);
                    c.children.push(geometry([x + left, y, (w - left).max(24.0), h]));
                }
                cells.push(c);
            }
            shape => {
                let form = if shape == "data" { "shape=cylinder3;boundedLbl=1;size=12;" } else { "rounded=1;arcSize=4;" };
                let mut c = cell(id, &value, format!("{form}{base}fillColor={};strokeColor={};align=center;verticalAlign=middle;", color("surface"), color("inner_border")), "1", true);
                c.children.push(geometry(g));
                cells.push(c);
            }
        }
    }

    for c in items(&model, "connections") {
        let points: Vec<(f64, f64)> = items(c, "points").iter().filter_map(point_of).collect();
        let dashed = if req_str(c, "kind")? == "control" { "dashed=1;" } else { "dashed=0;" };
        let mut e = cell(req_str(c, "id")?, &one_line(&connection_label(c)), format!("edgeStyle=none;rounded=0;html=0;endArrow=block;endFill=1;strokeColor={};strokeWidth=1.5;{dashed}fontFamily={family};fontSize={};fontColor={};", color("connector"), fmt_num(size("role")), color("secondary")), "1", false);
        e.attrs.push(("source".into(), req_str(c, "source")?.into()));
        e.attrs.push(("target".into(), req_str(c, "target")?.into()));
        let mut g = Element::new("mxGeometry");
        g.attrs = vec![("relative".into(), "1".into()), ("as".into(), "geometry".into())];
        g.children.push(point("mxPoint", points[0], Some("sourcePoint")));
        g.children.push(point("mxPoint", points[points.len() - 1], Some("targetPoint")));
        if points.len() > 2 {
            let mut array = Element::new("Array");
            array.attrs = vec![("as".into(), "points".into())];
            array.children = points[1..points.len() - 1].iter().map(|p| point("mxPoint", *p, None)).collect();
            g.children.push(array);
        }
        e.children.push(g);
        cells.push(e);
    }

    let mut root_el = Element::new("root");
    root_el.children = cells;
    let mut graph = Element::new("mxGraphModel");
    graph.attrs = vec![
        ("grid".into(), "1".into()),
        ("gridSize".into(), "8".into()),
        ("page".into(), "1".into()),
        ("pageWidth".into(), fmt_num(canvas[0])),
        ("pageHeight".into(), fmt_num(canvas[1])),
        ("background".into(), color("canvas")),
    ];
    graph.children.push(root_el);
    let mut diagram = Element::new("diagram");
    diagram.attrs = vec![("id".into(), req_str(&model, "id")?.into()), ("name".into(), req_str(&model, "title")?.into())];
    diagram.children.push(graph);
    let mut file = Element::new("mxfile");
    file.attrs = vec![("host".into(), "app.diagrams.net".into()), ("compressed".into(), "false".into())];
    file.children.push(diagram);
    Ok(to_string_plain(&file))
}
