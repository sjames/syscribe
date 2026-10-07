//! `REQ-TRS-SYSMLV2-067`..`-072`: canonical forms in the export read-back check, the
//! `@SyscribeStep` annotation for fields the 0.54 grammar has no syntax for, the degradation count
//! in the export summary.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_yaml::Value;
use syscribe_model::element::RawElement;
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-sysmlv2-close-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn native(files: &[(&str, &str)]) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    for (p, c) in files {
        write(&r, p, c);
    }
    walk_model(&r).unwrap()
}

fn reimport(text: &str) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/out.sysml", text);
    let els = walk_model(&r).unwrap();
    let bad: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "parse-back failed: {bad:?}\n{text}");
    els
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("no {q}"))
}

fn yaml(v: &Option<Vec<Value>>) -> Value {
    Value::Sequence(v.clone().unwrap_or_default())
}

fn entry<'a>(list: &'a Option<Vec<Value>>, name: &str) -> &'a Value {
    list.as_ref().unwrap().iter().find(|v| v.get("name").and_then(Value::as_str) == Some(name)).unwrap_or_else(|| panic!("no entry {name}"))
}

// ── REQ-TRS-SYSMLV2-067 ────────────────────────────────────────────────────

#[test]
fn accept_payload_mapping_exports_like_the_plain_string() {
    let els = native(&[(
        "B/S.md",
        "---\ntype: StateDef\nname: S\nsubStates:\n  - name: a\n    isInitial: true\n    transitions:\n\
         \x20     - target: b\n        accept:\n          payload: Cmd\n\
         \x20     - target: b\n        accept: Cmd\n\
         \x20     - target: b\n        accept:\n          payload: after 5\n\
         \x20 - name: b\ntransitions:\n  - source: a\n    target: b\n    accept:\n      payload: Top\n---\n",
    )]);
    let text = export_sysml(&els, None).unwrap().text;
    assert!(!text.contains("not exported"), "{text}");
    assert_eq!(text.matches("accept Cmd then b;").count(), 2, "{text}");
    assert!(text.contains("accept after 5 then b;"), "{text}");
    assert!(text.contains("transition first a accept Top then b;"), "{text}");
    // The reimported transitions are the canonical (string) form of the same triggers.
    let back = reimport(&text);
    let s = find(&back, "Imp::B::S");
    let a = s.frontmatter.sub_states.as_ref().unwrap().iter().find(|v| v.get("name").and_then(Value::as_str) == Some("a")).unwrap();
    let accepts: Vec<_> = a.get("transitions").unwrap().as_sequence().unwrap().iter().map(|t| t.get("accept").cloned().unwrap()).collect();
    assert_eq!(accepts[0], Value::String("Cmd".into()));
    assert_eq!(accepts[1], Value::String("Cmd".into()));
}

#[test]
fn accept_with_via_stays_a_mapping() {
    let els = native(&[(
        "B/S.md",
        "---\ntype: StateDef\nname: S\nsubStates:\n  - name: a\n    transitions:\n      - target: b\n        accept:\n          payload: Cmd\n          via: gate\n\
         \x20 - name: b\n---\n",
    )]);
    let text = export_sysml(&els, None).unwrap().text;
    assert!(text.contains("accept Cmd via gate then b;"), "{text}");
    assert!(!text.contains("not exported"), "{text}");
}

// ── REQ-TRS-SYSMLV2-068 ────────────────────────────────────────────────────

#[test]
fn equal_expression_spellings_export_as_text() {
    let els = native(&[
        (
            "B/S.md",
            "---\ntype: StateDef\nname: S\nsubStates:\n  - name: a\n    transitions:\n      - target: b\n        guard: \"x > 1 and y < 2\"\n\
             \x20 - name: b\n---\n",
        ),
        (
            "B/A.md",
            "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: c\n    kind: IfAction\n    condition: \"p and q\"\n\
             \x20 - name: w\n    kind: AssignmentAction\n    target: v\n    value: 10\n---\n",
        ),
    ]);
    let out = export_sysml(&els, None).unwrap();
    assert!(!out.text.contains("not exported"), "{}", out.text);
    assert_eq!(out.report.degraded_behaviour, 0);
    assert!(out.text.contains("x > 1 and y < 2"), "authored spelling is kept\n{}", out.text);
    let back = reimport(&out.text);
    let a = find(&back, "Imp::B::A");
    assert_eq!(entry(&a.frontmatter.sub_actions, "c").get("condition").and_then(Value::as_str), Some("p && q"));
}

#[test]
fn a_genuinely_different_or_unparseable_expression_still_degrades() {
    let els = native(&[(
        "B/A.md",
        "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: c\n    kind: IfAction\n    condition: \"<conditional expression>\"\n---\n",
    )]);
    let out = export_sysml(&els, None).unwrap();
    assert!(out.text.contains("// subAction not exported"), "{}", out.text);
    assert_eq!(out.report.degraded_behaviour, 1);
}

// ── REQ-TRS-SYSMLV2-069 / -070 ─────────────────────────────────────────────

const STEPS: &str = "---\ntype: ActionDef\nname: Steps\nsubActions:\n\
    \x20 - name: tell\n    kind: SendAction\n    payload: B::Cmd\n    via: controlOut\n\
    \x20 - name: listen\n    kind: AcceptAction\n    payload: B::Fix\n    via: gpsIn\n    trigger:\n      kind: change\n      condition: \"d < 2.0\"\n\
    \x20 - name: setIt\n    kind: AssignmentAction\n    target: self\n    referent: throttle\n    value: \"0.6\"\n    valueKind: initial\n\
    \x20 - name: plain\n    kind: AssignmentAction\n    target: x\n    value: \"1\"\n\
    \x20 - name: wait\n    kind: LoopAction\n    loopKind: until\n    condition: \"self.alt <= 0.1\"\n    body:\n\
    \x20     - name: inner\n        kind: AcceptAction\n        payload: B::Fix\n        via: gpsIn\n\
    successionConnections:\n  - after: tell\n    before: wait\n---\n";

#[test]
fn step_extension_fields_export_and_read_back_identically() {
    let els = native(&[("B/Steps.md", STEPS)]);
    let out = export_sysml(&els, None).unwrap();
    assert!(!out.text.contains("not exported"), "{}", out.text);
    assert_eq!(out.report.degraded_behaviour, 0);
    // `REQ-TRS-SYSMLV2-077`/`-079`: `via`, `referent` and `until` are native syntax now; the
    // deprecated annotation is written only for `valueKind` and a trigger beside a payload.
    for needle in ["@SyscribeStep {", "valueKind = 'initial';", "triggerKind = 'change';", "via controlOut", "assign self.throttle := 0.6;", "until self.alt <= 0.1;"] {
        assert!(out.text.contains(needle), "missing `{needle}`\n{}", out.text);
    }
    for gone in ["via = ", "feature = ", "loopKind = "] {
        assert!(!out.text.contains(gone), "`{gone}` must be native now\n{}", out.text);
    }
    let back = reimport(&out.text);
    let (a, b) = (find(&els, "B::Steps"), find(&back, "Imp::B::Steps"));
    assert_eq!(yaml(&a.frontmatter.sub_actions), yaml(&b.frontmatter.sub_actions), "{}", out.text);
    assert_eq!(yaml(&a.frontmatter.succession_connections), yaml(&b.frontmatter.succession_connections));
}

#[test]
fn step_annotation_does_not_disturb_synthesized_names_or_validation() {
    let els = native(&[("B/Steps.md", STEPS)]);
    let text = export_sysml(&els, None).unwrap().text;
    let back = reimport(&text);
    // `plain` is the first assign with no extras, so it is `assign_1` positionally only if bare.
    let s = find(&back, "Imp::B::Steps");
    assert_eq!(entry(&s.frontmatter.sub_actions, "plain").get("kind").and_then(Value::as_str), Some("AssignmentAction"));
    // The annotation is understood: ingesting it raises no unmapped-construct advisory.
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/out.sysml", &text);
    let els = walk_model(&r).unwrap();
    let findings = validate(&els).findings;
    let codes: Vec<String> = findings.iter().map(|f| f.code.to_string()).collect();
    assert!(!codes.iter().any(|c| matches!(c.as_str(), "W541" | "W543" | "W047")), "{codes:?}");
}

#[test]
fn unknown_or_non_text_extras_still_degrade() {
    let els = native(&[(
        "B/A.md",
        "---\ntype: ActionDef\nname: A\nsubActions:\n\
         \x20 - name: a\n    kind: SendAction\n    payload: B::Cmd\n    via: [x, y]\n\
         \x20 - name: b\n    kind: SendAction\n    payload: B::Cmd\n    priority: 3\n\
         \x20 - name: c\n    kind: AcceptAction\n    payload: B::Cmd\n    trigger:\n      kind: change\n---\n",
    )]);
    let out = export_sysml(&els, None).unwrap();
    assert_eq!(out.report.degraded_behaviour, 3, "{}", out.text);
}

// ── REQ-TRS-SYSMLV2-072 ────────────────────────────────────────────────────

#[test]
fn summary_counts_degraded_behaviour_entries() {
    let clean = export_sysml(&native(&[("B/Steps.md", STEPS)]), None).unwrap();
    assert!(clean.report.summary_line().ends_with("behaviour entries degraded to comments: 0"), "{}", clean.report.summary_line());
    let bad = native(&[(
        "B/A.md",
        "---\ntype: ActionDef\nname: A\nsubActions:\n  - name: a\n    kind: Wibble\n  - name: b\n    kind: PerformAction\n    owner: x\n---\n",
    )]);
    let out = export_sysml(&bad, None).unwrap();
    assert_eq!(out.report.degraded_behaviour, 2);
    assert!(out.text.contains("behaviour entries degraded to comments: 2"), "{}", out.text);
}
