//! `REQ-TRS-FMED-001`: the FeatureModel diagram through the real walker and
//! validator. A `Diagram` of kind `FeatureModel` with a `FeatureDef` subject,
//! a package subject or the whole model draws the feature tree in FODA
//! notation; the cross-tree constraints are overlay edges that take no part in
//! layout; and every writer (Mermaid, PlantUML, static SVG) renders it.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::plantuml::render_plantuml;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::validate;
use syscribe_model::vis::derive::feature::feature_diagram;
use syscribe_model::vis::{build_graph, render_mermaid, render_svg, sprotty, EdgeKind, NodeKind};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-fm-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// A car product line: Engine is an alternative group (Petrol, Electric), Radio
/// an optional member with an or group (FM, DAB), Electric requires Charger and
/// Petrol excludes Electric.
fn model() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    write(&root, "Features/Car.md", "---\ntype: FeatureDef\nid: FEAT-CAR\nname: Car\nmandatory: true\n---\n");
    write(&root, "Features/Car/Engine.md", "---\ntype: FeatureDef\nid: FEAT-ENGINE\nname: Engine\nmandatory: true\ngroupKind: alternative\n---\n");
    write(&root, "Features/Car/Engine/Petrol.md", "---\ntype: FeatureDef\nid: FEAT-PETROL\nname: Petrol\nexcludes: [FEAT-ELECTRIC]\n---\n");
    write(&root, "Features/Car/Engine/Electric.md", "---\ntype: FeatureDef\nid: FEAT-ELECTRIC\nname: Electric\nrequires: [FEAT-CHARGER]\n---\n");
    write(&root, "Features/Car/Charger.md", "---\ntype: FeatureDef\nid: FEAT-CHARGER\nname: Charger\n---\n");
    write(&root, "Features/Car/Radio.md", "---\ntype: FeatureDef\nid: FEAT-RADIO\nname: Radio\ngroupKind: or\n---\n");
    write(&root, "Features/Car/Radio/FM.md", "---\ntype: FeatureDef\nid: FEAT-FM\nname: FM\n---\n");
    write(&root, "Features/Car/Radio/DAB.md", "---\ntype: FeatureDef\nid: FEAT-DAB\nname: DAB\n---\n");
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    write(&root, "Diagrams/CarFeatures.md", "---\ntype: Diagram\nname: CarFeatures\ndiagramKind: FeatureModel\nsubject: Features\n---\n\nThe car's features.\n");
    write(&root, "Diagrams/EngineFeatures.md", "---\ntype: Diagram\nname: EngineFeatures\ndiagramKind: FeatureModel\nsubject: Features::Car::Engine\n---\n");
    write(&root, "Parts/Engine.md", "---\ntype: PartDef\nname: Engine\n---\n");
    write(&root, "Diagrams/WrongSubject.md", "---\ntype: Diagram\nname: WrongSubject\ndiagramKind: FeatureModel\nsubject: Parts::Engine\n---\n");
    root
}

#[test]
fn a_feature_model_diagram_is_derived_and_validates_clean() {
    let root = model();
    let els = walk_model(&root).unwrap();
    let resolver = Resolver::new(&els);
    let d = els.iter().find(|e| e.qualified_name == "Diagrams::CarFeatures").unwrap();
    let (g, issues) = build_graph(d, &els, &resolver).expect("a FeatureModel diagram has an IR");
    assert!(issues.is_empty(), "{issues:?}");
    assert_eq!(g.nodes.len(), 8);
    assert!(g.nodes.iter().all(|n| n.kind == NodeKind::Feature));
    let count = |k: EdgeKind| g.edges.iter().filter(|e| e.kind == k).count();
    assert_eq!(count(EdgeKind::FeatureChild), 7);
    assert_eq!(count(EdgeKind::Requires), 1);
    assert_eq!(count(EdgeKind::Excludes), 1);
    let findings: Vec<_> = validate(&els).findings.into_iter().filter(|f| f.file.contains("Diagrams/CarFeatures")).collect();
    assert!(findings.is_empty(), "{:?}", findings.iter().map(|f| format!("{} {}", f.code, f.message)).collect::<Vec<_>>());
}

#[test]
fn a_feature_subject_draws_its_subtree_and_a_wrong_subject_is_w418() {
    let root = model();
    let els = walk_model(&root).unwrap();
    let resolver = Resolver::new(&els);
    let engine = els.iter().find(|e| e.qualified_name == "Diagrams::EngineFeatures").unwrap();
    let (g, _) = build_graph(engine, &els, &resolver).unwrap();
    let labels: Vec<&str> = g.nodes.iter().map(|n| n.label.as_str()).collect();
    assert_eq!(labels, vec!["Engine", "Electric", "Petrol"]);
    let wrong = els.iter().find(|e| e.qualified_name == "Diagrams::WrongSubject").unwrap();
    let (g, issues) = build_graph(wrong, &els, &resolver).unwrap();
    assert!(g.nodes.is_empty());
    assert_eq!(issues[0].code, "W418");
}

#[test]
fn the_sprotty_model_marks_features_and_flags_constraints_as_overlay() {
    let root = model();
    let els = walk_model(&root).unwrap();
    let json = serde_json::to_value(sprotty::to_sgraph(&feature_diagram(&els, None))).unwrap();
    assert_eq!(json["diagramKind"], "FeatureModel");
    assert_eq!(json["layoutOptions"]["elk.algorithm"], "mrtree", "a forest is laid out as trees");
    let kids = json["children"].as_array().unwrap();
    let engine = kids.iter().find(|c| c["ref"] == "Features::Car::Engine").unwrap();
    assert_eq!(engine["kind"], "feature");
    assert_eq!(engine["feature"]["mandatory"], true);
    assert_eq!(engine["feature"]["group"], "alternative");
    assert_eq!(engine["feature"]["childCount"], 2);
    assert_eq!(engine["feature"]["id"], "FEAT-ENGINE");
    let edges: Vec<_> = kids.iter().filter(|c| c["type"] == "edge").collect();
    let overlay: Vec<_> = edges.iter().filter(|e| e["overlay"] == true).map(|e| e["kind"].as_str().unwrap()).collect();
    assert_eq!(overlay.len(), 2);
    assert!(overlay.contains(&"requires") && overlay.contains(&"excludes"));
    assert!(edges.iter().filter(|e| e["kind"] == "child").all(|e| e.get("overlay").is_none()), "tree edges are laid out");
    assert!(edges.iter().filter(|e| e["overlay"] == true).all(|e| e.get("children").is_none()), "nothing would place an overlay edge's labels");
}

#[test]
fn every_writer_renders_the_feature_diagram() {
    let root = model();
    let els = walk_model(&root).unwrap();
    let d = els.iter().find(|e| e.qualified_name == "Diagrams::CarFeatures").unwrap();
    let resolver = Resolver::new(&els);
    let (g, _) = build_graph(d, &els, &resolver).unwrap();

    let mermaid = render_mermaid(&g, &|_| None).unwrap();
    assert!(mermaid.starts_with("flowchart TD"), "{mermaid}");
    assert!(mermaid.contains("-.->|requires|") || mermaid.contains("-.->"), "{mermaid}");

    let puml = render_plantuml(d, &els, None).expect("PlantUML maps the kind");
    assert!(puml.contains("<<mandatory>>") && puml.contains("<<optional>>") && puml.contains("<<xor>>") && puml.contains("<<or>>"), "{puml}");
    assert!(puml.contains(": requires") && puml.contains(": excludes"), "{puml}");

    let svg = render_svg(&g, &|_| None).unwrap();
    assert!(svg.contains("class=\"syscribe-diagram FeatureModel\""), "{svg}");
    assert!(svg.contains("class=\"feature-mark\""), "mandatory and optional marks");
    assert!(svg.contains("class=\"group-arc\""), "group wedges");
    assert_eq!(svg.matches("class=\"group-arc\"").count(), 2, "Engine (xor) and Radio (or)");
    for name in ["Car", "Engine", "Petrol", "Electric", "Charger", "Radio", "FM", "DAB"] {
        assert!(svg.contains(&format!(">{name}</text>")), "{name} is drawn");
    }
}

#[test]
fn the_whole_model_diagram_has_no_diagram_element_behind_it() {
    let root = model();
    let els = walk_model(&root).unwrap();
    let all = feature_diagram(&els, None);
    assert_eq!(all.nodes.len(), 8);
    let sub = feature_diagram(&els, Some("Features::Car::Radio"));
    assert_eq!(sub.nodes.len(), 3);
    let pkg = feature_diagram(&els, Some("Features"));
    assert_eq!(pkg.nodes.len(), 8, "a package qualified name covers every feature under it");
    let none = feature_diagram(&els, Some("Nowhere"));
    assert!(none.nodes.is_empty(), "nothing lies under a name that is neither a feature nor a package of features");
}
