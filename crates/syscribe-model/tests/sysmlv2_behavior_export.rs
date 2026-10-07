//! `REQ-TRS-SYSMLV2-056`..`-058` (`ADR-SYS-SYSMLV2-002` addendum): action and state bodies
//! export as SysML v2 text that the existing ingestion reads back identically; anything it
//! would not read back is a comment, never an approximation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_yaml::Value;
use syscribe_model::element::RawElement;
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-behav-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn root_with(files: &[(&str, &str)]) -> PathBuf {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    for (p, c) in files {
        write(&r, p, c);
    }
    r
}

/// Export `elements`, drop the text into a `sysmlSubmodel` package and walk it again.
fn reimport(text: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/out.sysml", text);
    let els = walk_model(&r).unwrap();
    let bad: Vec<_> = els
        .iter()
        .flat_map(|e| e.derive_findings.iter())
        .filter(|(c, _, _)| c == "W541")
        .collect();
    assert!(bad.is_empty(), "parse-back failed: {bad:?}\n{text}");
    els
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("no {q}"))
}

fn yaml(v: &Option<Vec<Value>>) -> Value {
    Value::Sequence(v.clone().unwrap_or_default())
}

/// Ingest `.sysml` source into a submodel package `Imp` and return the walked elements.
fn ingest(src: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/in.sysml", src);
    let els = walk_model(&r).unwrap();
    let bad: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "source did not parse: {bad:?}");
    els
}

/// Export `els`, re-ingest the text and return the new elements (qnames gain a second `Imp::`).
fn export_and_back(els: &[RawElement]) -> (String, Vec<RawElement>) {
    let text = export_sysml(els, None).unwrap().text;
    let back = reimport(&text);
    (text, back)
}

const ACTIONS: &str = "package P {\n\
    action def Takeoff;\n\
    action def Mission {\n\
        action takeoff : Takeoff;\n\
        accept cmd : StartCmd;\n\
        send ack : AckCmd;\n\
        assign alt := 10;\n\
        while alt < 100 { action climb; assign alt := alt + 1; }\n\
        if windSpeed > 12.0 { action abort; } else { action carryOn; }\n\
        for i in 1..3 { action tick; }\n\
        loop { action spin; terminate; }\n\
        terminate done;\n\
        fork f1;\n\
        join j1;\n\
        decide d1;\n\
        merge m1;\n\
        first takeoff then f1;\n\
    }\n\
}\n";

fn assert_action_fields_equal(a: &RawElement, b: &RawElement, text: &str) {
    assert_eq!(yaml(&a.frontmatter.sub_actions), yaml(&b.frontmatter.sub_actions), "subActions\n{text}");
    assert_eq!(yaml(&a.frontmatter.control_nodes), yaml(&b.frontmatter.control_nodes), "controlNodes\n{text}");
    assert_eq!(
        yaml(&a.frontmatter.succession_connections),
        yaml(&b.frontmatter.succession_connections),
        "successionConnections\n{text}"
    );
}

#[test]
fn action_def_body_round_trips_through_ingestion() {
    let els = ingest(ACTIONS);
    let orig = find(&els, "Imp::P::Mission");
    assert!(orig.frontmatter.sub_actions.as_ref().is_some_and(|s| s.len() >= 8), "{:?}", orig.frontmatter.sub_actions);
    let (text, back) = export_and_back(&els);
    assert!(!text.contains("not exported"), "{text}");
    for needle in [
        "action takeoff : Takeoff;",
        "accept cmd : StartCmd;",
        "send ack : AckCmd;",
        "assign alt := 10;",
        "while alt < 100 {",
        "if windSpeed > 12.0 {",
        "} else {",
        "for i in 1..3 {",
        "loop {",
        "terminate done;",
        "fork f1;",
        "join j1;",
        "decide d1;",
        "merge m1;",
        "first takeoff then f1;",
    ] {
        assert!(text.contains(needle), "missing `{needle}`\n{text}");
    }
    assert_action_fields_equal(orig, find(&back, "Imp::Imp::P::Mission"), &text);
}

/// One action-body statement list: ingest, export, check the exact statement text appears, nothing
/// is degraded to a comment, and the re-ingested element has equal native fields.
fn action_case(body: &str, needles: &[&str]) {
    let els = ingest(&format!("package P {{ action def X {{\n{body}\n}} }}"));
    let orig = find(&els, "Imp::P::X");
    let (text, back) = export_and_back(&els);
    assert!(!text.contains("not exported"), "{text}");
    for n in needles {
        assert!(text.contains(n), "missing `{n}`\n{text}");
    }
    assert_action_fields_equal(orig, find(&back, "Imp::Imp::P::X"), &text);
}

#[test]
fn perform_sub_actions_round_trip() {
    action_case("action a : T;\naction b;", &["action a : T;", "action b;"]);
}

#[test]
fn accept_and_send_round_trip() {
    action_case("accept cmd : StartCmd;\nsend ack : AckCmd;", &["accept cmd : StartCmd;", "send ack : AckCmd;"]);
}

#[test]
fn assign_round_trips() {
    action_case("assign x := 1;\nassign y := x + 2;", &["assign x := 1;", "assign y := x + 2;"]);
}

#[test]
fn while_loop_and_loop_round_trip() {
    action_case(
        "while x < 3 { action a; }\nloop { action b; }\nwhile y { assign z := 0; }",
        &["while x < 3 {", "loop {", "while y {"],
    );
}

#[test]
fn for_loop_round_trips() {
    action_case("for i in 1..3 { action a; }", &["for i in 1..3 {"]);
}

#[test]
fn if_else_round_trips_including_nested() {
    action_case(
        "if a > 1 { if b { action deep; } else { assign q := 2; } } else { terminate; }\nif c { action t; }",
        &["if a > 1 {", "if b {", "} else {", "terminate;", "if c {"],
    );
}

#[test]
fn terminate_round_trips() {
    action_case("terminate;\nterminate stop;", &["terminate;", "terminate stop;"]);
}

#[test]
fn control_nodes_and_successions_round_trip() {
    action_case(
        "action s;\nfork f;\njoin j;\ndecide d;\nmerge m;\nfirst s then f;\nfirst f then j;",
        &["fork f;", "join j;", "decide d;", "merge m;", "first s then f;", "first f then j;"],
    );
}

#[test]
fn action_usage_body_round_trips() {
    let els = ingest("package P { action def T; action run : T { action step; if go { assign v := 1; } first step then end2; fork end2; } }");
    let orig = find(&els, "Imp::P::run");
    let (text, back) = export_and_back(&els);
    assert!(!text.contains("not exported"), "{text}");
    assert_action_fields_equal(orig, find(&back, "Imp::Imp::P::run"), &text);
}

const STATES: &str = "package P {\n\
    state def Flight {\n\
        entry action boot;\n\
        do action monitor;\n\
        exit action halt;\n\
        state idle {\n\
            entry action prep;\n\
            transition first idle accept Go then flying;\n\
        }\n\
        state flying {\n\
            state cruise;\n\
            state landed;\n\
            then cruise;\n\
            final landed;\n\
            transition first flying accept Land if fuel > 0 do action report : Report then idle;\n\
        }\n\
        then idle;\n\
        transition first idle accept Abort then flying;\n\
    }\n\
}\n";

fn assert_state_fields_equal(a: &RawElement, b: &RawElement, text: &str) {
    let f = |e: &RawElement| {
        (
            yaml(&e.frontmatter.sub_states),
            yaml(&e.frontmatter.transitions),
            e.frontmatter.entry_action.clone(),
            e.frontmatter.do_action.clone(),
            e.frontmatter.exit_action.clone(),
        )
    };
    assert_eq!(f(a), f(b), "state fields differ\n{text}");
}

#[test]
fn state_def_body_round_trips_through_ingestion() {
    let els = ingest(STATES);
    let orig = find(&els, "Imp::P::Flight");
    assert!(orig.frontmatter.sub_states.as_ref().is_some_and(|s| s.len() == 2));
    let (text, back) = export_and_back(&els);
    assert!(!text.contains("not exported"), "{text}");
    for n in [
        "entry action boot;",
        "do action monitor;",
        "exit action halt;",
        "state idle {",
        "then idle;",
        "then cruise;",
        "final landed;",
        "transition first idle accept Go then flying;",
        "if fuel > 0",
        "do action report : Report",
    ] {
        assert!(text.contains(n), "missing `{n}`\n{text}");
    }
    assert_state_fields_equal(orig, find(&back, "Imp::Imp::P::Flight"), &text);
}

#[test]
fn state_usage_round_trips() {
    let els = ingest("package P { state def M; state run : M { state a; state b; then a; transition first a accept Go then b; } }");
    let orig = find(&els, "Imp::P::run");
    let (text, back) = export_and_back(&els);
    assert!(!text.contains("not exported"), "{text}");
    assert_state_fields_equal(orig, find(&back, "Imp::Imp::P::run"), &text);
}

#[test]
fn nested_transition_without_explicit_source_round_trips() {
    let els = ingest("package P { state def S { state a { accept Go then b; } state b; then a; } }");
    let orig = find(&els, "Imp::P::S");
    let (text, back) = export_and_back(&els);
    assert!(!text.contains("not exported"), "{text}");
    assert_state_fields_equal(orig, find(&back, "Imp::Imp::P::S"), &text);
}

#[test]
fn accept_via_and_time_trigger_round_trip() {
    let els = ingest("package P { state def S { state a; state b; then a; transition first a accept Go via port1 then b; transition first b accept after 5 then a; } }");
    let orig = find(&els, "Imp::P::S");
    assert_eq!(orig.frontmatter.transitions.as_ref().map(Vec::len), Some(2));
    let (text, back) = export_and_back(&els);
    assert!(!text.contains("not exported"), "{text}");
    assert!(text.contains("accept Go via port1") && text.contains("accept after 5"), "{text}");
    assert_state_fields_equal(orig, find(&back, "Imp::Imp::P::S"), &text);
}

#[test]
fn exported_behaviour_is_stable_under_re_export() {
    let els = ingest(&format!("{ACTIONS}{STATES}").replace("package P {\n    action def Takeoff", "package Q {\n    action def Takeoff"));
    let (t1, back) = export_and_back(&els);
    // Re-exporting the re-ingested elements gives the same body text (modulo the added `Imp::` nesting).
    let t2 = export_sysml(&back, None).unwrap().text;
    assert!(t2.contains("action takeoff : Takeoff;") && t1.contains("action takeoff : Takeoff;"));
    assert_eq!(t1.matches("while alt < 100 {").count(), t2.matches("while alt < 100 {").count());
}

// ── REQ-TRS-SYSMLV2-058: degrade to comments, never approximate ────────────

#[test]
fn unrepresentable_action_entries_become_comments_and_the_rest_round_trips() {
    let r = root_with(&[(
        "B/Mission.md",
        "---\ntype: ActionDef\nname: Mission\nsubActions:\n\
         \x20 - name: takeoff\n    kind: PerformAction\n    typedBy: B::Takeoff\n\
         \x20 - name: weird\n    kind: SomethingElse\n\
         \x20 - name: set_alt\n    kind: AssignmentAction\n    target: alt\n    value: 5\n\
         \x20 - name: assign_1\n    kind: AssignmentAction\n    target: alt\n    value: 6\n\
         \x20 - name: extra\n    kind: PerformAction\n    owner: somebody\n\
         \x20 - name: cond\n    kind: IfAction\n    condition: \"<conditional expression>\"\n\
         controlNodes:\n  - name: f1\n    kind: ForkNode\n  - name: bad\n    kind: Wibble\n\
         successionConnections:\n  - after: takeoff\n    before: f1\n  - after: takeoff\n---\n",
    )]);
    let els = walk_model(&r).unwrap();
    let (text, back) = export_and_back(&els);
    for what in [
        "// subAction not exported (unsupported kind 'SomethingElse'): SomethingElse weird",
        "action set_alt {",
        "// subAction not exported (unsupported fields): PerformAction extra",
        "// subAction not exported (does not parse): IfAction cond",
        "// controlNode not exported (unknown control node kind): Wibble bad",
        "// successionConnection not exported (no before): takeoff -> ?",
    ] {
        assert!(text.contains(what), "missing `{what}`\n{text}");
    }
    // What is exported is exactly what is read back.
    let b = find(&back, "Imp::B::Mission");
    let names: Vec<_> = b
        .frontmatter
        .sub_actions
        .as_ref()
        .unwrap()
        .iter()
        .map(|v| v.get("name").and_then(Value::as_str).unwrap().to_string())
        .collect();
    assert_eq!(names, vec!["takeoff", "set_alt", "assign_1"], "{text}");
    assert_eq!(b.frontmatter.control_nodes.as_ref().map(Vec::len), Some(1));
    assert_eq!(b.frontmatter.succession_connections.as_ref().map(Vec::len), Some(1));
}

#[test]
fn unrepresentable_state_entries_become_comments_and_the_rest_round_trips() {
    let r = root_with(&[(
        "B/Flight.md",
        "---\ntype: StateDef\nname: Flight\nentryAction:\n  name: boot\ndoAction: monitor\n\
         subStates:\n  - name: idle\n    isInitial: true\n  - name: odd\n    description: free text\n  - name: done\n    isFinal: true\n\
         transitions:\n  - source: idle\n    target: done\n    accept: Go\n\
         \x20 - target: done\n    accept: NoSource\n\
         \x20 - source: idle\n    target: done\n    guard: \"<conditional expression>\"\n\
         \x20 - source: idle\n    target: done\n    trigger: legacy\n---\n",
    )]);
    let els = walk_model(&r).unwrap();
    let (text, back) = export_and_back(&els);
    for what in [
        "// entryAction not exported (entryAction is not a plain action name)",
        "// subState not exported (unsupported fields): odd",
        "// transition not exported (top-level transition has no source): (implicit) -> done",
        "// transition not exported (does not read back identically): idle -> done",
        "// transition not exported (unsupported fields): idle -> done",
    ] {
        assert!(text.contains(what), "missing `{what}`\n{text}");
    }
    let b = find(&back, "Imp::B::Flight");
    assert_eq!(b.frontmatter.do_action, Some(Value::String("monitor".into())), "{text}");
    assert!(b.frontmatter.entry_action.is_none());
    let subs = b.frontmatter.sub_states.as_ref().unwrap();
    assert_eq!(subs.len(), 2, "{text}");
    assert_eq!(b.frontmatter.transitions.as_ref().map(Vec::len), Some(1), "{text}");
}

#[test]
fn degraded_behaviour_text_still_parses() {
    // Whatever mix of comments and statements is produced, the export is valid SysML v2 text.
    let r = root_with(&[(
        "B/M.md",
        "---\ntype: ActionDef\nname: M\nsubActions:\n  - name: x\n    kind: Nope\n  - name: ok\n    kind: PerformAction\n---\n",
    )]);
    let els = walk_model(&r).unwrap();
    let text = export_sysml(&els, None).unwrap().text;
    assert!(text.contains("// subAction not exported"));
    let _ = reimport(&text); // asserts no W541
}
