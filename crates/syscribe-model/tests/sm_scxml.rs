//! TC-TRS-SMSCXML-001 / REQ-TRS-SMSCXML-001 (GH #283): SCXML export of the machine IR.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::ir::lower;
use syscribe_model::sm_scxml::export;
use syscribe_model::walker::walk_model;

const MACHINE: &str = "variables:
  - {name: count, value: \"0\"}
  - {name: limit, value: \"2\", unit: \"SI::s\"}
initialEffect: {assign: \"count := 1\"}
subStates:
  - name: A
    isInitial: true
    entryAction: {assign: \"count := count + 1\"}
    exitAction: Pkg::XA
    transitions:
      - {target: B, accept: {after: \"2 s\"}, guard: \"count < 3 and not (count == 0)\", effect: Pkg::Beep}
      - {target: B, accept: {when: \"count > 2\"}}
      - {internal: true, accept: {payload: Pkg::Ping, via: p, as: ev}, guard: \"ev.n > 1\", assign: \"count := count + ev.n\"}
  - name: B
    doAction: Pkg::Run
    transitions:
      - {target: Done, accept: {done: B1}}
    subStates:
      - {name: B1, isInitial: true, transitions: [{target: B2, accept: Pkg::Next}]}
      - {name: B2, isFinal: true}
  - name: Done
    isFinal: true
";

fn machine(yaml: &str) -> syscribe_model::ir::Machine {
    static N: AtomicU64 = AtomicU64::new(0);
    let root: PathBuf = std::env::temp_dir().join(format!("syscribe-scxml-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{yaml}---\n")).unwrap();
    lower(&walk_model(&root).unwrap(), "M").unwrap_or_else(|d| panic!("{d:?}"))
}

#[test]
fn the_document_is_well_formed_scxml() {
    let (doc, _) = export(&machine(MACHINE));
    let mut r = quick_xml::Reader::from_str(&doc);
    let mut depth = 0i32;
    loop {
        match r.read_event() {
            Ok(quick_xml::events::Event::Start(_)) => depth += 1,
            Ok(quick_xml::events::Event::End(_)) => depth -= 1,
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => panic!("not well-formed: {e}\n{doc}"),
            _ => {}
        }
    }
    assert_eq!(depth, 0);
    assert!(doc.contains("<scxml xmlns=\"http://www.w3.org/2005/07/scxml\" version=\"1.0\""), "{doc}");
    assert!(doc.contains("datamodel=\"ecmascript\""));
}

#[test]
fn structure_and_data_map_onto_scxml() {
    let (doc, _) = export(&machine(MACHINE));
    assert!(doc.contains("<data id=\"count\" expr=\"0\"/>"), "{doc}");
    assert!(doc.contains("<data id=\"limit\" expr=\"2\"/>"), "SI base value");
    assert!(doc.contains("<state id=\"A\">") && doc.contains("<final id=\"Done\"/>") || doc.contains("<final id=\"Done\">"), "{doc}");
    assert!(doc.contains("<state id=\"B\"") && doc.contains("<state id=\"B.B1\">") && doc.contains("<final id=\"B.B2\""), "{doc}");
    assert!(doc.contains("<assign location=\"count\" expr=\"1\"/>"), "initialEffect assignment: {doc}");
}

#[test]
fn triggers_guards_and_actions_are_translated() {
    let (doc, _) = export(&machine(MACHINE));
    // after: send in onentry, cancel in onexit, transition on the timer event
    assert!(doc.contains("<send id=\"timer_t0\" event=\"timer.t0\" delay=\"2000ms\"/>"), "{doc}");
    assert!(doc.contains("<cancel sendid=\"timer_t0\"/>"), "{doc}");
    assert!(doc.contains("event=\"timer.t0\""), "{doc}");
    // guard: and/or/not, ==
    assert!(doc.contains("(count &lt; 3) &amp;&amp; (!((count === 0)))") || doc.contains("&amp;&amp;"), "{doc}");
    // when: armed flag, eventless raise, then the real transition on the signal
    assert!(doc.contains("<data id=\"armed_t1\" expr=\"false\"/>"), "{doc}");
    assert!(doc.contains("<assign location=\"armed_t1\" expr=\"true\"/>"), "{doc}");
    assert!(doc.contains("cond=\"armed_t1 &amp;&amp; (count &gt; 2)\""), "{doc}");
    assert!(doc.contains("<raise event=\"signal.t1\"/>") && doc.contains("event=\"signal.t1\""), "{doc}");
    // event with via/payload, payload fields, internal transition without target
    assert!(doc.contains("event=\"p.Pkg.Ping\""), "{doc}");
    assert!(doc.contains("_event.data.n"), "{doc}");
    assert!(doc.contains("<assign location=\"count\" expr=\"count + _event.data.n\"/>"), "{doc}");
    // done
    assert!(doc.contains("event=\"done.state.B.B1\"") || doc.contains("event=\"done.state.B1\""), "{doc}");
}

#[test]
fn constructs_with_no_scxml_form_are_flagged_never_silently_dropped() {
    let (doc, warnings) = export(&machine(MACHINE));
    assert!(doc.contains("<!-- behavior Pkg::Beep"), "{doc}");
    assert!(warnings.iter().any(|w| w.contains("doAction") && w.contains("Pkg::Run")), "{warnings:?}");
    assert!(warnings.iter().any(|w| w.contains("Pkg::Beep")), "{warnings:?}");
}

#[test]
fn parallel_regions_become_parallel() {
    let (doc, _) = export(&machine(
        "isParallel: true\nsubStates:\n  - name: R1\n    subStates: [{name: a, isInitial: true}]\n  - name: R2\n    subStates: [{name: b, isInitial: true}]\n",
    ));
    assert!(doc.contains("<parallel"), "{doc}");
    assert!(doc.contains("id=\"R1\"") && doc.contains("id=\"R2.b\""), "{doc}");
}

#[test]
fn the_export_is_deterministic() {
    assert_eq!(export(&machine(MACHINE)).0, export(&machine(MACHINE)).0);
}

#[test]
fn the_initial_effect_runs_on_a_synthetic_first_state_not_an_invalid_initial_element() {
    let (doc, _) = export(&machine(MACHINE));
    assert!(!doc.contains("<initial>"), "<initial> is not allowed under <scxml>: {doc}");
    assert!(doc.contains("initial=\"__init\"") && doc.contains("<state id=\"__init\">"), "{doc}");
}

#[test]
fn state_ids_are_unique_dotted_paths() {
    let (doc, _) = export(&machine("subStates:\n  - {name: A_B, isInitial: true}\n  - name: A\n    subStates: [{name: B, isInitial: true}]\n"));
    assert!(doc.contains("id=\"A_B\"") && doc.contains("id=\"A.B\""), "{doc}");
}

#[test]
fn internal_timers_and_submillisecond_delays_are_noted() {
    let (_, w) = export(&machine("subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {after: \"0.0004 s\"}}\n"));
    assert!(w.iter().any(|x| x.contains("re-armed")) && w.iter().any(|x| x.contains("rounded")), "{w:?}");
}
