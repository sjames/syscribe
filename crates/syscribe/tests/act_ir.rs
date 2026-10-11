//! TC-TRS-ACTEXEC-001 / REQ-TRS-ACTEXEC-001 (GH #276): lowering an ActionDef to the action-graph IR.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const B: &str = "---
type: ActionDef
name: B
parameters:
  - {name: x, direction: in}
  - {name: y, direction: out}
variables:
  - {name: tmp, value: \"0\"}
subActions:
  - {name: calc, kind: AssignmentAction, assign: \"y := x * 3\"}
---
";

const A: &str = "---
type: ActionDef
name: A
parameters:
  - {name: n, direction: in}
  - {name: result, direction: out}
variables:
  - {name: acc, value: \"0\"}
subActions:
  - {name: init, kind: AssignmentAction, assign: \"acc := n\"}
  - name: call
    kind: PerformAction
    typedBy: B
    parameters:
      - {name: x, value: acc}
  - {name: big, kind: AssignmentAction, assign: \"result := call.y\"}
  - {name: small, kind: AssignmentAction, assign: \"result := acc\"}
controlNodes:
  - {name: d, kind: DecisionNode}
successionConnections:
  - {after: init, before: call}
  - {after: call, before: d}
  - {after: d, before: big, guard: \"call.y > 10\"}
  - {after: d, before: small, guard: else}
---
";

fn model(files: &[(&str, &str)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-actir-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in files {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

fn compile(files: &[(&str, &str)], name: &str) -> (i32, String, serde_json::Value) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model(files)).args(["compile", name, "--json"]).output().unwrap();
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    (o.status.code().unwrap_or(-1), format!("{out}{}", String::from_utf8_lossy(&o.stderr)), serde_json::from_str(&out).unwrap_or(serde_json::Value::Null))
}

#[test]
fn nodes_edges_and_slots_are_flat_tables_in_document_order() {
    let (c, t, j) = compile(&[("A", A), ("B", B)], "A");
    assert_eq!(c, 0, "{t}");
    let d = &j["defs"][0];
    let names: Vec<_> = d["nodes"].as_array().unwrap().iter().map(|n| n["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["init", "call", "big", "small", "d"], "sub-actions then control nodes, in document order");
    let kinds: Vec<_> = d["nodes"].as_array().unwrap().iter().map(|n| n["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["assign", "perform", "assign", "assign", "decision"]);
    let e = d["edges"].as_array().unwrap();
    assert_eq!(e.len(), 4);
    assert_eq!((e[0]["from"].as_u64(), e[0]["to"].as_u64()), (Some(0), Some(1)));
    assert_eq!(e[2]["guard"]["src"], "call.y > 10");
    assert_eq!(e[3]["else"], true);
    assert_eq!(d["start"], serde_json::json!([0]), "the node with no incoming succession starts");
    let slots: Vec<_> = d["slots"].as_array().unwrap().iter().map(|s| (s["name"].as_str().unwrap(), s["kind"].as_str().unwrap())).collect();
    assert_eq!(slots, [("n", "in"), ("result", "out"), ("acc", "local")]);
}

#[test]
fn performed_actions_are_resolved_to_callee_ir_and_names_are_classified() {
    let (_, t, j) = compile(&[("A", A), ("B", B)], "A");
    assert_eq!(j["defs"].as_array().unwrap().len(), 2, "root first, then its callees: {t}");
    assert_eq!(j["defs"][1]["qname"], "B");
    let call = &j["defs"][0]["nodes"][1];
    assert_eq!(call["callee"], 1);
    assert_eq!(call["args"][0]["name"], "x");
    let g = &j["defs"][0]["edges"][2]["guard"]["refs"][0];
    assert_eq!((g["name"].as_str(), g["kind"].as_str()), (Some("call.y"), Some("pin")));
    assert_eq!(j["defs"][0]["nodes"][0]["assigns"][0]["slot"], 2);
}

#[test]
fn the_hash_is_stable_and_changes_with_any_def() {
    let (_, _, a) = compile(&[("A", A), ("B", B)], "A");
    let (_, _, b) = compile(&[("A", A), ("B", B)], "A");
    assert_eq!(a["hash"], b["hash"]);
    assert_eq!(a["hash"].as_str().unwrap().len(), 64);
    let (_, _, c) = compile(&[("A", A), ("B", &B.replace("x * 3", "x * 4"))], "A");
    assert_ne!(a["hash"], c["hash"], "a changed callee changes the hash");
}

#[test]
fn nodes_and_edges_carry_source_pointers() {
    let (_, _, j) = compile(&[("A", A), ("B", B)], "A");
    assert_eq!(j["defs"][0]["nodes"][1]["origin"]["path"], "subActions[1]");
    assert_eq!(j["defs"][0]["edges"][3]["origin"]["path"], "successionConnections[3]");
    assert_eq!(j["defs"][0]["nodes"][4]["origin"]["path"], "controlNodes[0]");
}

#[test]
fn lowering_diagnostics_fail_compilation() {
    let (c, t, _) = compile(&[("A", &A.replace("typedBy: B", "typedBy: Nope")), ("B", B)], "A");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("Nope"), "{t}");
    let (c, t, _) = compile(&[("A", &A.replace("before: call", "before: ghost")), ("B", B)], "A");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("ghost"), "{t}");
    let cyc = "---\ntype: ActionDef\nname: C\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: again, kind: PerformAction, typedBy: C}\n---\n";
    let (c, t, _) = compile(&[("C", cyc)], "C");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("recursive") || t.contains("cycle"), "{t}");
    let nested = "---\ntype: ActionDef\nname: N\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - name: i\n    kind: IfAction\n    condition: \"v == 0\"\n    then:\n      - {name: w, kind: AcceptAction, payload: Pkg::E}\n---\n";
    let (c, t, _) = compile(&[("N", nested)], "N");
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("AcceptAction"), "an accept inside a block cannot suspend: {t}");
}

#[test]
fn nested_blocks_and_other_node_kinds_are_lowered() {
    let y = "---
type: ActionDef
name: K
variables:
  - {name: i, value: \"0\"}
subActions:
  - name: loop
    kind: LoopAction
    loopKind: while
    condition: \"i < 3\"
    body:
      - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}
  - name: choose
    kind: IfAction
    condition: \"i == 3\"
    then:
      - {name: yes, kind: AssignmentAction, assign: \"i := 10\"}
    else:
      - {name: no, kind: AssignmentAction, assign: \"i := 20\"}
  - {name: out, kind: SendAction, payload: Pkg::Done, via: p}
  - {name: wait, kind: AcceptAction, payload: Pkg::Go, as: g}
  - {name: timeout, kind: AcceptAction, after: \"2 s\"}
  - {name: stop, kind: TerminateAction}
controlNodes:
  - {name: f, kind: ForkNode}
  - {name: j, kind: JoinNode}
  - {name: m, kind: MergeNode}
successionConnections:
  - {after: loop, before: choose}
---
";
    let (c, t, j) = compile(&[("K", y)], "K");
    assert_eq!(c, 0, "{t}");
    let n = &j["defs"][0]["nodes"];
    assert_eq!(n[0]["kind"], "loop");
    assert_eq!(n[0]["body"][0]["kind"], "assign");
    assert_eq!(n[1]["kind"], "if");
    assert_eq!(n[1]["then"][0]["name"], "yes");
    assert_eq!(n[1]["else"][0]["name"], "no");
    assert_eq!(n[2]["kind"], "send");
    assert_eq!(n[2]["event"], "p/Pkg::Done");
    assert_eq!(n[3]["kind"], "accept");
    assert_eq!(n[3]["event"], "Pkg::Go");
    assert_eq!(n[3]["bind"], "g");
    assert_eq!(n[4]["afterSeconds"], 2.0);
    assert_eq!(n[5]["kind"], "terminate");
    assert_eq!(n[6]["kind"], "fork");
    assert_eq!(n[7]["kind"], "join");
    assert_eq!(n[8]["kind"], "merge");
}
