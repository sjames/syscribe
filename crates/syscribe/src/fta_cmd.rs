//! `fault-tree analyze <FT> [--json] [--max-order N] [--no-ccf]` — minimal cut
//! sets, top-event probability and importance measures (GH #211).
//!
//! A thin renderer over [`syscribe_model::fta`]; all numbers come from the
//! library so the `metrics` roll-up and the diagram deriver agree with it.

use serde_json::json;
use syscribe_model::element::RawElement;
use syscribe_model::fta::{
    analyze_fault_tree, find_fault_tree, AnalysisOptions, EventOrigin, EventResult, FaultTreeAnalysis,
};
use syscribe_model::resolver::Resolver;

pub const USAGE: &str =
    "Usage: syscribe -m <model> fault-tree analyze <FaultTree-id> [--json] [--max-order N] [--no-ccf]";

fn sci(v: Option<f64>) -> String {
    match v {
        Some(x) if x == 0.0 => "0".to_string(),
        Some(x) => format!("{x:.3e}"),
        None => "n/a".to_string(),
    }
}

fn fix(v: Option<f64>) -> String {
    v.map_or_else(|| "n/a".to_string(), |x| format!("{x:.4}"))
}

pub const RENDER_USAGE: &str =
    "Usage: syscribe -m <model> fault-tree render <FaultTree-id> [--format mermaid|dot|svg|plantuml]";

/// `fault-tree render <FT> --format mermaid|dot|svg|plantuml` (GH #223): the
/// analysed fault tree as a diagram — gate symbols by `gateType`, event shapes
/// by `eventKind`, probabilities, single-point events and the top-event
/// probability — from the same Diagram IR the web view and `diagram export`
/// draw. Returns the process exit code.
pub fn cmd_fault_tree_render_diagram(elements: &[RawElement], ft_id: &str, format: &str) -> i32 {
    use syscribe_model::vis;
    const FORMATS: &[&str] = &["mermaid", "dot", "svg", "plantuml"];
    if !FORMATS.contains(&format) {
        eprintln!("Error: invalid value '{format}' for --format; valid values: {}", FORMATS.join(", "));
        return 1;
    }
    let Some(tree) = find_fault_tree(elements, ft_id) else {
        eprintln!("Error: no FaultTree element found with id or qualified name '{ft_id}'");
        return 1;
    };
    let resolver = Resolver::new(elements);
    let graph = vis::derive::fault_tree::tree_diagram(elements, &resolver, tree);
    if graph.nodes.is_empty() {
        eprintln!("Error: fault tree '{ft_id}' has no gates or events to draw");
        return 1;
    }
    let no_links = |_: &str| None;
    let text = match format {
        "dot" => Some(vis::render_dot(&graph)),
        "mermaid" => vis::render_mermaid(&graph, &no_links),
        "plantuml" => Some(syscribe_model::plantuml::render_safety_plantuml(&graph, "FaultTree")),
        _ => match vis::render_svg(&graph, &no_links) {
            Ok(svg) => Some(svg),
            Err(e) => {
                eprintln!("Error: fault tree '{ft_id}': {e}");
                return 1;
            }
        },
    };
    match text {
        Some(t) => {
            print!("{t}");
            0
        }
        None => 1,
    }
}

/// Parsed command line of `fault-tree analyze`.
pub struct AnalyzeArgs {
    pub tree: String,
    pub json: bool,
    pub opts: AnalysisOptions,
}

pub fn parse_args(rest: &[String]) -> Result<AnalyzeArgs, String> {
    let mut tree = None;
    let mut json = false;
    let mut opts = AnalysisOptions::default();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => json = true,
            "--no-ccf" => opts.ccf = false,
            "--max-order" => {
                i += 1;
                let v = rest.get(i).ok_or("--max-order needs a value")?;
                let n: usize = v.parse().map_err(|_| format!("--max-order '{v}' is not a positive integer"))?;
                if n == 0 {
                    return Err("--max-order must be ≥ 1".to_string());
                }
                opts.max_order = Some(n);
            }
            a if a.starts_with("--") => return Err(format!("unknown option '{a}'")),
            a => {
                if tree.replace(a.to_string()).is_some() {
                    return Err(format!("unexpected extra argument '{a}'"));
                }
            }
        }
        i += 1;
    }
    Ok(AnalyzeArgs { tree: tree.ok_or("missing <FaultTree-id>")?, json, opts })
}

/// Run the command; returns the process exit code.
pub fn cmd_fault_tree_analyze(elements: &[RawElement], args: &AnalyzeArgs) -> i32 {
    let Some(tree) = find_fault_tree(elements, &args.tree) else {
        eprintln!("Error: no FaultTree element found with id or qualified name '{}'", args.tree);
        return 1;
    };
    let resolver = Resolver::new(elements);
    let a = match analyze_fault_tree(elements, &resolver, tree, &args.opts) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: fault tree '{}': {e}", args.tree);
            return 1;
        }
    };
    if args.json {
        println!("{}", serde_json::to_string_pretty(&to_json(&a)).unwrap_or_default());
    } else {
        print_text(&a);
    }
    0
}

fn to_json(a: &FaultTreeAnalysis) -> serde_json::Value {
    json!({
        "tree": a.tree_id,
        "qualifiedName": a.tree_qname,
        "topEvent": a.top_event,
        "topNode": a.top_node,
        "missionTimeHours": a.mission_time_hours,
        "coherent": a.coherent,
        "truncated": a.truncated,
        "topProbability": a.top_probability,
        "rareEventProbability": a.rare_event_probability,
        "minCutUpperBound": a.min_cut_upper_bound,
        "cutSets": a.cut_sets,
        "events": a.events,
        "notes": a.notes,
    })
}

fn print_text(a: &FaultTreeAnalysis) {
    println!("# Fault tree analysis — {}\n", a.tree_id);
    println!("- Top node: `{}`{}", a.top_node, a.top_event.as_deref().map(|t| format!(" (topEvent: {t})")).unwrap_or_default());
    match a.mission_time_hours {
        Some(t) => println!("- Mission time: {t} h"),
        None => println!("- Mission time: not set"),
    }
    println!("- Minimal cut sets: {}{}", a.cut_sets.len(), if a.truncated { " (truncated by --max-order)" } else { "" });
    println!("- Top-event probability (exact, BDD): {}", sci(a.top_probability));
    println!("- Rare-event approximation: {}", sci(a.rare_event_probability));
    println!("- Min-cut upper bound: {}", sci(a.min_cut_upper_bound));
    if !a.coherent {
        println!("- Coherent: no (contains NOT/XOR)");
    }

    println!("\n## Minimal cut sets\n");
    if a.cut_sets.is_empty() {
        println!("None — the top event cannot occur (or is masked by house events).");
    } else {
        println!("| # | Order | Events | Probability |");
        println!("|---|---|---|---|");
        for (i, c) in a.cut_sets.iter().enumerate() {
            println!("| {} | {} | {} | {} |", i + 1, c.order, c.events.join(", "), sci(c.probability));
        }
    }

    println!("\n## Events\n");
    println!("| Event | Role | Min order | Probability | Fussell-Vesely | Birnbaum | RAW |");
    println!("|---|---|---|---|---|---|---|");
    let mut evs: Vec<&EventResult> = a.events.iter().collect();
    evs.sort_by(|x, y| {
        y.fussell_vesely
            .unwrap_or(-1.0)
            .partial_cmp(&x.fussell_vesely.unwrap_or(-1.0))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| x.id.cmp(&y.id))
    });
    for e in evs {
        let tag = match e.origin {
            EventOrigin::Ccf { .. } => " (CCF)",
            EventOrigin::Model => "",
        };
        println!(
            "| {}{} | {} | {} | {} | {} | {} | {} |",
            e.id,
            tag,
            serde_json::to_value(e.role).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default(),
            e.min_cut_order.map_or("—".to_string(), |o| o.to_string()),
            sci(e.probability),
            fix(e.fussell_vesely),
            sci(e.birnbaum),
            fix(e.raw),
        );
    }
    if !a.notes.is_empty() {
        println!();
        for n in &a.notes {
            println!("> {n}");
        }
    }
}
