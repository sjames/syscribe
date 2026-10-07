//! `REQ-TRS-SYSMLV2-060`..`-066`: named control steps, dangling successions, cross-entry
//! consistency, standard-library tables, compound units and the multiplicity advisory.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_yaml::Value;
use syscribe_model::element::RawElement;
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::units::unit_dimension;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-sysmlv2-fid-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn submodel(src: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/in.sysml", src);
    let els = walk_model(&r).unwrap();
    let bad: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "source did not parse: {bad:?}\n{src}");
    els
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("no {q}"))
}

fn native(body: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    write(&r, "B/Mission.md", body);
    walk_model(&r).unwrap()
}

fn names(v: &Option<Vec<Value>>) -> Vec<String> {
    v.as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|e| e.get("name").and_then(Value::as_str).unwrap_or("?").to_string())
        .collect()
}

fn ymls(v: &Option<Vec<Value>>) -> Value {
    Value::Sequence(v.clone().unwrap_or_default())
}

fn codes(els: &[RawElement]) -> Vec<String> {
    validate(els).findings.iter().map(|f| f.code.to_string()).collect()
}

// ── REQ-TRS-SYSMLV2-060 ─────────────────────────────────────────────────────

#[test]
fn single_statement_action_usage_ingests_as_a_named_step() {
    let els = submodel(
        "package P { action def A {\n\
         action loopA { for w in ws { action x; } }\n\
         if c { action y; }\n\
         action setIt { assign a := 1; }\n\
         action stop { terminate; }\n\
         action whileB { while k < 3 { action z; } }\n\
         action forever { loop { action q; } }\n\
         while m { action r; }\n\
         } }",
    );
    let a = find(&els, "Imp::P::A");
    let subs = a.frontmatter.sub_actions.as_ref().unwrap();
    assert_eq!(names(&a.frontmatter.sub_actions), vec!["loopA", "if_1", "setIt", "stop", "whileB", "forever", "while_1"]);
    let kind = |i: usize| subs[i].get("kind").and_then(Value::as_str).unwrap().to_string();
    assert_eq!((kind(0), kind(2), kind(3), kind(4), kind(5)), ("LoopAction".into(), "AssignmentAction".into(), "TerminateAction".into(), "LoopAction".into(), "LoopAction".into()));
    assert_eq!(subs[0].get("loopKind").and_then(Value::as_str), Some("for"));
    // Nested body of a named step is still ingested.
    assert_eq!(names(&Some(subs[0].get("body").and_then(Value::as_sequence).cloned().unwrap())), vec!["x"]);
}

#[test]
fn other_nested_action_usages_stay_perform_actions() {
    let els = submodel(
        "package P { action def A {\n\
         action two { action a; action b; }\n\
         action typed : T { while c { action z; } }\n\
         action plain;\n\
         action withDoc { doc /* d */ }\n\
         } }",
    );
    let a = find(&els, "Imp::P::A");
    let subs = a.frontmatter.sub_actions.as_ref().unwrap();
    assert_eq!(names(&a.frontmatter.sub_actions), vec!["two", "typed", "plain", "withDoc"]);
    assert!(subs.iter().all(|s| s.get("kind").and_then(Value::as_str) == Some("PerformAction")), "{subs:?}");
}

#[test]
fn a_named_step_does_not_advance_the_synthesized_counter() {
    let els = submodel("package P { action def A { if a { action x; } action mid { if b { action y; } } if c { action z; } } }");
    assert_eq!(names(&find(&els, "Imp::P::A").frontmatter.sub_actions), vec!["if_1", "mid", "if_2"]);
}

// ── REQ-TRS-SYSMLV2-061 / -063 ──────────────────────────────────────────────

const NAMED_MODEL: &str = "---\ntype: ActionDef\nname: Mission\nsubActions:\n\
    \x20 - name: navigate\n    kind: LoopAction\n    loopKind: for\n    variable: w\n    sequence: ws\n    body:\n      - name: tick\n        kind: PerformAction\n\
    \x20 - name: checkWeather\n    kind: IfAction\n    condition: \"windSpeed > 12.0\"\n    then:\n      - name: abort\n        kind: PerformAction\n\
    \x20 - name: if_1\n    kind: IfAction\n    condition: \"x\"\n\
    \x20 - name: setAlt\n    kind: AssignmentAction\n    target: alt\n    value: \"5\"\n\
    \x20 - name: halt\n    kind: TerminateAction\n    target: done\n\
    \x20 - name: spin\n    kind: LoopAction\n    loopKind: loop\n\
    \x20 - name: until_ground\n    kind: LoopAction\n    loopKind: repeat\n    condition: \"a\"\n\
successionConnections:\n  - after: navigate\n    before: checkWeather\n  - after: checkWeather\n    before: setAlt\n  - after: halt\n    before: until_ground\n---\n";

fn reimport(text: &str) -> Vec<RawElement> {
    submodel(text)
}

#[test]
fn named_control_entries_export_as_named_steps_and_read_back_identically() {
    let els = native(NAMED_MODEL);
    let text = export_sysml(&els, None).unwrap().text;
    for needle in ["action navigate {", "for w in ws {", "action checkWeather {", "if windSpeed > 12.0 {", "action setAlt {", "action halt {", "action spin {"] {
        assert!(text.contains(needle), "missing `{needle}`\n{text}");
    }
    // `if_1` is exactly the synthesized name where it sits only if no other `if_N` preceded it:
    // `checkWeather` is named, so the bare `if x {}` reads back as `if_1`.
    assert!(text.contains("\n        if x {") || text.contains("\n            if x {"), "{text}");
    assert!(text.contains("// subAction not exported (unknown loopKind 'repeat'): LoopAction until_ground"), "{text}");
    let back = reimport(&text);
    let orig = find(&els, "B::Mission");
    let got = find(&back, "Imp::B::Mission");
    let exported: Vec<Value> = orig
        .frontmatter
        .sub_actions
        .clone()
        .unwrap()
        .into_iter()
        .filter(|v| v.get("name").and_then(Value::as_str) != Some("until_ground"))
        .collect();
    assert_eq!(ymls(&got.frontmatter.sub_actions), Value::Sequence(exported), "{text}");
}

#[test]
fn export_is_stable_under_re_export_with_named_steps() {
    let els = native(NAMED_MODEL);
    let t1 = export_sysml(&els, None).unwrap().text;
    let t2 = export_sysml(&reimport(&t1), None).unwrap().text;
    let body = |t: &str| t.lines().filter(|l| !l.trim_start().starts_with("//") && !l.contains("package")).map(|l| l.trim().to_string()).collect::<Vec<_>>();
    let strip = |v: Vec<String>| v.into_iter().filter(|l| !l.is_empty() && l != "}" ).collect::<Vec<_>>();
    assert_eq!(strip(body(&t1)), strip(body(&t2)), "{t1}\n----\n{t2}");
}

// ── REQ-TRS-SYSMLV2-062 / -063 ──────────────────────────────────────────────

#[test]
fn succession_with_a_commented_endpoint_is_commented_too() {
    let els = native(
        "---\ntype: ActionDef\nname: Mission\nsubActions:\n\
         \x20 - name: a\n    kind: PerformAction\n\
         \x20 - name: b\n    kind: SendAction\n    payload: P\n    bogus: out\n\
         \x20 - name: c\n    kind: PerformAction\n\
         successionConnections:\n  - after: a\n    before: b\n  - after: b\n    before: c\n  - after: a\n    before: c\n  - after: a\n    before: elsewhere\n---\n",
    );
    let text = export_sysml(&els, None).unwrap().text;
    assert!(text.contains("// successionConnection not exported (endpoint 'b' was not exported): a -> b"), "{text}");
    assert!(text.contains("// successionConnection not exported (endpoint 'b' was not exported): b -> c"), "{text}");
    assert!(text.contains("first a then c;"), "{text}");
    // An endpoint that is not an entry of this body is not ours to judge.
    assert!(text.contains("first a then elsewhere;"), "{text}");
    assert!(!text.contains("first a then b;") && !text.contains("first b then c;"), "{text}");
}

/// Every `first X then Y;` an export writes names only steps the same exported body declares, and
/// re-ingesting the text yields only edges whose entry endpoints were themselves exported.
fn assert_exported_bodies_consistent(els: &[RawElement], text: &str) {
    let dropped: Vec<&str> = text
        .lines()
        .filter(|l| l.contains("// subAction not exported") || l.contains("// controlNode not exported"))
        .filter_map(|l| l.split_whitespace().last())
        .collect();
    for line in text.lines().map(str::trim).filter(|l| l.starts_with("first ")) {
        let mut it = line.trim_end_matches(';').split_whitespace().skip(1);
        let (a, _, b) = (it.next().unwrap(), it.next(), it.next().unwrap());
        for n in [a, b] {
            assert!(!dropped.contains(&n), "`{line}` references unexported `{n}`\n{text}");
        }
    }
    for e in reimport(text).iter().filter(|e| e.frontmatter.succession_connections.is_some()) {
        let mut known = names(&e.frontmatter.sub_actions);
        known.extend(names(&e.frontmatter.control_nodes));
        let orig_q = e.qualified_name.strip_prefix("Imp::").unwrap_or(&e.qualified_name);
        let Some(o) = els.iter().find(|o| o.qualified_name == orig_q) else { continue };
        let mut orig_known = names(&o.frontmatter.sub_actions);
        orig_known.extend(names(&o.frontmatter.control_nodes));
        for s in e.frontmatter.succession_connections.as_deref().unwrap_or(&[]) {
            for end in ["after", "before"] {
                let n = s.get(end).and_then(Value::as_str).unwrap().to_string();
                assert!(!orig_known.contains(&n) || known.contains(&n), "{orig_q}: endpoint `{n}` is an entry that was not exported\n{text}");
            }
        }
    }
}

#[test]
fn cross_entry_consistency_holds_for_a_degraded_body() {
    let els = native(
        "---\ntype: ActionDef\nname: Mission\nsubActions:\n\
         \x20 - name: a\n    kind: PerformAction\n\
         \x20 - name: b\n    kind: SendAction\n    payload: P\n    bogus: out\n\
         \x20 - name: c\n    kind: PerformAction\n\
         successionConnections:\n  - after: a\n    before: b\n  - after: b\n    before: c\n  - after: a\n    before: c\n---\n",
    );
    let text = export_sysml(&els, None).unwrap().text;
    assert_exported_bodies_consistent(&els, &text);
    let got = find(&reimport(&text), "Imp::B::Mission").clone();
    assert_eq!(got.frontmatter.succession_connections.as_ref().map(Vec::len), Some(1));
}

#[test]
fn cross_entry_consistency_holds_for_the_repository_model() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model");
    let els = walk_model(&root).unwrap();
    let text = export_sysml(&els, Some("Behavior")).unwrap().text;
    assert_exported_bodies_consistent(&els, &text);
    assert!(text.contains("action navigateWaypoints {"), "named loop must export\n{text}");
    assert!(text.contains("action checkWeather {"), "named if must export\n{text}");
    for line in text.lines().filter(|l| l.trim_start().starts_with("first ")) {
        // Both endpoints of every written succession are written steps of the file.
        let t = line.trim().trim_end_matches(';');
        let mut it = t.split_whitespace();
        let (_, a, _, b) = (it.next(), it.next().unwrap(), it.next(), it.next().unwrap());
        for n in [a, b] {
            assert!(
                !text.lines().any(|l| l.contains("// subAction not exported") && l.trim_end().ends_with(&format!(" {n}"))),
                "succession `{line}` references unexported `{n}`"
            );
        }
    }
}

// ── REQ-TRS-SYSMLV2-064 ─────────────────────────────────────────────────────

#[test]
fn scalar_values_members_raise_no_unknown_member_finding() {
    let els = submodel(
        "package P { part def M { doc /* d */\n\
         attribute a : ScalarValues::Real; attribute b : ScalarValues::Integer; attribute c : ScalarValues::Boolean;\n\
         attribute d : ScalarValues::String; attribute e : ScalarValues::Natural; attribute f : ScalarValues::Rational;\n\
         attribute g : ScalarValues::Complex; attribute h : ScalarValues::Number; attribute i : ScalarValues::NumericalValue;\n\
         attribute j : ScalarValues::ScalarValue; attribute k : ISQ::TorqueValue; attribute l : Real; } }",
    );
    let c = codes(&els);
    for bad in ["E111", "W043", "W404"] {
        assert!(!c.iter().any(|x| x == bad), "{bad} in {c:?}");
    }
    let text = export_sysml(&els, None).unwrap().text;
    assert!(text.contains("attribute g : ScalarValues::Complex;") && text.contains("attribute k : ISQ::TorqueValue;"), "{text}");
    assert!(!text.contains("not exported") && !text.contains("skipped:"), "{text}");
    // A real typo is still caught.
    let typo = submodel("package P { part def M { doc /* d */\n attribute a : ScalarValues::Flota; } }");
    assert!(codes(&typo).iter().any(|x| x == "W043"));
}

#[test]
fn compound_unit_dimensions_derive_from_the_table() {
    let d = |s: &str| unit_dimension(s).map(|d| d.human());
    assert_eq!(d("N*m"), d("joule"));
    assert_eq!(d("m/s"), d("metrePerSecond"));
    assert_eq!(d("kg*m/s^2"), d("newton"));
    assert_eq!(d("m^2").as_deref(), Some("L^2"));
    assert_eq!(d("1/s"), d("hertz"));
    assert_eq!(d("rpm"), d("hertz"));
    assert_eq!(d("SI::m/SI::s"), d("m/s"));
    assert_eq!(d("m/zz"), None);
    assert_eq!(d("m/"), None);
    assert_eq!(d("bogus"), None);
}

#[test]
fn compound_unit_is_checked_against_the_quantity_type() {
    let ok = native(
        "---\ntype: PartDef\nname: Mission\nfeatures:\n  - name: t\n    typedBy: ISQ::TorqueValue\n    unit: N*m\n---\n\nd\n",
    );
    assert!(!codes(&ok).iter().any(|c| c == "W044"), "{:?}", codes(&ok));
    let bad = native(
        "---\ntype: PartDef\nname: Mission\nfeatures:\n  - name: t\n    typedBy: ISQ::MassValue\n    unit: m/s\n---\n\nd\n",
    );
    assert!(codes(&bad).iter().any(|c| c == "W044"), "{:?}", codes(&bad));
}

// ── REQ-TRS-SYSMLV2-065 ─────────────────────────────────────────────────────

#[test]
fn compound_units_ingest_with_or_without_spaces_and_export_as_expressions() {
    let els = submodel(
        "package P { part def M { doc /* d */\n\
         attribute a : ISQ::TorqueValue = 4 [N * m];\n\
         attribute b : ISQ::TorqueValue = 4 [N*m];\n\
         attribute c : ISQ::SpeedValue = 3 [m / s];\n\
         attribute d : ISQ::AreaValue = 2 [m^2];\n\
         attribute e : ISQ::MassValue = 5 [kg];\n\
         part x : M [2]; } }",
    );
    let unit = |n: &str| find(&els, &format!("Imp::P::M::{n}")).frontmatter.unit.clone();
    assert_eq!(unit("a").as_deref(), Some("N*m"));
    assert_eq!(unit("b").as_deref(), Some("N*m"));
    assert_eq!(unit("c").as_deref(), Some("m/s"));
    assert_eq!(unit("d").as_deref(), Some("m^2"));
    assert_eq!(unit("e").as_deref(), Some("kg"));
    // A real multiplicity is untouched.
    assert_eq!(find(&els, "Imp::P::M::x").frontmatter.multiplicity.as_deref(), Some("2"));
    let text = export_sysml(&els, None).unwrap().text;
    assert!(text.contains("= 4 [N*m];") && text.contains("= 3 [m/s];") && text.contains("= 2 [m^2];"), "{text}");
    assert!(!text.contains("['"), "units must not be quoted names\n{text}");
    let back = submodel(&text);
    for n in ["a", "b", "c", "d", "e"] {
        assert_eq!(
            find(&back, &format!("Imp::Imp::P::M::{n}")).frontmatter.unit,
            find(&els, &format!("Imp::P::M::{n}")).frontmatter.unit,
            "{n}\n{text}"
        );
    }
}

// ── REQ-TRS-SYSMLV2-066 ─────────────────────────────────────────────────────

#[test]
fn reversed_or_non_natural_multiplicity_raises_w544() {
    let els = submodel(
        "package P { part def M { doc /* d */\n\
         part a : M [3..1];\n\
         part b : M [0..*];\n\
         part c : M [2];\n\
         part d : M [n];\n\
         part e : M [0..n];\n\
         part f : M [1..4]; } }",
    );
    let w: Vec<String> = validate(&els).findings.iter().filter(|f| f.code == "W544").map(|f| f.message.clone()).collect();
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("Imp::P::M::a") && w[0].contains("3..1"), "{w:?}");
}

#[test]
fn negative_or_fractional_multiplicity_bound_raises_w544() {
    let els = submodel("package P { part def M { doc /* d */\n part a : M [-1..2];\n part b : M [1.5..3]; } }");
    let n = validate(&els).findings.iter().filter(|f| f.code == "W544").count();
    assert_eq!(n, 2);
}
