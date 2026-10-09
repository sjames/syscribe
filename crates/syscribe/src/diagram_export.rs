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
const USAGE: &str = "Usage: syscribe -m <model> diagram export <qname> [--format plantuml|mermaid|svg|dot] [--out <file>]";

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
            syscribe_model::plantuml::render_plantuml_with(elem, elements, Some(&cfg), &syscribe_model::cyber_config::CyberConfig::load(model_root))
                .ok_or_else(|| format!("'{qname}' has a diagramKind with no PlantUML mapping — export mermaid instead"))
        }
        "mermaid" | "svg" | "dot" => {
            let Some((graph, _issues)) = vis::build_graph_with(elem, elements, resolver, &syscribe_model::cyber_config::CyberConfig::load(model_root)) else {
                return Err(format!(
                    "'{qname}' is a hand-authored {} diagram — its body is the source; use `render`",
                    elem.frontmatter.diagram_kind.as_deref().unwrap_or("Custom")
                ));
            };
            let links = link_resolver(elements, resolver, config);
            if format == "mermaid" {
                vis::render_mermaid(&graph, &links).ok_or_else(|| format!("'{qname}' has no Mermaid mapping"))
            } else if format == "dot" {
                Ok(vis::render_dot(&graph))
            } else {
                // REQ-TRS-VIS-016: drawn from pins when fully pinned, else
                // laid out by the embedded ELK first.
                vis::render_svg(&graph, &links).map_err(|e| match e {
                    vis::SvgError::Empty => format!("'{qname}' has no shapes to draw"),
                    other => format!("'{qname}': {other}"),
                })
            }
        }
        other => Err(format!("invalid value '{other}' for --format; valid values: {}", FORMATS.join(", "))),
    }
}

fn cmd_export(elements: &[RawElement], resolver: &Resolver, model_root: &Path, config: &ValidateConfig, args: &[String]) -> i32 {
    let format = crate::cliargs::or_exit(crate::cliargs::enum_value("diagram export", args, "--format", FORMATS, "plantuml"));
    let out_file = args.windows(2).find(|w| w[0] == "--out").map(|w| w[1].as_str());
    let qname = {
        let mut found = None;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--format" || args[i] == "--out" {
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
    let Some(elem) = resolver.resolve_ref(elements, qname) else {
        eprintln!("error: element '{qname}' not found");
        return 1;
    };
    match export_diagram(elem, elements, resolver, model_root, config, format) {
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
