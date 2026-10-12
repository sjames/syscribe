//! GH #330: `models:` links an executable part to the architecture element it is a behavioural model of, and the
//! checks it enables: E990 (target), E991 (executable port ↔ architecture port), W990 (architecture port with no
//! executable counterpart), W991 (connection correspondence), W992 (a TestCase whose subject cannot be evidence for a
//! requirement it verifies). All of it is dormant until some part declares `models:`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// The base model: a software `Manager` and `Relay` joined in `Board`, an executable `Monitor` and `RelayM` that model
/// them, joined in `Chain`, and a TestCase whose subject is `Chain`. It validates with no `models:` finding.
fn base() -> BTreeMap<&'static str, String> {
    let mut m = BTreeMap::new();
    let mut put = |k: &'static str, v: &str| {
        m.insert(k, v.to_string());
    };
    put("_index.md", "---\ntype: Package\nname: Root\n---\n");
    put("Reqs/REQ-MO-001.md", "---\ntype: Requirement\nid: REQ-MO-001\nname: trip\nstatus: draft\nreqDomain: software\n---\n\nThe manager shall trip.\n");
    put("Reqs/REQ-MO-002.md", "---\ntype: Requirement\nid: REQ-MO-002\nname: open\nstatus: draft\nreqDomain: software\n---\n\nThe relay driver shall open.\n");
    put(
        "Arch/Manager.md",
        "---\ntype: PartDef\nname: Manager\ndomain: software\nsatisfies: [REQ-MO-001]\nfeatures:\n  - {name: samples, type: Port, direction: in}\n  - {name: trip, type: Port, direction: out}\n  - {name: profile, type: Port, direction: in}\n---\n",
    );
    put("Arch/Relay.md", "---\ntype: PartDef\nname: Relay\ndomain: software\nsatisfies: [REQ-MO-002]\nfeatures:\n  - {name: cmd, type: Port, direction: in}\n---\n");
    put(
        "Arch/Board.md",
        "---\ntype: PartDef\nname: Board\ndomain: software\nfeatures:\n  - {name: mgr, typedBy: Arch::Manager}\n  - {name: relay, typedBy: Arch::Relay}\nconnections:\n  - {from: mgr.trip, to: relay.cmd}\n---\n",
    );
    put(
        "Exec/Monitor.md",
        "---\ntype: PartDef\nname: Monitor\nmodels: [Arch::Manager]\nabstracts: [profile]\nportMap: {out: trip}\nfeatures:\n  - {name: samples, type: Port, direction: in}\n  - {name: out, type: Port, direction: out}\n---\n",
    );
    put("Exec/RelayM.md", "---\ntype: PartDef\nname: RelayM\nmodels: [Arch::Relay]\nfeatures:\n  - {name: cmd, type: Port, direction: in}\n---\n");
    put(
        "Exec/Chain.md",
        "---\ntype: PartDef\nname: Chain\nfeatures:\n  - {name: mon, typedBy: Exec::Monitor}\n  - {name: rel, typedBy: Exec::RelayM}\nconnections:\n  - {from: mon.out, to: rel.cmd}\n---\n",
    );
    put("TC-MO-001.md", "---\ntype: TestCase\nid: TC-MO-001\nname: t\nstatus: active\ntestLevel: L3\nsubject: Exec::Chain\nverifies: [REQ-MO-001, REQ-MO-002]\n---\n");
    m
}

fn write(files: &BTreeMap<&'static str, String>) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-models-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&r);
    for (k, v) in files {
        let p = r.join(k);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, v).unwrap();
    }
    r
}

fn run(root: &PathBuf, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn validate(files: &BTreeMap<&'static str, String>) -> String {
    run(&write(files), &[])
}

fn lines<'a>(out: &'a str, code: &str) -> Vec<&'a str> {
    out.lines().filter(|l| l.starts_with(&format!("| {code} |"))).collect()
}

fn none_of_ours(out: &str) {
    for c in ["E990", "E991", "W990", "W991", "W992"] {
        assert!(lines(out, c).is_empty(), "unexpected {c}:\n{out}");
    }
}

fn edit(files: &mut BTreeMap<&'static str, String>, k: &'static str, from: &str, to: &str) {
    let v = files.get_mut(k).unwrap();
    assert!(v.contains(from), "{k} has no {from:?}");
    *v = v.replace(from, to);
}

#[test]
fn the_base_model_is_clean() {
    let out = validate(&base());
    none_of_ours(&out);
    assert!(lines(&out, "W047").is_empty(), "the new keys are known fields:\n{out}");
}

#[test]
fn a_model_with_no_models_link_gets_no_new_finding() {
    let mut f = base();
    // Remove every `models:`/`portMap:`: the same parts, ports and TestCase subject, and nothing new fires — not even
    // W992, although the subject now models nothing.
    edit(&mut f, "Exec/Monitor.md", "models: [Arch::Manager]\nabstracts: [profile]\nportMap: {out: trip}\n", "");
    edit(&mut f, "Exec/RelayM.md", "models: [Arch::Relay]\n", "");
    none_of_ours(&validate(&f));
}

#[test]
fn e990_unresolved_and_non_architecture_targets() {
    let mut f = base();
    edit(&mut f, "Exec/RelayM.md", "models: [Arch::Relay]", "models: [Arch::Rleay]");
    let out = validate(&f);
    assert!(lines(&out, "E990").iter().any(|l| l.contains("`models: Arch::Rleay` does not resolve")), "{out}");

    let mut f = base();
    edit(&mut f, "Exec/RelayM.md", "models: [Arch::Relay]", "models: [Exec::Chain]");
    let out = validate(&f);
    assert!(lines(&out, "E990").iter().any(|l| l.contains("not an architecture element")), "{out}");
}

#[test]
fn e991_port_with_no_counterpart_and_bad_port_map_key() {
    let mut f = base();
    edit(&mut f, "Exec/Monitor.md", "portMap: {out: trip}", "portMap: {out: trip, oot: trip}");
    edit(&mut f, "Exec/Monitor.md", "  - {name: samples, type: Port, direction: in}\n", "  - {name: samples, type: Port, direction: in}\n  - {name: debug, type: Port, direction: out}\n");
    let out = validate(&f);
    let e = lines(&out, "E991");
    assert!(e.iter().any(|l| l.contains("`portMap:` key 'oot' is not a port")), "{out}");
    assert!(e.iter().any(|l| l.contains("port 'debug' corresponds to no port of 'Arch::Manager'")), "{out}");
    // `null` says "no counterpart, on purpose".
    edit(&mut f, "Exec/Monitor.md", "portMap: {out: trip, oot: trip}", "portMap: {out: trip, debug: null}");
    none_of_ours(&validate(&f));
}

#[test]
fn e991_direction_mismatch_is_not_also_w990() {
    let mut f = base();
    edit(&mut f, "Exec/Monitor.md", "{name: samples, type: Port, direction: in}", "{name: samples, type: Port, direction: out}");
    let out = validate(&f);
    assert!(lines(&out, "E991").iter().any(|l| l.contains("port 'samples' is `out` but its counterpart 'Arch::Manager.samples' is `in`")), "{out}");
    assert!(lines(&out, "W990").is_empty(), "{out}");
}

#[test]
fn w990_uncovered_architecture_port_unless_abstracted() {
    let mut f = base();
    edit(&mut f, "Exec/Monitor.md", "abstracts: [profile]\n", "");
    let out = validate(&f);
    let w = lines(&out, "W990");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("port 'profile' of 'Arch::Manager' has no counterpart in 'Exec::Monitor'"), "{out}");
}

#[test]
fn many_to_one_ports_are_checked_on_the_union() {
    let mut f = base();
    // Two machines model one element: Monitor covers samples/trip, Profiler covers profile. Neither abstracts it.
    edit(&mut f, "Exec/Monitor.md", "abstracts: [profile]\n", "");
    f.insert("Exec/Profiler.md", "---\ntype: PartDef\nname: Profiler\nmodels: [Arch::Manager]\nfeatures:\n  - {name: profile, type: Port, direction: in}\n---\n".into());
    none_of_ours(&validate(&f));
}

#[test]
fn one_to_many_two_instances_distinguished_by_usage_port_map() {
    let mut f = base();
    // Sensing has two output chains; one executable Sensor part models it, twice.
    f.insert(
        "Arch/Sensing.md",
        "---\ntype: PartDef\nname: Sensing\ndomain: software\nfeatures:\n  - {name: sig, type: Port, direction: in}\n  - {name: adcOut, type: Port, direction: out}\n  - {name: hwOut, type: Port, direction: out}\n---\n".into(),
    );
    f.insert("Exec/Sensor.md", "---\ntype: PartDef\nname: Sensor\nmodels: [Arch::Sensing]\nportMap: {inp: sig, out: adcOut}\nfeatures:\n  - {name: inp, type: Port, direction: in}\n  - {name: out, type: Port, direction: out}\n---\n".into());
    f.insert(
        "Exec/Bench.md",
        "---\ntype: PartDef\nname: Bench\nfeatures:\n  - {name: sw, typedBy: Exec::Sensor}\n  - {name: hw, typedBy: Exec::Sensor, portMap: {inp: sig, out: hwOut}}\n---\n".into(),
    );
    none_of_ours(&validate(&f));
    // Without the usage-level map, hwOut is uncovered.
    edit(&mut f, "Exec/Bench.md", ", portMap: {inp: sig, out: hwOut}", "");
    let out = validate(&f);
    assert!(lines(&out, "W990").iter().any(|l| l.contains("port 'hwOut' of 'Arch::Sensing'")), "{out}");
}

#[test]
fn a_port_map_list_stands_for_several_architecture_ports() {
    let mut f = base();
    edit(&mut f, "Arch/Manager.md", "  - {name: trip, type: Port, direction: out}\n", "  - {name: trip, type: Port, direction: out}\n  - {name: tripB, type: Port, direction: out}\n");
    let out = validate(&f);
    assert!(lines(&out, "W990").iter().any(|l| l.contains("port 'tripB'")), "{out}");
    edit(&mut f, "Exec/Monitor.md", "portMap: {out: trip}", "portMap: {out: [trip, tripB]}");
    none_of_ours(&validate(&f));
}

#[test]
fn w991_executable_connection_with_no_architecture_counterpart() {
    let mut f = base();
    edit(&mut f, "Arch/Board.md", "connections:\n  - {from: mgr.trip, to: relay.cmd}\n", "");
    let out = validate(&f);
    let w = lines(&out, "W991");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("connection mon.out → rel.cmd of 'Exec::Chain' has no counterpart in the architecture"), "{out}");
}

#[test]
fn w991_architecture_connection_with_no_executable_counterpart() {
    let mut f = base();
    edit(&mut f, "Exec/Chain.md", "connections:\n  - {from: mon.out, to: rel.cmd}\n", "");
    let out = validate(&f);
    let w = lines(&out, "W991");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("architecture connection 'Arch::Manager.trip' → 'Arch::Relay.cmd' (in 'Arch::Board') has no counterpart in 'Exec::Chain'"), "{out}");
}

#[test]
fn w991_skips_a_connection_across_the_hardware_software_boundary() {
    let mut f = base();
    // §12.6: hardware and software meet through allocation, not a connection.
    edit(&mut f, "Arch/Relay.md", "domain: software", "domain: hardware");
    edit(&mut f, "Reqs/REQ-MO-002.md", "reqDomain: software", "reqDomain: hardware");
    edit(&mut f, "Arch/Board.md", "connections:\n  - {from: mgr.trip, to: relay.cmd}\n", "");
    edit(&mut f, "Arch/Board.md", "domain: software", "domain: system");
    none_of_ours(&validate(&f));
}

#[test]
fn w992_subject_contains_no_part_that_models_the_satisfier() {
    let mut f = base();
    f.insert("Exec/OnlyMon.md", "---\ntype: PartDef\nname: OnlyMon\nfeatures:\n  - {name: mon, typedBy: Exec::Monitor}\n---\n".into());
    edit(&mut f, "TC-MO-001.md", "subject: Exec::Chain", "subject: Exec::OnlyMon");
    let out = validate(&f);
    let w = lines(&out, "W992");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("TC-MO-001 verifies REQ-MO-002, satisfied by 'Arch::Relay', but its subject 'Exec::OnlyMon' contains no part that models it"), "{out}");
}

#[test]
fn w992_a_requirement_named_in_abstracts_is_not_evidenced() {
    let mut f = base();
    edit(&mut f, "Exec/RelayM.md", "models: [Arch::Relay]\n", "models: [Arch::Relay]\nabstracts: [REQ-MO-002]\n");
    let out = validate(&f);
    let w = lines(&out, "W992");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("name REQ-MO-002 in `abstracts:`"), "{out}");
}

#[test]
fn the_link_appears_in_links_follow_why_trace_impact_and_suspect() {
    let root = write(&base());
    let out = run(&root, &["links", "Exec::Monitor"]);
    assert!(out.contains("| models | Arch::Manager | PartDef |"), "{out}");
    let out = run(&root, &["follow", "Arch::Manager", "modeledBy"]);
    assert!(out.contains("Exec::Monitor"), "{out}");
    let out = run(&root, &["why", "Exec::Monitor"]);
    assert!(out.contains("## Models") && out.contains("## Requirements of the modelled elements") && out.contains("REQ-MO-001"), "{out}");
    let out = run(&root, &["trace", "REQ-MO-001"]);
    assert!(out.contains("| Arch::Manager | Exec::Monitor |"), "{out}");
    let out = run(&root, &["impact", "Arch::Manager"]);
    assert!(out.contains("Exec::Monitor") && out.contains("modeledBy"), "{out}");
    let out = run(&root, &["suspect", "list"]);
    assert!(out.contains("Arch::Manager") && out.contains("models"), "{out}");
}

#[test]
fn a_port_name_in_abstracts_is_not_a_requirement_with_that_display_name() {
    let mut f = base();
    edit(&mut f, "Reqs/REQ-MO-001.md", "name: trip", "name: profile");
    none_of_ours(&validate(&f));
}

#[test]
fn an_inherited_models_link_checks_the_subtype_ports() {
    let mut f = base();
    f.insert(
        "Exec/MonSub.md",
        "---\ntype: PartDef\nname: MonSub\nsupertype: Exec::Monitor\nfeatures:\n  - {name: debug, type: Port, direction: out}\n---\n".into(),
    );
    let out = validate(&f);
    assert!(lines(&out, "E991").iter().any(|l| l.contains("'Exec::MonSub': port 'debug' corresponds to no port of 'Arch::Manager'")), "{out}");
    assert!(lines(&out, "E990").is_empty(), "the target is reported where `models:` is written, not again: {out}");
}

#[test]
fn one_to_many_needs_no_definition_level_port_map() {
    let mut f = base();
    f.insert(
        "Arch/Sensing.md",
        "---\ntype: PartDef\nname: Sensing\ndomain: software\nfeatures:\n  - {name: sig, type: Port, direction: in}\n  - {name: adcOut, type: Port, direction: out}\n  - {name: hwOut, type: Port, direction: out}\n---\n".into(),
    );
    f.insert("Exec/Sensor.md", "---\ntype: PartDef\nname: Sensor\nmodels: [Arch::Sensing]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n  - {name: out, type: Port, direction: out}\n---\n".into());
    f.insert(
        "Exec/Bench.md",
        "---\ntype: PartDef\nname: Bench\nfeatures:\n  - {name: sw, typedBy: Exec::Sensor, portMap: {inp: sig, out: adcOut}}\n  - {name: hw, typedBy: Exec::Sensor, portMap: {inp: sig, out: hwOut}}\n---\n".into(),
    );
    none_of_ours(&validate(&f));
}

#[test]
fn an_architecture_usage_typed_by_a_part_matches_its_definition() {
    let mut f = base();
    f.insert("Arch/TheMgr.md", "---\ntype: Part\nname: TheMgr\ndomain: software\ntypedBy: Arch::Manager\n---\n".into());
    edit(&mut f, "Arch/Board.md", "{name: mgr, typedBy: Arch::Manager}", "{name: mgr, typedBy: Arch::TheMgr}");
    none_of_ours(&validate(&f));
}

#[test]
fn a_requirement_satisfied_by_a_supertype_is_exercised_through_the_specialisation() {
    let mut f = base();
    f.insert("Arch/MgrBase.md", "---\ntype: PartDef\nname: MgrBase\ndomain: software\nisAbstract: true\nsatisfies: [REQ-MO-001]\n---\n".into());
    edit(&mut f, "Arch/Manager.md", "satisfies: [REQ-MO-001]\n", "supertype: Arch::MgrBase\n");
    none_of_ours(&validate(&f));
}

#[test]
fn a_subtype_composite_with_nothing_of_its_own_does_not_repeat_w991() {
    let mut f = base();
    edit(&mut f, "Arch/Board.md", "connections:\n  - {from: mgr.trip, to: relay.cmd}\n", "");
    f.insert("Exec/ChainSub.md", "---\ntype: PartDef\nname: ChainSub\nsupertype: Exec::Chain\n---\n".into());
    let out = validate(&f);
    let w = lines(&out, "W991");
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("of 'Exec::Chain'"), "{out}");
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &p);
        } else {
            std::fs::copy(e.path(), p).unwrap();
        }
    }
}

#[test]
fn grid_tie_regression_a_protection_test_listing_unmodelled_requirements_is_reported() {
    // GH #330 acceptance: TC-GTI-PROT-IN-001 once listed REQ-GTI-SYS-037 (10 µs overcurrent trip) and REQ-GTI-HW-002
    // (relay weld detection), which the executable chain does not model.
    let gti = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_gti");
    let root = std::env::temp_dir().join(format!("syscribe-models-gti-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    copy_dir(&gti, &root);
    let tc = root.join("Verification/TC-GTI-PROT-IN-001.md");
    let t = std::fs::read_to_string(&tc).unwrap().replacen("  - REQ-GTI-SYS-030\n", "  - REQ-GTI-SYS-030\n  - REQ-GTI-SYS-037\n  - REQ-GTI-HW-002\n", 1);
    std::fs::write(&tc, t).unwrap();
    let out = run(&root, &[]);
    let _ = std::fs::remove_dir_all(&root);
    let w = lines(&out, "W992");
    assert_eq!(w.len(), 2, "{out}");
    assert!(w.iter().any(|l| l.contains("REQ-GTI-SYS-037")) && w.iter().any(|l| l.contains("REQ-GTI-HW-002")), "{out}");
}
