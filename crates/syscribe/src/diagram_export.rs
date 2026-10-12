//! `syscribe -m <root> diagram export <qname> --format plantuml|mermaid|svg
//! [--out <file>]` (`REQ-TRS-VIS-009`, `REQ-TRS-VIS-010`).
//!
//! The one `diagram` subcommand left after `ADR-SYS-VIS-001` retired the CLI
//! toolkit: it writes one `Diagram` element's PlantUML, Mermaid or static
//! SVG — each a pure function of the Diagram IR (`syscribe_model::vis`) — to
//! stdout or to `--out`. SVG works on any diagram with an IR: pins are
//! honoured, and an unpinned diagram is laid out by the embedded ELK
//! (`REQ-TRS-VIS-016`). Mermaid and SVG take their `[links]` hyperlinks from
//! `ValidateConfig::hosted_url_for` (inert when `[links]` is absent); PlantUML
//! keeps its own `[plantuml] base_url` links.

use std::path::Path;

use syscribe_model::config::{load_plantuml_config, ValidateConfig};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;
use syscribe_model::vis;

const FORMATS: &[&str] = &["plantuml", "mermaid", "svg", "dot"];
const USAGE: &str = "Usage: syscribe -m <model> diagram export <qname> [--format plantuml|mermaid|svg|dot] [--config <C> [--config <D>]] [--out <file>]";

/// Entry point for `syscribe -m <root> diagram <sub> …`. Returns the exit code.
pub fn cmd_diagram(
    elements: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    config: &ValidateConfig,
    args: &[String],
) -> i32 {
    match args.first().map(String::as_str) {
        Some("export") => cmd_export(elements, resolver, model_root, config, &args[1..]),
        Some(other) => {
            eprintln!("error: unrecognized subcommand '{other}' — the diagram command has one subcommand: export");
            eprintln!("{USAGE}");
            1
        }
        None => {
            eprintln!("{USAGE}");
            1
        }
    }
}

/// The hosted URL of the element a diagram reference names, per the model's
/// `[links]` table — `None` when the reference resolves to no element or no
/// `[links]` is configured.
pub fn link_resolver<'a>(
    elements: &'a [RawElement],
    resolver: &'a Resolver,
    config: &'a ValidateConfig,
) -> impl Fn(&str) -> Option<String> + 'a {
    move |qname: &str| {
        let target = resolver.resolve_ref(elements, qname)?;
        config.hosted_url_for(&target.file_path, &target.qualified_name, target.frontmatter.id.as_deref().unwrap_or(""))
    }
}

/// Render one `Diagram` element in `format`. `Err` carries the message the
/// caller prints (CLI) or returns (MCP), without an `error: ` prefix.
pub fn export_diagram(
    elem: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    config: &ValidateConfig,
    format: &str,
) -> Result<String, String> {
    let qname = &elem.qualified_name;
    if elem.frontmatter.element_type != Some(ElementType::Diagram) {
        return Err(format!("'{qname}' is not a Diagram"));
    }
    match format {
        "plantuml" => {
            let cfg = load_plantuml_config(model_root);
            syscribe_model::plantuml::render_plantuml_in(elem, elements, Some(&cfg), model_root)
                .ok_or_else(|| format!("'{qname}' has a diagramKind with no PlantUML mapping — export mermaid instead"))
        }
        "mermaid" | "svg" | "dot" => {
            let Some((graph, _issues)) = vis::build_graph_in(elem, elements, resolver, model_root) else {
                return Err(hand_authored(elem));
            };
            render_graph_as(elem, graph, elements, resolver, model_root, config, format)
        }
        other => Err(format!("invalid value '{other}' for --format; valid values: {}", FORMATS.join(", "))),
    }
}

fn hand_authored(elem: &RawElement) -> String {
    format!(
        "'{}' is a hand-authored {} diagram — its body is the source; use `render`",
        elem.qualified_name,
        elem.frontmatter.diagram_kind.as_deref().unwrap_or("Custom")
    )
}

/// Render an already-built graph of `elem` in `format`.
fn render_graph_as(
    elem: &RawElement,
    graph: vis::DiagramGraph,
    elements: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    config: &ValidateConfig,
    format: &str,
) -> Result<String, String> {
    let qname = &elem.qualified_name;
    let links = link_resolver(elements, resolver, config);
    match format {
        "plantuml" => syscribe_model::plantuml::render_plantuml_graph(elem, graph, Some(&load_plantuml_config(model_root)))
            .ok_or_else(|| format!("'{qname}' has a diagramKind with no PlantUML mapping — export mermaid instead")),
        "mermaid" => vis::render_mermaid(&graph, &links).ok_or_else(|| format!("'{qname}' has no Mermaid mapping")),
        "dot" => Ok(vis::render_dot(&graph)),
        // REQ-TRS-VIS-016: drawn from pins when fully pinned, else laid out by the embedded ELK first.
        _ => vis::render_svg(&graph, &links).map_err(|e| match e {
            vis::SvgError::Empty => format!("'{qname}' has no shapes to draw"),
            other => format!("'{qname}': {other}"),
        }),
    }
}

/// The variant view (GH #338): `elem` drawn for two Configurations at once, what only one has marked.
fn export_variant(elements: &[RawElement], model_root: &Path, config: &ValidateConfig, qname: &str, configs: [&str; 2], format: &str) -> Result<String, String> {
    let mut graphs = Vec::new();
    let mut labels = Vec::new();
    for c in configs {
        let sel = match syscribe_model::projection::resolve_config_flag(elements, c) {
            syscribe_model::projection::SelectionOutcome::Resolved(sel) => sel,
            syscribe_model::projection::SelectionOutcome::Dormant => return Err(format!("the model has no product line, so '{c}' selects nothing")),
            syscribe_model::projection::SelectionOutcome::Error(m) => return Err(m),
        };
        let mut view = syscribe_model::projection::project(elements, &sel);
        syscribe_model::vis::settle_variant(&mut view);
        let resolver = Resolver::new(&view);
        let Some(elem) = resolver.resolve_ref(&view, qname) else {
            return Err(format!("'{qname}' is not active in configuration '{c}'"));
        };
        if elem.frontmatter.element_type != Some(ElementType::Diagram) {
            return Err(format!("'{qname}' is not a Diagram"));
        }
        let (graph, _issues) = vis::build_graph_in(elem, &view, &resolver, model_root).ok_or_else(|| hand_authored(elem))?;
        graphs.push((graph, elem.clone()));
        // the Configuration's id names it on the diagram
        labels.push(resolver.resolve_ref(&view, c).and_then(|e| e.frontmatter.id.clone()).unwrap_or_else(|| c.to_string()));
    }
    let (b, _) = graphs.pop().unwrap();
    let (a, elem) = graphs.pop().unwrap();
    let merged = syscribe_model::vis::variant::diff(a, b, &labels[0], &labels[1]);
    let resolver = Resolver::new(elements);
    render_graph_as(&elem, merged, elements, &resolver, model_root, config, format)
}

fn cmd_export(elements: &[RawElement], resolver: &Resolver, model_root: &Path, config: &ValidateConfig, args: &[String]) -> i32 {
    let format = crate::cliargs::or_exit(crate::cliargs::enum_value("diagram export", args, "--format", FORMATS, "plantuml"));
    let out_file = args.windows(2).find(|w| w[0] == "--out").map(|w| w[1].as_str());
    let configs: Vec<&str> = args.windows(2).filter(|w| w[0] == "--config").map(|w| w[1].as_str()).collect();
    let config_flag = configs.first().copied();
    let qname = {
        let mut found = None;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--format" || args[i] == "--out" || args[i] == "--config" {
                i += 2;
                continue;
            }
            if args[i].starts_with('-') {
                i += 1;
                continue;
            }
            found = Some(args[i].as_str());
            break;
        }
        found
    };
    let Some(qname) = qname else {
        eprintln!("{USAGE}");
        return 1;
    };
    if configs.len() > 2 {
        eprintln!("error: --config is given at most twice (two products for a variant view)");
        return 1;
    }
    if configs.len() == 2 {
        return emit(export_variant(elements, model_root, config, qname, [configs[0], configs[1]], format), out_file);
    }
    // `--config`: derive from the model projected onto that Configuration (REQ-TRS-BDDCFG-001).
    let projected: Option<Vec<RawElement>> = match config_flag {
        None => None,
        Some(c) => match syscribe_model::projection::resolve_config_flag(elements, c) {
            syscribe_model::projection::SelectionOutcome::Dormant => None,
            syscribe_model::projection::SelectionOutcome::Resolved(sel) => {
                let mut view = syscribe_model::projection::project(elements, &sel);
                // in one product every part drawn is present: no `optional` badge (GH #336)
                syscribe_model::vis::settle_variant(&mut view);
                Some(view)
            }
            syscribe_model::projection::SelectionOutcome::Error(m) => {
                eprintln!("{m}");
                return 1;
            }
        },
    };
    let (full_elements, full_resolver) = (elements, resolver);
    let view_resolver;
    let (elements, resolver): (&[RawElement], &Resolver) = match &projected {
        Some(v) => {
            view_resolver = Resolver::new(v);
            (v.as_slice(), &view_resolver)
        }
        None => (elements, resolver),
    };
    let Some(elem) = resolver.resolve_ref(elements, qname) else {
        match config_flag {
            Some(c) if full_resolver.resolve_ref(full_elements, qname).is_some() => {
                eprintln!("error: '{qname}' exists but is not active in configuration '{c}'")
            }
            Some(_) => eprintln!("error: element '{qname}' not found"),
            None => eprintln!("error: element '{qname}' not found"),
        }
        return 1;
    };
    emit(export_diagram(elem, elements, resolver, model_root, config, format), out_file)
}

/// Print the export, or write it to `out_file`; the exit code.
fn emit(result: Result<String, String>, out_file: Option<&str>) -> i32 {
    match result {
        Ok(text) => match out_file {
            Some(path) => {
                let p = Path::new(path);
                if let Some(parent) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("error: cannot create {}: {e}", parent.display());
                        return 1;
                    }
                }
                if let Err(e) = std::fs::write(p, &text) {
                    eprintln!("error: cannot write {}: {e}", p.display());
                    return 1;
                }
                eprintln!("wrote {}", p.display());
                0
            }
            None => {
                print!("{text}");
                0
            }
        },
        Err(msg) => {
            eprintln!("error: {msg}");
            1
        }
    }
}
