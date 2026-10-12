//! `REQ-TRS-VIS-031` (GH #338): diagram views. The mapping form of `include:` (`kinds`, `domains`, `tags`,
//! `features`, `depth`, `members`) says what an IBD is about and fades the rest; a named style
//! (`[diagram.style.<name>]`, selected by `style:`) restyles flow kinds and, with `context = "hide"`, leaves the rest
//! out; `layout:` takes the hints `flow` and `layers` beside its pins; and two products' graphs merge into a variant
//! view that marks what only one has.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::resolver::Resolver;
use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::{validate, validate_with_config, Finding};
use syscribe_model::vis::flow::{apply, Context, DiagramConfig};
use syscribe_model::vis::ir::{FlowKind, LayoutDirection};
use syscribe_model::vis::{build_graph, DiagramGraph, Tone};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-views-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// An inverter: a DC input and a power stage joined by power, a sensor and a protection board joined by protection
/// to the power stage, and firmware (software, tagged `control`) on a board, gated by the `Afci` feature.
fn model(diagram_fm: &str) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    write(&root, "Features/Afci.md", "---\ntype: FeatureDef\nid: FEAT-AFCI\nname: Afci\n---\n");
    write(&root, "Inv/_index.md", "---\ntype: Package\nname: Inv\n---\n");
    write(&root, "Inv/Dc.md", "---\ntype: PartDef\nname: Dc\ndomain: hardware\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\n");
    write(&root, "Inv/Stage.md", "---\ntype: PartDef\nname: Stage\ndomain: hardware\nfeatures:\n  - {name: dcIn, type: Port, direction: in}\n  - {name: trip, type: Port, direction: in}\n---\n");
    write(&root, "Inv/Sensor.md", "---\ntype: PartDef\nname: Sensor\ndomain: hardware\nfeatures:\n  - {name: trip, type: Port, direction: out}\n---\n");
    write(&root, "Inv/Fw.md", "---\ntype: PartDef\nname: Fw\ndomain: software\ntags: [control]\nappliesWhen: Features::Afci\n---\n");
    write(
        &root,
        "Inv/Inverter.md",
        "---\ntype: PartDef\nname: Inverter\ndomain: system\nfeatures:\n  - {name: dc, typedBy: Inv::Dc}\n  - {name: stage, typedBy: Inv::Stage}\n  - {name: sensor, typedBy: Inv::Sensor}\n  - {name: fw, typedBy: Inv::Fw}\nconnections:\n  - {from: dc.out, to: stage.dcIn, flowKind: power}\n  - {from: sensor.trip, to: stage.trip, flowKind: protection}\n---\n",
    );
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    write(&root, "Diagrams/View.md", &format!("---\ntype: Diagram\nname: View\ndiagramKind: IBD\nsubject: Inv::Inverter\n{diagram_fm}---\n\nView.\n"));
    root
}

fn graph(root: &Path) -> DiagramGraph {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::View").unwrap();
    build_graph(d, &elements, &resolver).unwrap().0
}

fn faded(g: &DiagramGraph, id: &str) -> bool {
    g.node(id).unwrap_or_else(|| panic!("no node {id}")).faded
}

fn hits(root: &Path, code: &str) -> Vec<Finding> {
    let cfg = ValidateConfig { model_root: Some(root.to_path_buf()), ..Default::default() };
    validate_with_config(&walk_model(root).unwrap(), &cfg).findings.into_iter().filter(|f| f.code == code).collect()
}

#[test]
fn a_kinds_view_fades_the_parts_and_edges_it_does_not_include() {
    let g = graph(&model("include: {kinds: [protection]}\n"));
    assert!(!faded(&g, "s-inv-inverter-sensor") && !faded(&g, "s-inv-inverter-stage"));
    assert!(faded(&g, "s-inv-inverter-dc") && faded(&g, "s-inv-inverter-fw"));
    assert!(faded(&g, "s-inv-inverter-dc-out"), "a faded part's ports fade with it");
    let edge = |f: FlowKind| g.edges.iter().find(|e| e.flow == Some(f)).unwrap().faded;
    assert!(!edge(FlowKind::Protection));
    assert!(edge(FlowKind::Power), "the power edge is outside a protection view");
    assert!(!faded(&g, "s-inv-inverter"), "the boundary is never faded");
}

#[test]
fn domains_tags_and_features_select_parts() {
    let ids = |fm: &str| {
        let g = graph(&model(fm));
        g.nodes.iter().filter(|n| n.kind == syscribe_model::vis::NodeKind::Block && !n.faded).map(|n| n.id.clone()).collect::<Vec<_>>()
    };
    assert_eq!(ids("include: {domains: [software]}\n"), vec!["s-inv-inverter-fw"]);
    assert_eq!(ids("include: {tags: [control]}\n"), vec!["s-inv-inverter-fw"]);
    assert_eq!(ids("include: {features: [FEAT-AFCI]}\n"), vec!["s-inv-inverter-fw"], "by id");
    assert_eq!(ids("include: {features: [Features::Afci]}\n"), vec!["s-inv-inverter-fw"], "by qname");
    assert_eq!(ids("include: {domains: [hardware], kinds: [power]}\n"), vec!["s-inv-inverter-dc", "s-inv-inverter-stage"], "every key must hold");
    // the list form is unchanged: a member filter, nothing faded
    let g = graph(&model("include: [dc, stage]\n"));
    assert!(g.nodes.iter().all(|n| !n.faded));
    assert!(g.node("s-inv-inverter-sensor").is_none());
    // members in the mapping form filter as the list does
    assert!(graph(&model("include: {members: [dc], domains: [hardware]}\n")).node("s-inv-inverter-stage").is_none());
}

#[test]
fn a_named_style_hides_the_context_and_restyles_a_flow_kind() {
    let cfg = DiagramConfig::from_toml_str(
        "[diagram.styles.protection]\nwidth = 2\n[diagram.style.focus]\ncontext = \"fade\"\n[diagram.style.safety]\nextends = \"focus\"\n[diagram.style.safety.flows.protection]\nwidth = 5\nstroke = \"#cc0000\"\n[diagram.style.isolate]\ncontext = \"hide\"\n",
    );
    assert!(cfg.problems.is_empty(), "{:?}", cfg.problems);
    let safety = cfg.resolve(Some("safety"));
    assert_eq!(safety.context, Context::Fade);
    let p = &safety.flows[&FlowKind::Protection];
    assert_eq!((p.width, p.stroke.as_deref()), (Some(5.0), Some("#cc0000")), "the style wins over the project's look");
    assert_eq!(cfg.resolve(None).flows[&FlowKind::Protection].width, Some(2.0), "no style: the project's look");

    let mut g = graph(&model("include: {kinds: [protection]}\nstyle: isolate\n"));
    assert_eq!(g.style.as_deref(), Some("isolate"));
    apply(&mut g, &cfg);
    assert!(g.node("s-inv-inverter-dc").is_none() && g.node("s-inv-inverter-dc-out").is_none(), "hidden with its ports");
    assert!(g.edges.iter().all(|e| e.flow == Some(FlowKind::Protection)));
    // switched by the one field: the same view under `safety` keeps the context, faded
    let mut g = graph(&model("include: {kinds: [protection]}\nstyle: safety\n"));
    apply(&mut g, &cfg);
    assert!(faded(&g, "s-inv-inverter-dc"));
    assert_eq!(g.flow_styles[&FlowKind::Protection].width, Some(5.0));
}

#[test]
fn style_problems_are_w419_and_w421() {
    let cfg = DiagramConfig::from_toml_str("[diagram.style.a]\nextends = \"b\"\n[diagram.style.b]\nextends = \"a\"\n[diagram.style.c]\ncontext = \"blur\"\nextends = \"nope\"\n");
    assert_eq!(cfg.problems.len(), 3, "{:?}", cfg.problems);
    let root = model("style: nosuch\n");
    write(&root, ".syscribe.toml", "[diagram.style.focus]\ncontext = \"fade\"\n");
    let f = hits(&root, "W421");
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].message.contains("'nosuch'") && f[0].message.contains("default, focus"), "{}", f[0].message);
}

#[test]
fn view_problems_are_w421() {
    let root = model("include: {kinds: [power, steam], domains: [firmware], features: [Features::Nope], colour: red}\n");
    let f: Vec<String> = hits(&root, "W421").into_iter().map(|f| f.message).collect();
    assert_eq!(f.len(), 4, "{f:?}");
    assert!(f.iter().any(|m| m.contains("'steam'")) && f.iter().any(|m| m.contains("'firmware'")));
    assert!(f.iter().any(|m| m.contains("Features::Nope")) && f.iter().any(|m| m.contains("'colour'")));
    // on a BDD, and `style:` off a Diagram
    let root = model("");
    write(&root, "Diagrams/Bdd.md", "---\ntype: Diagram\nname: Bdd\ndiagramKind: BDD\nsubject: Inv::Inverter\ninclude: {domains: [hardware]}\n---\n\nBDD.\n");
    write(&root, "Inv/Dc.md", "---\ntype: PartDef\nname: Dc\nstyle: focus\n---\n");
    assert_eq!(hits(&root, "W421").len(), 2);
    assert!(validate(&walk_model(&root).unwrap()).findings.iter().all(|f| f.code != "W047"), "include mapping and style are known fields");
}

#[test]
fn layout_hints_set_the_flow_and_order_the_bands() {
    let g = graph(&model("layout: {flow: top-to-bottom, layers: [protection, power]}\n"));
    assert_eq!(g.layout_hints.direction, LayoutDirection::Down);
    let blocks: Vec<&str> = g.nodes.iter().filter(|n| n.kind == syscribe_model::vis::NodeKind::Block).map(|n| n.id.as_str()).collect();
    // sensor and stage carry protection (first band), dc power (second), fw nothing (last)
    assert_eq!(blocks, vec!["s-inv-inverter-stage", "s-inv-inverter-sensor", "s-inv-inverter-dc", "s-inv-inverter-fw"]);
    let root = model("layout: {flow: sideways, layers: [ghost]}\n");
    assert_eq!(hits(&root, "W421").len(), 2);
    assert!(hits(&root, "W416").is_empty(), "hints are not pins");
    // a pin beside the hints still pins
    let g = graph(&model("layout: {flow: left-to-right, s-inv-inverter-dc: {x: 10, y: 20}}\n"));
    assert_eq!(g.node("s-inv-inverter-dc").unwrap().pin.as_ref().map(|p| (p.x, p.y)), Some((10.0, 20.0)));
}

#[test]
fn the_variant_view_marks_what_only_one_product_has() {
    let root = model("");
    let base = graph(&root);
    let mut without = base.clone();
    without.nodes.retain(|n| !n.id.starts_with("s-inv-inverter-fw"));
    let mut extra = base.clone();
    extra.edges.clear();
    let g = syscribe_model::vis::variant::diff(base.clone(), without, "CONF-A", "CONF-B");
    let fw = g.node("s-inv-inverter-fw").unwrap().mark.clone().expect("fw only in A");
    // the firmware's own `optional` badge (unsettled here) stays; the variant badge is added after it
    assert_eq!((fw.badges.last().map(String::as_str), fw.tone), (Some("only CONF-A"), Tone::Bad), "{:?}", fw.badges);
    assert!(g.node("s-inv-inverter-dc").unwrap().mark.is_none(), "in both: unmarked");
    let g = syscribe_model::vis::variant::diff(extra, base, "CONF-A", "CONF-B");
    assert!(g.edges.iter().all(|e| e.label.as_deref().is_some_and(|l| l.ends_with("[only CONF-B]"))), "{:?}", g.edges);
}

#[test]
fn writers_draw_faded_elements() {
    let g = graph(&model("include: {kinds: [protection]}\n"));
    let svg = syscribe_model::vis::render_svg(&g, &|_| None).unwrap();
    assert!(svg.contains("class=\"block Part faded\"") && svg.contains("opacity=\"0.25\""), "svg");
    let mmd = syscribe_model::vis::render_mermaid(&g, &|_| None).unwrap();
    assert!(mmd.contains("classDef faded opacity:0.25") && mmd.contains("opacity:0.25"), "{mmd}");
    let dot = syscribe_model::vis::render_dot(&g);
    assert!(dot.contains("40\""), "{dot}");
    let json = syscribe_model::vis::sprotty::to_sgraph_json(&g).to_string();
    assert!(json.contains("\"cssClasses\":[\"faded\"]"), "sprotty");
}

/// The laid-out top edge of a block.
fn top(g: &DiagramGraph, id: &str) -> f64 {
    let sizes = syscribe_model::vis::size::size_graph_default(g);
    syscribe_model::vis::layout(g, &sizes).unwrap().nodes[id].y
}

#[test]
fn layers_order_the_laid_out_bands() {
    // left to right, dc and sensor share the first layer: the band order decides which is on top
    let g = graph(&model("layout: {flow: left-to-right, layers: [protection, power]}\n"));
    assert!(top(&g, "s-inv-inverter-sensor") < top(&g, "s-inv-inverter-dc"), "protection band first");
    let g = graph(&model("layout: {flow: left-to-right, layers: [power, protection]}\n"));
    assert!(top(&g, "s-inv-inverter-dc") < top(&g, "s-inv-inverter-sensor"), "power band first");
    assert!(g.layout_hints.model_order);
    let opts = syscribe_model::vis::sprotty::to_sgraph_json(&g)["layoutOptions"].clone();
    assert_eq!(opts["elk.layered.crossingMinimization.forceNodeModelOrder"], "true", "the browser keeps the order too: {opts}");
}

#[test]
fn ignored_entries_select_nothing_away_and_malformed_values_are_w421() {
    // every kind ignored: the key is ignored as a whole, nothing fades
    let root = model("include: {kinds: [steam]}\n");
    assert!(graph(&root).nodes.iter().all(|n| !n.faded));
    assert_eq!(hits(&root, "W421").len(), 1);
    // a scalar where a list belongs is W421, not a parse error
    let root = model("include: {kinds: power, depth: deep}\n");
    let f: Vec<String> = hits(&root, "W421").into_iter().map(|f| f.message).collect();
    assert!(f.iter().any(|m| m.contains("`include.kinds` must be a list")) && f.iter().any(|m| m.contains("`include.depth` must be a whole number")), "{f:?}");
    assert!(hits(&root, "E002").is_empty());
    // the list form keeps its own message
    let e = hits(&model("include: dc\n"), "E002");
    assert!(e.iter().any(|f| f.message.contains("expected a sequence")), "{e:?}");
    // the mapping form on a manifest diagram
    let root = model("");
    write(&root, "Diagrams/Hand.md", "---\ntype: Diagram\nname: Hand\ndiagramKind: IBD\nshapes:\n  dc: {ref: Inv::Dc, kind: PartDef}\ninclude: {kinds: [power]}\n---\n\nHand.\n");
    assert!(hits(&root, "W421").iter().any(|f| f.message.contains("derived IBD")));
}

#[test]
fn a_negated_feature_does_not_select_a_part() {
    let root = model("include: {features: [FEAT-AFCI]}\n");
    write(&root, "Inv/Fw.md", "---\ntype: PartDef\nname: Fw\ndomain: software\nappliesWhen: \"not Features::Afci\"\n---\n");
    assert!(faded(&graph(&root), "s-inv-inverter-fw"));
}

#[test]
fn only_a_style_on_a_cycle_is_blamed_and_default_may_be_extended() {
    let cfg = DiagramConfig::from_toml_str("[diagram.style.a]\nextends = \"b\"\n[diagram.style.b]\nextends = \"c\"\n[diagram.style.c]\nextends = \"b\"\n[diagram.style.d]\nextends = \"default\"\n");
    assert_eq!(cfg.problems.len(), 1, "{:?}", cfg.problems);
    assert!(cfg.problems[0].contains("[diagram.style.b]"), "{:?}", cfg.problems);
}

#[test]
fn writers_fade_frames_and_follow_the_flow() {
    let root = model("include: {kinds: [protection]}\ngroupBy: domain\nlayout: {flow: top-to-bottom}\n");
    let g = graph(&root);
    // the software frame holds only the faded firmware
    let frame = g.nodes.iter().find(|n| n.kind == syscribe_model::vis::NodeKind::Swimlane && n.faded).expect("a faded frame");
    let dot = syscribe_model::vis::render_dot(&g);
    let line = dot.lines().skip_while(|l| !l.contains(&format!("cluster_{}", frame.id.replace('-', "_")))).nth(1).unwrap_or_else(|| panic!("{dot}"));
    assert!(line.contains("40\"") && line.contains("fontcolor"), "{line}");
    let elements = walk_model(&root).unwrap();
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::View").unwrap();
    let puml = syscribe_model::plantuml::render_plantuml(d, &elements, None).unwrap();
    assert!(puml.contains("#f7f7f7;line:bbbbbb") && puml.contains("-[#bbbbbb]->") && puml.contains("top to bottom direction"), "{puml}");
    let mmd = syscribe_model::vis::render_mermaid(&g, &|_| None).unwrap();
    assert!(mmd.starts_with("flowchart TD"), "{mmd}");
}

#[test]
fn the_variant_view_marks_the_outermost_part_and_quotes_its_labels() {
    let root = model("");
    let base = graph(&root);
    let mut without = base.clone();
    without.nodes.retain(|n| !n.id.starts_with("s-inv-inverter-dc"));
    without.edges.retain(|e| !e.source.starts_with("s-inv-inverter-dc"));
    let g = syscribe_model::vis::variant::diff(base, without, "CONF-A", "CONF-B");
    assert!(g.node("s-inv-inverter-dc").unwrap().mark.is_some());
    assert!(g.node("s-inv-inverter-dc-out").unwrap().mark.is_none(), "its port goes with it, unmarked");
    let mmd = syscribe_model::vis::render_mermaid(&g, &|_| None).unwrap();
    assert!(mmd.contains("|\"connection [only CONF-A]\"|") || mmd.contains("[only CONF-A]\"|"), "{mmd}");
}
