//! Integration tests for mapping `state def`/`state` onto the native
//! `StateDef`/`State` schema (`REQ-TRS-SYSMLV2-018`).
//!
//! Two-layer, mirroring `sysmlv2_connections.rs`: (a) raw YAML shape
//! assertions on `frontmatter.sub_states`/`.transitions`, and (b) driving
//! `validator::validate` to confirm the *existing* `W070`–`W079` checks fire
//! on synthesized input exactly as they would on hand-authored input, with
//! no `validator.rs` changes.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-states-test-{}-{}",
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

fn base_model(root: &Path) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        root,
        "SysML2Legacy/_index.md",
        "---\ntype: Package\nname: SysML2Legacy\nsysmlSubmodel: true\n---\n",
    );
}

fn codes(findings: &[syscribe_model::validator::Finding]) -> Vec<&str> {
    findings.iter().map(|f| f.code).collect()
}

#[test]
fn a_top_level_state_def_becomes_a_real_element_with_substates() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flight.sysml",
        "package Behavior {\n\
         state def FlightStates {\n\
         state disarmed { transition first disarmed accept StartCmd then armed; }\n\
         state armed { transition first armed accept StopCmd then disarmed; }\n\
         then disarmed;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Behavior::FlightStates")
        .expect("FlightStates should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(syscribe_model::element::ElementType::StateDef));

    let subs = el.frontmatter.sub_states.as_deref().unwrap_or(&[]);
    assert_eq!(subs.len(), 2, "{subs:#?}");
    let disarmed = subs
        .iter()
        .find(|s| s.get("name").and_then(|v| v.as_str()) == Some("disarmed"))
        .expect("disarmed substate present");
    assert_eq!(disarmed.get("isInitial").and_then(|v| v.as_bool()), Some(true));

    // Nested states are inline data only, never separate elements.
    assert!(!elements.iter().any(|e| e.qualified_name.ends_with("::disarmed")));
    assert!(!elements.iter().any(|e| e.qualified_name.ends_with("::armed")));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "{:#?}", result.findings);
}

#[test]
fn transition_fields_use_the_canonical_source_target_accept_guard_effect_shape() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flight.sysml",
        "package Behavior {\n\
         state def FlightStates {\n\
         state disarmed {\n\
         transition first disarmed accept cmd : StartCmd if armStatus == 1 do action startTakeoff : Takeoff then armed;\n\
         }\n\
         state armed { transition first armed accept StopCmd then disarmed; }\n\
         then disarmed;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Behavior::FlightStates").unwrap();
    let subs = el.frontmatter.sub_states.as_deref().unwrap();
    let disarmed = subs.iter().find(|s| s.get("name").and_then(|v| v.as_str()) == Some("disarmed")).unwrap();
    let transitions = disarmed.get("transitions").and_then(|v| v.as_sequence()).expect("transitions present");
    assert_eq!(transitions.len(), 1);
    let t = &transitions[0];
    // An explicit `first disarmed` clause was written, so the AST's own
    // `source` is `Some` and is preserved verbatim -- even though the
    // transition is also nested under `disarmed`'s own body.
    assert_eq!(t.get("source").and_then(|v| v.as_str()), Some("disarmed"));
    assert_eq!(t.get("target").and_then(|v| v.as_str()), Some("armed"));
    assert!(t.get("accept").is_some());
    assert!(t.get("guard").is_some());
    assert!(t.get("effect").is_some());

    let result = validate(&elements);
    assert!(!codes(&result.findings).contains(&"W075"), "canonical keys must never trigger W075: {:#?}", result.findings);
}

#[test]
fn a_nested_transition_with_no_explicit_first_clause_omits_source_implicit_from_nesting() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flight.sysml",
        "package Behavior {\n\
         state def FlightStates {\n\
         state disarmed {\n\
         accept StartCmd then armed;\n\
         }\n\
         state armed { accept StopCmd then disarmed; }\n\
         then disarmed;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Behavior::FlightStates").unwrap();
    let subs = el.frontmatter.sub_states.as_deref().unwrap();
    let disarmed = subs.iter().find(|s| s.get("name").and_then(|v| v.as_str()) == Some("disarmed")).unwrap();
    let transitions = disarmed.get("transitions").and_then(|v| v.as_sequence()).expect("transitions present");
    assert_eq!(transitions.len(), 1);
    let t = &transitions[0];
    // No `first` clause was written -- the AST's own `source` is `None`,
    // omitted here since the substate's own `name:` already supplies the
    // implicit source (`transitions_from`'s `implicit_source` parameter).
    assert!(t.get("source").is_none(), "{t:#?}");
    assert_eq!(t.get("target").and_then(|v| v.as_str()), Some("armed"));

    // The implicit source still resolves correctly for completeness checks:
    // a clean two-state machine with both directions covered raises no W07x.
    let result = validate(&elements);
    let w07x: Vec<&str> = codes(&result.findings).into_iter().filter(|c| c.starts_with("W07")).collect();
    assert!(w07x.is_empty(), "{:#?}", result.findings);
}

#[test]
fn dead_and_trap_states_raise_w070_and_w071() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flight.sysml",
        "package Behavior {\n\
         state def Broken {\n\
         state a { transition first a accept Go then b; }\n\
         state b;\n\
         state c { transition first c accept Go then a; }\n\
         then a;\n\
         }\n\
         }\n",
    );
    // `c` has no incoming transition and isn't initial -> W070 dead state.
    // `b` has no outgoing transition and isn't final -> W071 trap state.

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let cs = codes(&result.findings);
    assert!(cs.contains(&"W070"), "{:#?}", result.findings);
    assert!(cs.contains(&"W071"), "{:#?}", result.findings);
}

#[test]
fn non_deterministic_transitions_raise_w072() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flight.sysml",
        "package Behavior {\n\
         state def Ambiguous {\n\
         state a {\n\
         transition first a accept Go then b;\n\
         transition first a accept Go then c;\n\
         }\n\
         state b;\n\
         state c;\n\
         then a;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    assert!(codes(&result.findings).contains(&"W072"), "{:#?}", result.findings);
}

#[test]
fn a_clean_state_machine_raises_no_w07x_findings() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flight.sysml",
        "package Behavior {\n\
         state def Clean {\n\
         state a { transition first a accept Go then b; }\n\
         state b { transition first b accept Back then a; }\n\
         then a;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let w07x: Vec<&str> = codes(&result.findings).into_iter().filter(|c| c.starts_with("W07")).collect();
    assert!(w07x.is_empty(), "{:#?}", result.findings);
}

#[test]
fn entry_do_exit_action_names_lift_onto_the_substate() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flight.sysml",
        "package Behavior {\n\
         state def FlightStates {\n\
         state takingOff {\n\
         entry action doTakeoff;\n\
         transition first takingOff accept Go then flying;\n\
         }\n\
         state flying { transition first flying accept Land then takingOff; }\n\
         then takingOff;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Behavior::FlightStates").unwrap();
    let subs = el.frontmatter.sub_states.as_deref().unwrap();
    let taking_off = subs.iter().find(|s| s.get("name").and_then(|v| v.as_str()) == Some("takingOff")).unwrap();
    assert_eq!(taking_off.get("entryAction").and_then(|v| v.as_str()), Some("doTakeoff"));
}

#[test]
fn a_top_level_standalone_state_usage_becomes_its_own_element() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Flight.sysml",
        "package Behavior {\n\
         state def Idle;\n\
         state armedState : Idle;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let qnames: Vec<&str> = elements.iter().map(|e| e.qualified_name.as_str()).collect();
    assert!(qnames.contains(&"SysML2Legacy::Behavior::armedState"), "{qnames:#?}");
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Behavior::armedState").unwrap();
    assert_eq!(el.frontmatter.element_type, Some(syscribe_model::element::ElementType::State));
}
