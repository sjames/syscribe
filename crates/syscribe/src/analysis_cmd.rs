//! Diagram output of the analysis reports (GH #223): `zones --format`,
//! `cyber-risk --format dot|mermaid|plantuml|svg` and `hara trace`.
//!
//! The graphs are the `Traceability`, `ZoneConduit` and `ThreatGraph` kinds of
//! the Diagram IR over the whole model (or one subject), written by the same
//! writers `diagram export` uses.

use syscribe_model::cyber_config::CyberConfig;
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::results::ResultsData;
use syscribe_model::safety_case::Verdict;
use syscribe_model::vis::{self, DiagramGraph, DiagramKind};

use crate::query::{tc_verdict, TcVerdict};

/// The diagram formats of the analysis reports.
pub const GRAPH_FORMATS: &[&str] = &["dot", "mermaid", "plantuml", "svg"];

/// Whether `--format <value>` asks for a diagram rather than a table.
pub fn is_graph_format(value: &str) -> bool {
    GRAPH_FORMATS.contains(&value)
}

/// The value of `--format`, when present.
pub fn format_arg(args: &[String]) -> Option<&str> {
    args.windows(2).find(|w| w[0] == "--format").map(|w| w[1].as_str())
}

/// Write `graph` in `format` (one of [`GRAPH_FORMATS`]).
pub fn render(graph: &DiagramGraph, format: &str) -> Result<String, String> {
    let no_links = |_: &str| None;
    match format {
        "dot" => Ok(vis::render_dot(graph)),
        "mermaid" => vis::render_mermaid(graph, &no_links).ok_or_else(|| "no Mermaid mapping".to_string()),
        "plantuml" => Ok(syscribe_model::plantuml::render_analysis_plantuml(graph, graph.kind.as_str())),
        "svg" => vis::render_svg(graph, &no_links).map_err(|e| match e {
            vis::SvgError::Empty => "nothing to draw".to_string(),
            other => other.to_string(),
        }),
        other => Err(format!("invalid value '{other}' for --format; valid values: {}", GRAPH_FORMATS.join(", "))),
    }
}

/// Print the whole-model graph of `kind` in `format`; the process exit code.
pub fn cmd_model_graph(
    cmd: &str,
    kind: DiagramKind,
    elements: &[RawElement],
    cyber: &CyberConfig,
    results: Option<&ResultsData>,
    format: &str,
) -> i32 {
    let resolver = Resolver::new(elements);
    let verdict_of = verdicts(results);
    let (graph, _issues) = vis::build_model_graph(kind, elements, &resolver, cyber, &verdict_of);
    emit(cmd, &graph, format)
}

fn emit(cmd: &str, graph: &DiagramGraph, format: &str) -> i32 {
    if graph.nodes.is_empty() {
        eprintln!("{cmd}: nothing to draw — the model has no {} content", graph.kind.as_str());
        return 1;
    }
    match render(graph, format) {
        Ok(text) => {
            print!("{text}");
            if !text.ends_with('\n') {
                println!();
            }
            0
        }
        Err(e) => {
            eprintln!("{cmd}: {e}");
            1
        }
    }
}

/// The verdict of a test case from the results sidecar (`unknown` without one).
fn verdicts(results: Option<&ResultsData>) -> impl Fn(&RawElement) -> Verdict + '_ {
    move |tc: &RawElement| match tc_verdict(tc, results) {
        TcVerdict::Pass => Verdict::Pass,
        TcVerdict::Fail => Verdict::Fail,
        TcVerdict::Unknown => Verdict::Unknown,
    }
}

pub const TRACE_USAGE: &str =
    "Usage: syscribe -m <model> hara trace [<SafetyGoal|HazardousEvent|Requirement|Package>] [--format dot|mermaid|plantuml|svg]";

/// `hara trace [<subject>] [--format dot|mermaid|plantuml|svg]`: the hazard-to-test
/// graph (HazardousEvent, SafetyGoal, Requirements, TestCases with the ingested
/// verdicts, fault tree and argument), whole model unless a subject is named.
pub fn cmd_hara_trace(elements: &[RawElement], cyber: &CyberConfig, results: Option<&ResultsData>, args: &[String]) -> i32 {
    let format = format_arg(args).unwrap_or("dot");
    if !is_graph_format(format) {
        eprintln!("Error: invalid value '{format}' for --format; valid values: {}", GRAPH_FORMATS.join(", "));
        return 1;
    }
    let mut subject: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--format" {
            i += 2;
            continue;
        }
        if args[i].starts_with("--") {
            i += 1;
            continue;
        }
        subject = Some(args[i].as_str());
        break;
    }
    let resolver = Resolver::new(elements);
    let verdict_of = verdicts(results);
    let (graph, issues) = match subject {
        None => vis::build_model_graph(DiagramKind::Traceability, elements, &resolver, cyber, &verdict_of),
        Some(key) => {
            let Some(elem) = resolver.resolve_ref(elements, key) else {
                eprintln!("Error: element '{key}' not found");
                return 1;
            };
            vis::derive::traceability::graph_with_verdicts(elem, elements, &resolver, &verdict_of)
        }
    };
    if let Some(i) = issues.first() {
        eprintln!("hara trace: {}: {}", i.code, i.message);
        return 1;
    }
    emit("hara trace", &graph, format)
}
