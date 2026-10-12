//! GH #331: a port typed by a `PortDef` carries the events its items name; a machine's `send`/`accept` through it is
//! checked against them (`E932`: event not carried, carried the other way, unknown payload field; `E935`: a payload
//! value or field use of the wrong type or dimension). Conjugation reverses item directions. Untyped ports are checked
//! as before.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A sensor `Src` sending `Sample{v: V, ok: Boolean}` through a `SamplePort`, a monitor `Mon` accepting it through the
/// conjugated port and answering with `Trip`, joined in `Sys`. Clean.
fn base() -> BTreeMap<&'static str, String> {
    let mut m = BTreeMap::new();
    m.insert("Ifc/Sample.md", "---\ntype: ItemDef\nname: Sample\nfeatures:\n  - {name: v, typedBy: ScalarValues::Real, unit: V}\n  - {name: ok, typedBy: ScalarValues::Boolean}\n---\nA sample.\n".to_string());
    m.insert("Ifc/SamplePort.md", "---\ntype: PortDef\nname: SamplePort\nfeatures:\n  - {name: sample, typedBy: Ifc::Sample, direction: out}\n  - {name: trip, typedBy: Ifc::Trip, direction: in}\n---\nSamples out, trips in.\n".to_string());
    m.insert("Ifc/Trip.md", "---\ntype: ItemDef\nname: Trip\n---\nA trip.\n".to_string());
    m.insert("P/Mon.md", "---\ntype: PartDef\nname: Mon\nexhibitsStates: [P::MonM]\nfeatures:\n  - {name: inp, type: Port, direction: inout, typedBy: Ifc::SamplePort, isConjugated: true}\n---\n".to_string());
    m.insert("P/MonM.md", "---\ntype: StateDef\nname: MonM\nvariables:\n  - {name: last, value: \"0 V\"}\nsubStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: A, accept: {payload: Sample, via: inp, as: s}, guard: \"s.v > 2 V and s.ok\", assign: \"last := s.v\", effect: {send: Trip, via: inp}}\n---\n".to_string());
    m.insert("P/Src.md", "---\ntype: PartDef\nname: Src\nexhibitsStates: [P::SrcM]\nfeatures:\n  - {name: out, type: Port, direction: inout, typedBy: Ifc::SamplePort}\n---\n".to_string());
    m.insert("P/SrcM.md", "---\ntype: StateDef\nname: SrcM\nvariables:\n  - {name: n, value: \"0\"}\nsubStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: A, accept: Tick, effect: {send: Sample, via: out, payload: {v: \"3 V\", ok: \"true\"}}}\n      - {target: A, accept: {payload: Trip, via: out}, assign: \"n := n + 1\"}\n---\n".to_string());
    m.insert("P/Sys.md", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: src, typedBy: P::Src}\n  - {name: mon, typedBy: P::Mon}\nconnections:\n  - {from: src.out, to: mon.inp}\n  - {from: mon.inp, to: src.out}\n---\n".to_string());
    m.insert("_index.md", "---\ntype: Package\nname: Root\n---\n".to_string());
    m
}

fn run(files: &BTreeMap<&'static str, String>) -> String {
    static N: AtomicU64 = AtomicU64::new(0);
    let r: PathBuf = std::env::temp_dir().join(format!("syscribe-ptypes-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&r);
    for (k, v) in files {
        let p = r.join(k);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, v).unwrap();
    }
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&r).output().unwrap();
    let _ = std::fs::remove_dir_all(&r);
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn lines<'a>(out: &'a str, code: &str) -> Vec<&'a str> {
    out.lines().filter(|l| l.starts_with(&format!("| {code} |"))).collect()
}

fn edit(f: &mut BTreeMap<&'static str, String>, k: &'static str, from: &str, to: &str) {
    let v = f.get_mut(k).unwrap();
    assert!(v.contains(from), "{k} has no {from:?}");
    *v = v.replace(from, to);
}

fn has(out: &str, code: &str, needle: &str) {
    assert!(lines(out, code).iter().any(|l| l.contains(needle)), "no {code} containing {needle:?}:\n{out}");
}

#[test]
fn matching_typed_ports_are_clean() {
    let out = run(&base());
    for c in ["E932", "E935", "E931", "W936", "W047"] {
        assert!(lines(&out, c).is_empty(), "{c}:\n{out}");
    }
}

#[test]
fn an_event_the_port_type_does_not_carry() {
    let mut f = base();
    edit(&mut f, "P/SrcM.md", "send: Sample, via: out,", "send: Sampel, via: out,");
    has(&run(&f), "E932", "sends Sampel via 'out', but its type 'Ifc::SamplePort' carries no Sampel (it carries Sample, Trip)");
    let mut f = base();
    edit(&mut f, "P/MonM.md", "accept: {payload: Sample, via: inp, as: s}", "accept: {payload: Sampel, via: inp, as: s}");
    has(&run(&f), "E932", "accepts Sampel via 'inp', but its type 'Ifc::SamplePort' carries no Sampel");
}

#[test]
fn an_event_carried_the_other_way_and_conjugation() {
    let mut f = base();
    // Src sends Trip, which SamplePort carries inward at an unconjugated port
    edit(&mut f, "P/SrcM.md", "accept: {payload: Trip, via: out}, assign: \"n := n + 1\"", "accept: Tuck, effect: {send: Trip, via: out}");
    has(&run(&f), "E932", "sends Trip via 'out', but 'Ifc::SamplePort' carries Trip inward");
    // without `isConjugated`, Mon's port carries Sample outward, so accepting it is wrong
    let mut f = base();
    edit(&mut f, "P/Mon.md", ", isConjugated: true", "");
    let out = run(&f);
    has(&out, "E932", "accepts Sample via 'inp', but 'Ifc::SamplePort' carries Sample outward");
    has(&out, "E932", "sends Trip via 'inp', but 'Ifc::SamplePort' carries Trip inward");
}

#[test]
fn an_unknown_payload_field() {
    let mut f = base();
    edit(&mut f, "P/SrcM.md", "payload: {v: \"3 V\", ok: \"true\"}", "payload: {v: \"3 V\", ok: \"true\", x: \"1\"}");
    has(&run(&f), "E932", "with payload field 'x', which Sample does not have (it has v, ok)");
    let mut f = base();
    edit(&mut f, "P/MonM.md", "s.v > 2 V and s.ok", "s.v > 2 V and s.okay");
    has(&run(&f), "E932", "uses 's.okay', but Sample has no field 'okay' (it has v, ok)");
}

#[test]
fn a_payload_of_the_wrong_type_or_dimension() {
    let mut f = base();
    edit(&mut f, "P/SrcM.md", "v: \"3 V\"", "v: \"3 A\"");
    has(&run(&f), "E935", "payload 'v' of Sample is a number");
    let mut f = base();
    edit(&mut f, "P/SrcM.md", "ok: \"true\"", "ok: \"1\"");
    has(&run(&f), "E935", "payload 'ok' of Sample is a boolean, but `1` is a number");
    let mut f = base();
    edit(&mut f, "P/MonM.md", "s.v > 2 V and s.ok", "s.v > 2 and s.ok");
    has(&run(&f), "E935", "`s.v > 2 and s.ok`: cannot compare");
    let mut f = base();
    edit(&mut f, "P/MonM.md", "assign: \"last := s.v\"", "assign: \"last := s.ok\"");
    has(&run(&f), "E935", "`last := s.ok` assigns a boolean to a number");
}

#[test]
fn untyped_ports_are_checked_as_before() {
    let mut f = base();
    for k in ["P/Src.md", "P/Mon.md"] {
        let v = f.get_mut(k).unwrap();
        *v = v.replace(", typedBy: Ifc::SamplePort", "").replace(", isConjugated: true", "");
    }
    edit(&mut f, "P/SrcM.md", "payload: {v: \"3 V\", ok: \"true\"}", "payload: {v: \"3 A\", ok: \"1\", x: \"1\"}");
    let out = run(&f);
    for c in ["E930", "E931", "E932", "E933", "E934", "E935"] {
        assert!(lines(&out, c).is_empty(), "{c}:\n{out}");
    }
}

#[test]
fn a_port_def_with_no_items_is_an_untyped_port() {
    // a legacy PortDef with only an attribute: the port is checked by name and direction, as before
    let mut f = base();
    f.insert("Ifc/SamplePort.md", "---\ntype: PortDef\nname: SamplePort\nfeatures:\n  - {name: voltage, typedBy: ScalarValues::Real, unit: V, direction: out}\n---\nA flow.\n".to_string());
    let out = run(&f);
    for c in ["E932", "E935"] {
        assert!(lines(&out, c).is_empty(), "{c}:\n{out}");
    }
}

#[test]
fn an_item_whose_item_def_does_not_resolve_is_still_carried() {
    let mut f = base();
    edit(&mut f, "Ifc/SamplePort.md", "typedBy: Ifc::Trip,", "typedBy: Ifc::Trap,");
    f.insert("P/MonM.md", f["P/MonM.md"].replace("send: Trip, via: inp", "send: Trap, via: inp"));
    f.insert("P/SrcM.md", f["P/SrcM.md"].replace("payload: Trip, via: out", "payload: Trap, via: out"));
    let out = run(&f);
    assert!(!lines(&out, "E111").is_empty(), "{out}");
    assert!(lines(&out, "E932").is_empty(), "the unresolved item is reported once (E111), not at every use:\n{out}");
}

#[test]
fn one_item_def_declared_both_ways_flows_both_ways() {
    let mut f = base();
    edit(&mut f, "Ifc/SamplePort.md", "  - {name: trip,", "  - {name: echo, typedBy: Ifc::Sample, direction: in}\n  - {name: trip,");
    // Src (unconjugated) also accepts a Sample back through its port
    edit(&mut f, "P/SrcM.md", "accept: {payload: Trip, via: out}", "accept: {payload: Sample, via: out}");
    let out = run(&f);
    assert!(lines(&out, "E932").is_empty(), "{out}");
}

#[test]
fn child_port_elements_and_inherited_ports_are_typed_too() {
    // Mon's port as a child Port element
    let mut f = base();
    edit(&mut f, "P/Mon.md", "features:\n  - {name: inp, type: Port, direction: inout, typedBy: Ifc::SamplePort, isConjugated: true}\n", "");
    f.insert("P/Mon/inp.md", "---\ntype: Port\nname: inp\ndirection: inout\ntypedBy: Ifc::SamplePort\nisConjugated: true\n---\n".to_string());
    edit(&mut f, "P/MonM.md", "accept: {payload: Sample, via: inp, as: s}", "accept: {payload: Sampel, via: inp, as: s}");
    has(&run(&f), "E932", "accepts Sampel via 'inp', but its type 'Ifc::SamplePort' carries no Sampel");
    // Src's port inherited from a supertype
    let mut f = base();
    f.insert("P/SrcBase.md", "---\ntype: PartDef\nname: SrcBase\nfeatures:\n  - {name: out, type: Port, direction: inout, typedBy: Ifc::SamplePort}\n---\n".to_string());
    f.insert("P/Src.md", "---\ntype: PartDef\nname: Src\nsupertype: P::SrcBase\nexhibitsStates: [P::SrcM]\n---\n".to_string());
    edit(&mut f, "P/SrcM.md", "send: Sample, via: out,", "send: Sampel, via: out,");
    has(&run(&f), "E932", "sends Sampel via 'out', but its type 'Ifc::SamplePort' carries no Sampel");
}

#[test]
fn a_forwarded_payload_value_has_the_accepted_field_type() {
    // Mon forwards the accepted sample's boolean as the voltage of a Sample it sends back
    let mut f = base();
    edit(&mut f, "Ifc/SamplePort.md", "  - {name: trip,", "  - {name: echo, typedBy: Ifc::Sample, direction: in}\n  - {name: trip,");
    edit(&mut f, "P/MonM.md", "effect: {send: Trip, via: inp}", "effect: {send: Sample, via: inp, payload: {v: \"s.ok\", ok: \"s.ok\"}}");
    has(&run(&f), "E935", "payload 'v' of Sample is");
}
