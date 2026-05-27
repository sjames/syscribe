mod compose;
mod layout;
mod list;
mod measure;
mod req;
mod solver;

use clap::{Arg, Command};

use crate::diagram::layout::{IncludeFilter, ViewConfig, ViewPreset};

pub fn cmd_diagram(
    elements: &[syscribe_model::element::RawElement],
    _resolver: &syscribe_model::resolver::Resolver,
    subcmd: &str,
    args: &[String],
) {
    let mut argv: Vec<&str> = vec!["diagram"];
    if !subcmd.is_empty() {
        argv.push(subcmd);
    }
    for a in args {
        argv.push(a.as_str());
    }

    let matches = build_cli().get_matches_from(argv);

    match matches.subcommand() {
        Some(("list", m)) => {
            let type_filter = m.get_one::<String>("type").map(|s| s.as_str());
            let ns_filter = m.get_one::<String>("namespace").map(|s| s.as_str());
            list::cmd_diagram_list(elements, type_filter, ns_filter);
        }
        Some(("measure", m)) => {
            let qnames = m.get_one::<String>("qnames").unwrap();
            let view = view_from_matches(m);
            measure::cmd_diagram_measure(elements, qnames, view);
        }
        Some(("render", m)) => {
            let qname = m.get_one::<String>("qname").unwrap();
            let output = m.get_one::<String>("output").map(|s| s.as_str());
            let view = view_from_matches(m);
            compose::cmd_diagram_render(elements, qname, output, view);
        }
        Some(("compose", m)) => {
            let layout_file = m.get_one::<String>("layout-file").unwrap();
            let output = m.get_one::<String>("output").map(|s| s.as_str());
            let kind = m.get_one::<String>("kind").map(|s| s.as_str()).unwrap_or("arch");
            let ibd = kind == "ibd";
            compose::cmd_diagram_compose(elements, layout_file, output, ibd);
        }
        Some(("layout", m)) => {
            let placement_file = m.get_one::<String>("placement-file").unwrap();
            let output = m.get_one::<String>("output").map(|s| s.as_str());
            let compose_after = m.get_flag("compose");
            let kind = m.get_one::<String>("kind").map(|s| s.as_str());
            let svg_output = m.get_one::<String>("svg").map(|s| s.as_str());
            solver::cmd_diagram_layout(elements, placement_file, output, compose_after, kind, svg_output);
        }
        Some(("req", m)) => {
            let root = m.get_one::<String>("root").unwrap();
            let depth = m.get_one::<String>("depth").and_then(|s| s.parse().ok());
            let show_verify = m.get_flag("show-verify");
            let show_satisfy = m.get_flag("show-satisfy");
            let output = m.get_one::<String>("output").map(|s| s.as_str());
            req::cmd_diagram_req(
                elements,
                req::ReqDiagramOptions { root, depth, show_verify, show_satisfy, output },
            );
        }
        _ => {
            build_cli().print_help().unwrap();
        }
    }
}

fn build_cli() -> Command {
    Command::new("diagram")
        .about("Generate and compose SysML block diagrams as SVG")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list")
                .about("List all elements as JSON")
                .arg(
                    Arg::new("type")
                        .long("type")
                        .short('t')
                        .help("Filter by element type(s), comma-separated (e.g. PartDef,Part)")
                        .value_name("TYPES"),
                )
                .arg(
                    Arg::new("namespace")
                        .long("namespace")
                        .short('n')
                        .help("Filter by qualified-name prefix")
                        .value_name("NS"),
                ),
        )
        .subcommand(
            Command::new("measure")
                .about("Compute box sizes and port anchors as JSON")
                .arg(
                    Arg::new("qnames")
                        .help("Comma-separated qualified names")
                        .required(true)
                        .value_name("QNAMES"),
                )
                .args(view_args()),
        )
        .subcommand(
            Command::new("render")
                .about("Render a single element as SVG")
                .arg(
                    Arg::new("qname")
                        .help("Qualified name of the element to render")
                        .required(true)
                        .value_name("QNAME"),
                )
                .args(view_args())
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Write SVG to FILE (default: stdout)")
                        .value_name("FILE"),
                ),
        )
        .subcommand(
            Command::new("layout")
                .about("Solve element positions from col/row placement using Cassowary constraints")
                .arg(
                    Arg::new("placement-file")
                        .help("JSON placement file with col/row positions")
                        .required(true)
                        .value_name("PLACEMENT.JSON"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Write resolved layout JSON to FILE (default: stdout)")
                        .value_name("FILE"),
                )
                .arg(
                    Arg::new("compose")
                        .long("compose")
                        .help("Pipe resolved layout directly into compose and emit SVG")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("kind")
                        .long("kind")
                        .help("Diagram kind for compose: bdd | ibd | arch (default: from placement file)")
                        .value_name("KIND"),
                )
                .arg(
                    Arg::new("svg")
                        .long("svg")
                        .help("SVG output path when --compose is set (default: stdout)")
                        .value_name("FILE"),
                ),
        )
        .subcommand(
            Command::new("req")
                .about("Auto-layout requirement tree diagram (derive/verify/satisfy edges)")
                .arg(
                    Arg::new("root")
                        .help("Root requirement REQ-* ID or qualified name")
                        .required(true)
                        .value_name("ROOT"),
                )
                .arg(
                    Arg::new("depth")
                        .long("depth")
                        .help("Maximum tree depth to show (default: all)")
                        .value_name("N"),
                )
                .arg(
                    Arg::new("show-verify")
                        .long("show-verify")
                        .help("Include test cases with «verify» edges")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("show-satisfy")
                        .long("show-satisfy")
                        .help("Include architecture elements with «satisfy» edges")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Write SVG to FILE (default: stdout)")
                        .value_name("FILE"),
                ),
        )
        .subcommand(
            Command::new("compose")
                .about("Assemble elements + edges into a full diagram SVG")
                .arg(
                    Arg::new("layout-file")
                        .help("JSON layout file specifying element placements and edges")
                        .required(true)
                        .value_name("LAYOUT.JSON"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Write SVG to FILE (default: stdout)")
                        .value_name("FILE"),
                )
                .arg(
                    Arg::new("kind")
                        .long("kind")
                        .value_name("KIND")
                        .help("Diagram kind: bdd | ibd | arch (default: arch)")
                        .required(false),
                ),
        )
}

fn view_args() -> Vec<Arg> {
    vec![
        Arg::new("view")
            .long("view")
            .help("View preset: full | ports | features | compact | name | requirement")
            .value_name("PRESET")
            .default_value("full"),
        Arg::new("include-ports")
            .long("include-ports")
            .help("Comma-separated port names to show")
            .value_name("PORTS"),
        Arg::new("include-features")
            .long("include-features")
            .help("Comma-separated feature names to show")
            .value_name("FEATURES"),
        Arg::new("min-width")
            .long("min-width")
            .help("Minimum box width in pixels")
            .value_name("N"),
    ]
}

fn view_from_matches(m: &clap::ArgMatches) -> ViewConfig {
    let preset_str = m.get_one::<String>("view").map(|s| s.as_str()).unwrap_or("full");

    let include_ports = m
        .get_one::<String>("include-ports")
        .map(|s| s.split(',').map(|p| p.trim().to_string()).collect());

    let include_features = m
        .get_one::<String>("include-features")
        .map(|s| s.split(',').map(|f| f.trim().to_string()).collect());

    let min_width = m
        .get_one::<String>("min-width")
        .and_then(|s| s.parse().ok());

    ViewConfig {
        preset: ViewPreset::from_str(preset_str),
        include: IncludeFilter { ports: include_ports, features: include_features },
        min_width,
        ibd: false,
    }
}
