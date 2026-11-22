//! Stress tests for the state-machine interpreter: deep hierarchy, nested parallel regions,
//! cross-region preemption, entry through a region, done-chains and timers across regions.
//! Expectations follow SCXML (W3C §3, Appendix D) as the design doc states.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::expr::{format_value, parse_value};
use syscribe_model::ir::{lower, Machine};
use syscribe_model::sm_run::{Options, Run};
use syscribe_model::walker::walk_model;

fn machine(yaml: &str) -> Machine {
    static N: AtomicU64 = AtomicU64::new(0);
    let root: PathBuf = std::env::temp_dir().join(format!("syscribe-smstress-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{yaml}---\n")).unwrap();
    let elems = walk_model(&root).unwrap();
    lower(&elems, "M").unwrap_or_else(|d| panic!("lowering failed: {d:?}"))
}

fn num(r: &Run, name: &str) -> String {
    format_value(&r.var(name).unwrap())
}

fn last_move<'a>(r: &'a Run) -> &'a syscribe_model::sm_run::Step {
    r.trace().iter().rev().find(|s| s.transition.is_some()).expect("a transition step")
}

// ── hierarchy ────────────────────────────────────────────────────────────────

const DEEP: &str = "subStates:
  - name: L1
    isInitial: true
    subStates:
      - name: L2a
        isInitial: true
        subStates:
          - name: L3a
            isInitial: true
            subStates:
              - name: L4a
                isInitial: true
                transitions:
                  - {target: L1.L2b.L3b.L4b, accept: Pkg::Cross, effect: Pkg::T}
              - name: L4x
      - name: L2b
        subStates:
          - name: L3b
            isInitial: true
            subStates:
              - name: L4b
                isInitial: true
              - name: L4c
";

#[test]
fn deep_start_enters_every_level_outermost_first() {
    let m = machine(DEEP);
    let r = Run::new(&m, Options::default());
    assert_eq!(r.active(), ["L1", "L1.L2a", "L1.L2a.L3a", "L1.L2a.L3a.L4a"]);
}

#[test]
fn a_cross_branch_transition_exits_innermost_first_and_enters_outermost_first_under_the_lca() {
    let m = machine(DEEP);
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Cross", &[]);
    assert_eq!(r.active(), ["L1", "L1.L2b", "L1.L2b.L3b", "L1.L2b.L3b.L4b"]);
    let s = last_move(&r);
    assert_eq!(s.exited, ["L1.L2a.L3a.L4a", "L1.L2a.L3a", "L1.L2a"], "L1 is the LCA and stays");
    assert_eq!(s.entered, ["L1.L2b", "L1.L2b.L3b", "L1.L2b.L3b.L4b"]);
    assert_eq!(s.actions, ["behavior Pkg::T"]);
}

#[test]
fn an_external_self_transition_on_a_composite_resets_its_children_to_the_initial() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    transitions:
      - {target: P, accept: Pkg::Reset}
    subStates:
      - name: C1
        isInitial: true
        transitions: [{target: C2, accept: Pkg::Next}]
      - name: C2
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Next", &[]);
    assert_eq!(r.active(), ["P", "P.C2"]);
    r.send("Pkg::Reset", &[]);
    assert_eq!(r.active(), ["P", "P.C1"]);
    let s = last_move(&r);
    assert_eq!(s.exited, ["P.C2", "P"]);
    assert_eq!(s.entered, ["P", "P.C1"]);
}

#[test]
fn an_internal_transition_on_a_composite_keeps_its_children() {
    let m = machine(
        "variables:
  - {name: n, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    transitions:
      - {internal: true, accept: Pkg::Tick, assign: \"n := n + 1\"}
    subStates:
      - name: C1
        isInitial: true
        transitions: [{target: C2, accept: Pkg::Next}]
      - name: C2
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Next", &[]);
    r.send("Pkg::Tick", &[]);
    assert_eq!(r.active(), ["P", "P.C2"]);
    assert_eq!(num(&r, "n"), "1");
    let s = last_move(&r);
    assert!(s.exited.is_empty() && s.entered.is_empty());
}

#[test]
fn a_child_to_ancestor_transition_exits_and_reenters_the_ancestor() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    subStates:
      - name: C
        isInitial: true
        transitions: [{target: P, accept: Pkg::Up}]
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Up", &[]);
    assert_eq!(r.active(), ["P", "P.C"]);
    let s = last_move(&r);
    assert_eq!(s.exited, ["P.C", "P"], "target is a proper ancestor: external, so P is left and re-entered");
    assert_eq!(s.entered, ["P", "P.C"]);
}

#[test]
fn entry_and_exit_behaviors_run_in_hierarchy_order() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    entryAction: Pkg::EP
    exitAction: Pkg::XP
    transitions: [{target: Q, accept: Pkg::Go, effect: Pkg::T}]
    subStates:
      - name: C
        isInitial: true
        entryAction: Pkg::EC
        exitAction: Pkg::XC
  - name: Q
    entryAction: Pkg::EQ
    subStates:
      - name: D
        isInitial: true
        entryAction: Pkg::ED
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(last_move(&r).actions, ["behavior Pkg::XC", "behavior Pkg::XP", "behavior Pkg::T", "behavior Pkg::EQ", "behavior Pkg::ED"]);
}

// ── parallel ─────────────────────────────────────────────────────────────────

const NESTED_PAR: &str = "variables:
  - {name: hits, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: A
        subStates:
          - {name: a1, isInitial: true, transitions: [{target: a2, accept: Pkg::Go, assign: \"hits := hits + 1\"}]}
          - {name: a2}
      - name: B
        subStates:
          - name: b1
            isInitial: true
            isParallel: true
            subStates:
              - name: B1
                subStates:
                  - {name: x1, isInitial: true, transitions: [{target: x2, accept: Pkg::Go, assign: \"hits := hits + 10\"}]}
                  - {name: x2}
              - name: B2
                subStates:
                  - {name: y1, isInitial: true, transitions: [{target: y2, accept: Pkg::Go, assign: \"hits := hits + 100\"}]}
                  - {name: y2}
          - name: b2
";

#[test]
fn nested_parallel_regions_all_start_and_all_react_to_one_event() {
    let m = machine(NESTED_PAR);
    let mut r = Run::new(&m, Options::default());
    let start = r.active();
    for s in ["P.A.a1", "P.B.b1.B1.x1", "P.B.b1.B2.y1"] {
        assert!(start.iter().any(|a| a == s), "{s} active at start: {start:?}");
    }
    r.send("Pkg::Go", &[]);
    let now = r.active();
    for s in ["P.A.a2", "P.B.b1.B1.x2", "P.B.b1.B2.y2"] {
        assert!(now.iter().any(|a| a == s), "{s} active after Go: {now:?}");
    }
    assert_eq!(num(&r, "hits"), "111");
}

#[test]
fn one_event_in_nested_parallel_is_one_microstep_taking_three_transitions() {
    let m = machine(NESTED_PAR);
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    let moves: Vec<_> = r.trace().iter().filter(|s| s.transition.is_some()).collect();
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].taken.len(), 3);
}

#[test]
fn leaving_a_parallel_state_exits_every_region_in_reverse_document_order() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    isParallel: true
    transitions: [{target: Out, accept: Pkg::Quit}]
    subStates:
      - name: R1
        subStates: [{name: a, isInitial: true, exitAction: Pkg::XA}]
      - name: R2
        subStates: [{name: b, isInitial: true, exitAction: Pkg::XB}]
  - name: Out
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Quit", &[]);
    assert_eq!(r.active(), ["Out"]);
    let s = last_move(&r);
    assert_eq!(s.actions, ["behavior Pkg::XB", "behavior Pkg::XA"], "SCXML exits in reverse document order: later region first, children before parents");
    assert_eq!(s.exited.last().unwrap(), "P");
    assert_eq!(s.exited.len(), 5);
}

#[test]
fn a_region_that_exits_the_parallel_state_conflicts_with_a_sibling_region_by_document_order() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates:
          - {name: a, isInitial: true, transitions: [{target: Out, accept: Pkg::Go}]}
      - name: R2
        subStates:
          - {name: b, isInitial: true, transitions: [{target: c, accept: Pkg::Go}]}
          - {name: c}
  - name: Out
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["Out"], "R1's transition exits R2 too; it is first in document order, so R2's is dropped");
}

#[test]
fn the_conflict_resolves_the_other_way_when_the_internal_region_comes_first() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates:
          - {name: b, isInitial: true, transitions: [{target: c, accept: Pkg::Go}]}
          - {name: c}
      - name: R2
        subStates:
          - {name: a, isInitial: true, transitions: [{target: Out, accept: Pkg::Go}]}
  - name: Out
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["P", "P.R1", "P.R1.c", "P.R2", "P.R2.a"], "R1.b→c is first; R2's exit-everything transition conflicts and is dropped");
}

// Strict mode reports a tie that only document order resolves: two enabled transitions in different
// regions whose exit sets overlap (SCXML removeConflictingTransitions, the "neither is a descendant" case).
#[test]
fn strict_mode_reports_the_cross_region_conflict() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates: [{name: a, isInitial: true, transitions: [{target: Out, accept: Pkg::Go}]}]
      - name: R2
        subStates:
          - {name: b, isInitial: true, transitions: [{target: c, accept: Pkg::Go}]}
          - {name: c}
  - name: Out
",
    );
    let mut r = Run::new(&m, Options { strict: true, ..Options::default() });
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_some(), "strict: conflicting pair is an error");
}

#[test]
fn targeting_one_region_from_outside_starts_the_other_regions_in_their_initials() {
    let m = machine(
        "subStates:
  - name: Idle
    isInitial: true
    transitions: [{target: P.R1.a2, accept: Pkg::Go}]
  - name: P
    isParallel: true
    subStates:
      - name: R1
        subStates:
          - {name: a1, isInitial: true}
          - {name: a2}
      - name: R2
        subStates:
          - {name: b1, isInitial: true}
          - {name: b2}
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["P", "P.R1", "P.R1.a2", "P.R2", "P.R2.b1"]);
}

#[test]
fn a_transition_between_regions_leaves_the_other_region_untouched() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates:
          - {name: a1, isInitial: true, transitions: [{target: a2, accept: Pkg::Go}], exitAction: Pkg::XA1}
          - {name: a2}
      - name: R2
        subStates:
          - {name: b1, isInitial: true, exitAction: Pkg::XB1}
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(last_move(&r).actions, ["behavior Pkg::XA1"]);
    assert!(r.active().contains(&"P.R2.b1".to_string()));
}

#[test]
fn done_of_a_parallel_state_waits_for_every_region_then_fires_once() {
    let m = machine(
        "variables:
  - {name: done_count, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    isParallel: true
    transitions: [{target: After, accept: {done: P}, assign: \"done_count := done_count + 1\"}]
    subStates:
      - name: R1
        subStates:
          - {name: a, isInitial: true, transitions: [{target: af, accept: Pkg::One}]}
          - {name: af, isFinal: true}
      - name: R2
        subStates:
          - {name: b, isInitial: true, transitions: [{target: bf, accept: Pkg::Two}]}
          - {name: bf, isFinal: true}
  - name: After
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::One", &[]);
    assert!(r.active().contains(&"P".to_string()), "one region final is not enough");
    r.send("Pkg::Two", &[]);
    assert_eq!(r.active(), ["After"]);
    assert_eq!(num(&r, "done_count"), "1");
}

#[test]
fn done_cascades_up_through_nested_composites_in_one_macrostep() {
    let m = machine(
        "subStates:
  - name: Outer
    isInitial: true
    transitions: [{target: End, accept: {done: Outer}}]
    subStates:
      - name: Mid
        isInitial: true
        transitions: [{target: Fin, accept: {done: Mid}}]
        subStates:
          - {name: Leaf, isInitial: true, transitions: [{target: LeafEnd, accept: Pkg::Go}]}
          - {name: LeafEnd, isFinal: true}
      - {name: Fin, isFinal: true}
  - name: End
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["End"], "Leaf→LeafEnd completes Mid → Fin completes Outer → End");
}

#[test]
fn a_done_chain_through_entry_runs_without_any_external_event() {
    let m = machine(
        "subStates:
  - name: S
    isInitial: true
    transitions: [{target: P, accept: Pkg::Go}]
  - name: P
    transitions: [{target: End, accept: {done: P}}]
    subStates:
      - {name: only, isInitial: true, isFinal: true}
  - name: End
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["End"]);
}

#[test]
fn eventless_transitions_in_two_regions_settle_in_one_macrostep_and_respect_guards() {
    let m = machine(
        "variables:
  - {name: x, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates:
          - {name: a, isInitial: true, entryAction: {assign: \"x := 1\"}, transitions: [{target: a2, guard: \"x == 1\"}]}
          - {name: a2, transitions: [{target: a3, guard: \"x == 2\"}]}
          - {name: a3}
      - name: R2
        subStates:
          - {name: b, isInitial: true, transitions: [{target: b2, guard: \"x == 1\", assign: \"x := 2\"}]}
          - {name: b2}
",
    );
    let r = Run::new(&m, Options::default());
    assert!(r.error().is_none(), "{:?}", r.error());
    assert_eq!(r.active(), ["P", "P.R1", "P.R1.a3", "P.R2", "P.R2.b2"]);
    assert_eq!(num(&r, "x"), "2");
}

#[test]
fn timers_in_parallel_regions_fire_independently_and_are_cancelled_when_the_parent_is_left() {
    let src = "subStates:
  - name: P
    isInitial: true
    isParallel: true
    transitions: [{target: Out, accept: Pkg::Quit}]
    subStates:
      - name: R1
        subStates:
          - {name: a, isInitial: true, transitions: [{target: a2, accept: {after: \"1 s\"}}]}
          - {name: a2}
      - name: R2
        subStates:
          - {name: b, isInitial: true, transitions: [{target: b2, accept: {after: \"3 s\"}}]}
          - {name: b2}
  - name: Out
";
    let m = machine(src);
    let mut r = Run::new(&m, Options::default());
    r.advance(1.5);
    assert_eq!(r.active(), ["P", "P.R1", "P.R1.a2", "P.R2", "P.R2.b"]);
    r.advance(1.6);
    assert_eq!(r.active(), ["P", "P.R1", "P.R1.a2", "P.R2", "P.R2.b2"]);

    let mut r = Run::new(&m, Options::default());
    r.advance(0.5);
    r.send("Pkg::Quit", &[]);
    r.advance(10.0);
    assert_eq!(r.active(), ["Out"], "the 1 s and 3 s timers died with P");
    assert!(r.next_timer().is_none());
}

#[test]
fn re_entering_a_state_restarts_its_timer() {
    let m = machine(
        "subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: {after: \"2 s\"}}
      - {target: A, accept: Pkg::Kick}
  - name: B
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.5);
    r.send("Pkg::Kick", &[]);
    r.advance(1.5);
    assert_eq!(r.active(), ["A"], "timer restarted at 1.5 s; due at 3.5 s");
    r.advance(0.6);
    assert_eq!(r.active(), ["B"]);
}

#[test]
fn a_when_trigger_in_one_region_reacts_to_another_regions_assignment_in_the_same_macrostep() {
    let m = machine(
        "variables:
  - {name: flag, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates:
          - {name: a, isInitial: true, transitions: [{target: a2, accept: Pkg::Go, assign: \"flag := 1\"}]}
          - {name: a2}
      - name: R2
        subStates:
          - {name: b, isInitial: true, transitions: [{target: b2, accept: {when: \"flag == 1\"}}]}
          - {name: b2}
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["P", "P.R1", "P.R1.a2", "P.R2", "P.R2.b2"]);
}

#[test]
fn parallel_regions_in_a_composite_reentered_repeatedly_do_not_leak_state() {
    let m = machine(
        "variables:
  - {name: n, value: \"0\"}
subStates:
  - name: Off
    isInitial: true
    transitions: [{target: On, accept: Pkg::On}]
  - name: On
    isParallel: true
    transitions: [{target: Off, accept: Pkg::Off}]
    subStates:
      - name: R1
        subStates:
          - {name: a1, isInitial: true, entryAction: {assign: \"n := n + 1\"}, transitions: [{target: a2, accept: Pkg::Step}]}
          - {name: a2}
      - name: R2
        subStates:
          - {name: b1, isInitial: true, transitions: [{target: b2, accept: Pkg::Step}]}
          - {name: b2}
",
    );
    let mut r = Run::new(&m, Options::default());
    for i in 1..=50 {
        r.send("Pkg::On", &[]);
        assert_eq!(r.active(), ["On", "On.R1", "On.R1.a1", "On.R2", "On.R2.b1"], "round {i}");
        r.send("Pkg::Step", &[]);
        assert_eq!(r.active(), ["On", "On.R1", "On.R1.a2", "On.R2", "On.R2.b2"], "round {i}");
        r.send("Pkg::Off", &[]);
        assert_eq!(r.active(), ["Off"], "round {i}");
    }
    assert_eq!(num(&r, "n"), "50");
    assert!(r.error().is_none());
}

#[test]
fn a_deeply_nested_machine_survives_a_long_random_walk_with_a_stable_invariant() {
    // Invariant: exactly one leaf of each region is active, and a rerun reproduces the same trace.
    let m = machine(NESTED_PAR);
    let walk = |seed: u64| {
        let mut r = Run::new(&m, Options::default());
        let mut s = seed;
        for _ in 0..200 {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let _ = parse_value("1");
            r.send(if (s >> 33) % 2 == 0 { "Pkg::Go" } else { "Pkg::Noise" }, &[]);
            assert!(r.error().is_none());
            let act = r.active();
            let a = act.iter().filter(|x| x.starts_with("P.A.a")).count();
            assert_eq!(a, 1, "region A has one leaf: {act:?}");
        }
        r.trace().len()
    };
    assert_eq!(walk(7), walk(7));
}

#[test]
fn strict_mode_still_allows_deeper_preemption_and_disjoint_regions() {
    // Ancestor vs descendant is a defined priority, not a tie; regions with disjoint exit sets don't conflict.
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    isParallel: true
    transitions:
      - {target: Z, accept: Pkg::Go}
    subStates:
      - name: R1
        subStates:
          - {name: a, isInitial: true}
      - name: R2
        subStates:
          - {name: b, isInitial: true, transitions: [{target: c, accept: Pkg::Go}]}
          - {name: c}
  - name: Z
",
    );
    let mut r = Run::new(&m, Options { strict: true, ..Options::default() });
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    assert_eq!(r.active(), ["P", "P.R1", "P.R1.a", "P.R2", "P.R2.c"]);
}

#[test]
fn strict_mode_flags_even_commutative_shared_increments() {
    // Three regions each do `hits := hits + N`: the sum is order-independent, but strict mode does not analyse
    // commutativity; use separate variables per region.
    let m = machine(NESTED_PAR);
    let mut r = Run::new(&m, Options { strict: true, ..Options::default() });
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_some_and(|e| e.contains("'hits'")), "{:?}", r.error());
}

fn two_writers(second: &str) -> String {
    format!(
        "variables:
  - {{name: v, value: \"0\"}}
  - {{name: w, value: \"0\"}}
subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates:
          - {{name: a, isInitial: true, transitions: [{{target: a2, accept: Pkg::Go, assign: \"v := 1\"}}]}}
          - {{name: a2}}
      - name: R2
        subStates:
          - {{name: b, isInitial: true, transitions: [{{target: b2, accept: Pkg::Go, assign: \"{second}\"}}]}}
          - {{name: b2}}
"
    )
}

#[test]
fn strict_mode_reports_two_regions_assigning_the_same_variable() {
    let m = machine(&two_writers("v := 2"));
    let mut r = Run::new(&m, Options { strict: true, ..Options::default() });
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_some_and(|e| e.contains("'v'") && e.contains("region order")), "{:?}", r.error());
    // lenient mode keeps the document-order result
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none());
    assert_eq!(num(&r, "v"), "2");
}

#[test]
fn strict_mode_allows_regions_that_write_different_variables() {
    let m = machine(&two_writers("w := 2"));
    let mut r = Run::new(&m, Options { strict: true, ..Options::default() });
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    assert_eq!((num(&r, "v"), num(&r, "w")), ("1".into(), "2".into()));
}

#[test]
fn strict_mode_reports_a_same_source_tie_even_when_both_go_to_the_same_target() {
    let m = machine(
        "variables:
  - {name: v, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: Pkg::Go, assign: \"v := 1\"}
      - {target: B, accept: Pkg::Go, assign: \"v := 2\"}
  - name: B
",
    );
    let mut r = Run::new(&m, Options { strict: true, ..Options::default() });
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_some_and(|e| e.contains("t0") && e.contains("t1")), "{:?}", r.error());
}

#[test]
fn two_timers_on_one_state_are_both_restarted_by_an_external_self_transition() {
    // GH #512: leaving a state cancels all of its timers; re-entry arms each afresh
    let m = machine(
        "subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: {after: \"2 s\"}}
      - {target: C, accept: {after: \"3 s\"}}
      - {target: A, accept: Pkg::Kick}
  - name: B
  - name: C
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    r.send("Pkg::Kick", &[]);
    r.advance(1.9);
    assert_eq!(r.active(), ["A"], "both timers restarted at 1 s: due at 3 s and 4 s");
    r.advance(0.2);
    assert_eq!(r.active(), ["B"], "only the shorter timer fires, at its own time");
}

#[test]
fn equal_timers_do_not_both_fire_after_re_entry() {
    let m = machine(
        "subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: {after: \"2 s\"}}
      - {target: C, accept: {after: \"2 s\"}}
      - {target: A, accept: Pkg::Kick}
  - name: B
  - name: C
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    r.send("Pkg::Kick", &[]);
    r.advance(1.5);
    assert_eq!(r.active(), ["A"]);
}

#[test]
fn a_timer_driven_self_transition_restarts_the_states_other_timer() {
    let m = machine(
        "subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: A, accept: {after: \"1 s\"}}
      - {target: B, accept: {after: \"2.5 s\"}}
  - name: B
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(2.4);
    assert_eq!(r.active(), ["A"]);
    r.advance(0.2);
    assert_eq!(r.active(), ["A"], "the 2.5 s timer was cancelled at 1 s and restarted at 2 s, now due at 4.5 s");
    r.advance(2.0);
    assert_eq!(r.active(), ["A"], "the 1 s self-loop keeps cancelling the 2.5 s timer");
}

#[test]
fn simultaneous_timers_on_one_state_fire_only_the_first_because_leaving_cancels_the_other() {
    let m = machine(
        "variables:
  - {name: n, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: {after: \"2 s\"}, assign: \"n := n + 1\"}
      - {target: C, accept: {after: \"2 s\"}, assign: \"n := n + 10\"}
  - name: B
  - name: C
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(2.5);
    assert_eq!(num(&r, "n"), "1", "only one of the two timers of A may fire");
    assert_eq!(r.active().len(), 1, "{:?}", r.active());
}

#[test]
fn timers_of_a_nested_state_are_restarted_by_a_transition_declared_on_its_parent() {
    let m = machine(
        "subStates:
  - name: Live
    isInitial: true
    transitions:
      - {target: Live.Run, accept: Pkg::Kick}
    subStates:
      - name: Run
        isInitial: true
        transitions:
          - {target: Done, accept: {after: \"2 s\"}}
          - {target: Late, accept: {after: \"3 s\"}}
      - {name: Done}
      - {name: Late}
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    r.send("Pkg::Kick", &[]);
    r.advance(1.9);
    assert_eq!(r.active(), ["Live", "Live.Run"], "{:?}", r.active());
    r.advance(0.2);
    assert_eq!(r.active(), ["Live", "Live.Done"], "{:?}", r.active());
}

// ── GH #512 probes: more ways a state's two timers could outlive an external self-transition ──────────────────────────

fn two_timer_machine(extra: &str) -> Machine {
    machine(&format!(
        "variables:
  - {{name: b, value: \"0\"}}
  - {{name: c, value: \"0\"}}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {{target: B, accept: {{after: \"2 s\"}}, assign: \"b := b + 1\"}}
      - {{target: C, accept: {{after: \"3 s\"}}, assign: \"c := c + 1\"}}
      - {{target: A, accept: Pkg::Kick}}
  - name: B
  - name: C
{extra}"
    ))
}

#[test]
fn an_external_event_at_the_exact_instant_a_timer_is_due_leaves_no_timer_behind() {
    let m = two_timer_machine("");
    let mut r = Run::new(&m, Options::default());
    r.advance_to(2.0); // the 2 s timer fires here: A -> B; the kick below finds B
    r.send("Pkg::Kick", &[]);
    r.advance(5.0);
    assert_eq!((num(&r, "b"), num(&r, "c")), ("1".into(), "0".into()), "{:?} {:?}", r.active(), r.trace().iter().map(|s| s.note.clone()).collect::<Vec<_>>());
}

#[test]
fn a_kick_just_before_the_due_time_restarts_both_timers() {
    let m = two_timer_machine("");
    let mut r = Run::new(&m, Options::default());
    r.advance_to(1.999);
    r.send("Pkg::Kick", &[]);
    r.advance_to(3.0);
    assert_eq!(r.active(), ["A"], "restarted at 1.999 s: due at 3.999 s and 4.999 s");
    r.advance_to(4.0);
    assert_eq!((r.active(), num(&r, "b")), (vec!["B".to_string()], "1".into()));
    r.advance(5.0);
    assert_eq!(num(&r, "c"), "0", "the 3 s timer died with A");
}

#[test]
fn a_self_transition_in_one_region_restarts_the_timers_of_that_region_only() {
    let m = machine(
        "variables:
  - {name: n, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates:
          - name: A
            isInitial: true
            transitions:
              - {target: A1, accept: {after: \"2 s\"}, assign: \"n := n + 1\"}
              - {target: A2, accept: {after: \"3 s\"}, assign: \"n := n + 10\"}
              - {target: A, accept: Pkg::Kick}
          - name: A1
          - name: A2
      - name: R2
        subStates:
          - name: X
            isInitial: true
            transitions: [{target: Y, accept: {after: \"2.5 s\"}, assign: \"n := n + 100\"}]
          - name: Y
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    r.send("Pkg::Kick", &[]);
    r.advance(1.9); // t = 2.9: A restarted at 1 s (due 3 s, 4 s); R2's own timer fired at 2.5 s
    assert_eq!(num(&r, "n"), "100", "{:?}", r.active());
    r.advance(0.2); // t = 3.1: only A's 2 s timer (due 3 s) fires
    assert_eq!(num(&r, "n"), "101", "{:?}", r.active());
    r.advance(3.0);
    assert_eq!(num(&r, "n"), "101", "the 3 s timer of the old A must not fire");
}

#[test]
fn a_self_transition_declared_on_the_parent_restarts_the_timers_of_its_children() {
    let m = machine(
        "variables:
  - {name: n, value: \"0\"}
subStates:
  - name: Live
    isInitial: true
    transitions:
      - {target: Live, accept: Pkg::Kick}
    subStates:
      - name: Run
        isInitial: true
        transitions:
          - {target: Done, accept: {after: \"2 s\"}, assign: \"n := n + 1\"}
          - {target: Late, accept: {after: \"3 s\"}, assign: \"n := n + 10\"}
      - name: Done
      - name: Late
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    r.send("Pkg::Kick", &[]);
    r.advance(1.9);
    assert_eq!((num(&r, "n"), r.active()), ("0".into(), vec!["Live".to_string(), "Live.Run".to_string()]));
    r.advance(0.2);
    assert_eq!(num(&r, "n"), "1");
    r.advance(5.0);
    assert_eq!(num(&r, "n"), "1");
}

// ── GH #511 / ADR-SYS-AFTER-001: a timer's duration is an expression read when the timer is armed ─────────────────────

#[test]
fn after_can_name_a_variable() {
    let m = machine(
        "variables:
  - {name: deadline, value: \"2 s\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{target: B, accept: {after: deadline}}]
  - name: B
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.9);
    assert_eq!(r.active(), ["A"]);
    r.advance(0.2);
    assert_eq!(r.active(), ["B"]);
}

#[test]
fn after_can_be_an_expression_over_variables() {
    let m = machine(
        "variables:
  - {name: base, value: \"1 s\"}
  - {name: margin, value: \"500 ms\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{target: B, accept: {after: \"base + 2 * margin\"}}]
  - name: B
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.9);
    assert_eq!(r.active(), ["A"]);
    r.advance(0.2);
    assert_eq!(r.active(), ["B"], "1 s + 2 * 0.5 s = 2 s");
}

#[test]
fn the_value_is_read_when_the_state_is_entered_so_a_deadline_set_by_an_earlier_event_counts() {
    let m = machine(
        "variables:
  - {name: deadline, value: \"1 s\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{target: B, accept: Pkg::Go, assign: \"deadline := 5 s\"}]
  - name: B
    transitions: [{target: C, accept: {after: deadline}}]
  - name: C
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    r.advance(4.9);
    assert_eq!(r.active(), ["B"], "the 5 s assigned before entering B, not the initial 1 s");
    r.advance(0.2);
    assert_eq!(r.active(), ["C"]);
}

#[test]
fn an_armed_timer_is_not_re_timed_when_its_variable_changes() {
    let m = machine(
        "variables:
  - {name: deadline, value: \"5 s\"}
subStates:
  - name: B
    isInitial: true
    transitions:
      - {target: C, accept: {after: deadline}}
      - {internal: true, accept: Pkg::Shorten, assign: \"deadline := 1 s\"}
  - name: C
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    r.send("Pkg::Shorten", &[]);
    r.advance(3.9);
    assert_eq!(r.active(), ["B"], "the timer armed at 0 s stays due at 5 s");
    r.advance(0.2);
    assert_eq!(r.active(), ["C"]);
}

#[test]
fn an_internal_timer_transition_reads_the_variable_again_when_it_re_arms() {
    let m = machine(
        "variables:
  - {name: period, value: \"1 s\"}
  - {name: n, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{internal: true, accept: {after: period}, assign: \"n := n + 1; period := period * 2\"}]
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.05); // fires at 1 s, re-armed for 2 s more (3 s)
    assert_eq!(num(&r, "n"), "1");
    r.advance(1.5); // t = 2.55: not yet
    assert_eq!(num(&r, "n"), "1");
    r.advance(0.6); // t = 3.15: the second firing at 3 s
    assert_eq!(num(&r, "n"), "2");
}

#[test]
fn a_negative_duration_stops_the_run_with_the_expression_named() {
    let m = machine(
        "variables:
  - {name: deadline, value: \"1 s\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{target: B, accept: Pkg::Go, assign: \"deadline := 0 s - 1 s\"}]
  - name: B
    transitions: [{target: C, accept: {after: deadline}}]
  - name: C
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    let e = r.error().expect("a negative timer is a run-time error").to_string();
    assert!(e.contains("deadline") && e.contains("negative"), "{e}");
}
