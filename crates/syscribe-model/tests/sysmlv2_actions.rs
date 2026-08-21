//! Integration tests for mapping `action def`/`action` onto the native
//! `ActionDef`/`Action` schema (`REQ-TRS-SYSMLV2-019`).
//!
//! Two-layer, mirroring `sysmlv2_states.rs`: (a) raw YAML shape assertions
//! on `frontmatter.sub_actions`/`.control_nodes`/`.succession_connections`,
//! and (b) confirming the depth ceiling — `fork`/`join` control nodes carry
//! no recoverable body content, since the pinned parser itself discards it.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::ElementType;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-sysmlv2-actions-test-{}-{}",
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

#[test]
fn a_top_level_action_def_becomes_a_real_element_with_sub_actions() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Mission.sysml",
        "package Behavior {\n\
         action def MissionExecution {\n\
         action takeoff;\n\
         action navigate;\n\
         first takeoff then navigate;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Behavior::MissionExecution")
        .expect("MissionExecution should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::ActionDef));

    let subs = el.frontmatter.sub_actions.as_deref().unwrap_or(&[]);
    let names: Vec<&str> = subs.iter().filter_map(|s| s.get("name").and_then(|v| v.as_str())).collect();
    assert!(names.contains(&"takeoff"), "{names:#?}");
    assert!(names.contains(&"navigate"), "{names:#?}");
    for s in subs {
        assert_eq!(s.get("kind").and_then(|v| v.as_str()), Some("PerformAction"));
    }

    let succ = el.frontmatter.succession_connections.as_deref().unwrap_or(&[]);
    assert_eq!(succ.len(), 1, "{succ:#?}");
    assert_eq!(succ[0].get("after").and_then(|v| v.as_str()), Some("takeoff"));
    assert_eq!(succ[0].get("before").and_then(|v| v.as_str()), Some("navigate"));

    // Nested actions are inline data only, never separate elements.
    assert!(!elements.iter().any(|e| e.qualified_name.ends_with("::takeoff")));
    assert!(!elements.iter().any(|e| e.qualified_name.ends_with("::navigate")));

    let result = validate(&elements);
    assert_eq!(result.errors().count(), 0, "{:#?}", result.findings);
}

#[test]
fn if_action_recurses_for_real_into_then_and_else() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Mission.sysml",
        "package Behavior {\n\
         action def MissionExecution {\n\
         if windSpeed > 12.0 {\n\
         action abortMission;\n\
         } else {\n\
         action continueMission;\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Behavior::MissionExecution").unwrap();
    let subs = el.frontmatter.sub_actions.as_deref().unwrap();
    assert_eq!(subs.len(), 1, "{subs:#?}");
    let if_action = &subs[0];
    assert_eq!(if_action.get("kind").and_then(|v| v.as_str()), Some("IfAction"));
    assert!(if_action.get("condition").is_some());

    let then_list = if_action.get("then").and_then(|v| v.as_sequence()).expect("then present");
    assert_eq!(then_list[0].get("name").and_then(|v| v.as_str()), Some("abortMission"));
    let else_list = if_action.get("else").and_then(|v| v.as_sequence()).expect("else present");
    assert_eq!(else_list[0].get("name").and_then(|v| v.as_str()), Some("continueMission"));
}

#[test]
fn loop_action_recurses_for_real_into_its_body() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Mission.sysml",
        "package Behavior {\n\
         action def Takeoff {\n\
         while altitude < target {\n\
         action adjustThrottle;\n\
         }\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Behavior::Takeoff").unwrap();
    let subs = el.frontmatter.sub_actions.as_deref().unwrap();
    assert_eq!(subs.len(), 1, "{subs:#?}");
    let loop_action = &subs[0];
    assert_eq!(loop_action.get("kind").and_then(|v| v.as_str()), Some("LoopAction"));
    assert_eq!(loop_action.get("loopKind").and_then(|v| v.as_str()), Some("while"));
    assert!(loop_action.get("condition").is_some());
    let body = loop_action.get("body").and_then(|v| v.as_sequence()).expect("body present");
    assert_eq!(body[0].get("name").and_then(|v| v.as_str()), Some("adjustThrottle"));
}

#[test]
fn fork_and_join_become_flat_control_nodes_with_no_recoverable_body() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Mission.sysml",
        "package Behavior {\n\
         action def MissionExecution {\n\
         fork missionStart {\n\
         action takeoff;\n\
         }\n\
         join missionEnd;\n\
         action takeoff;\n\
         first missionStart then takeoff;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Behavior::MissionExecution").unwrap();
    let nodes = el.frontmatter.control_nodes.as_deref().unwrap_or(&[]);
    let fork = nodes
        .iter()
        .find(|n| n.get("kind").and_then(|v| v.as_str()) == Some("ForkNode"))
        .expect("a ForkNode control node");
    assert_eq!(fork.get("name").and_then(|v| v.as_str()), Some("missionStart"));
    // Only `name`/`kind` -- no recoverable body content (the parser itself
    // discards fork/join/decide/merge block bodies; see ADR-SYS-SYSMLV2-001's
    // addendum). Confirm no stray third key snuck in.
    assert_eq!(fork.as_mapping().unwrap().len(), 2, "{fork:#?}");

    let join = nodes
        .iter()
        .find(|n| n.get("kind").and_then(|v| v.as_str()) == Some("JoinNode"))
        .expect("a JoinNode control node");
    assert_eq!(join.get("name").and_then(|v| v.as_str()), Some("missionEnd"));
    assert_eq!(join.as_mapping().unwrap().len(), 2, "{join:#?}");

    // The `action takeoff;` inside the fork's own discarded block is
    // invisible (the parser gave us nothing to recover); the sibling
    // top-level `action takeoff;` is still a real subActions entry.
    let sub_action_names: Vec<&str> = el
        .frontmatter
        .sub_actions
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|s| s.get("name").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(sub_action_names, vec!["takeoff"], "{sub_action_names:#?}");
}

#[test]
fn a_nested_part_usage_inside_an_action_body_is_still_a_real_element() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Mission.sysml",
        "package Behavior {\n\
         action def MissionExecution {\n\
         part sensorRig;\n\
         action takeoff;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    assert!(
        elements.iter().any(|e| e.qualified_name == "SysML2Legacy::Behavior::MissionExecution::sensorRig"),
        "{:#?}",
        elements.iter().map(|e| &e.qualified_name).collect::<Vec<_>>()
    );
}

#[test]
fn a_top_level_standalone_action_usage_becomes_its_own_element() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Mission.sysml",
        "package Behavior {\n\
         action def Takeoff;\n\
         action initialClimb : Takeoff;\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements
        .iter()
        .find(|e| e.qualified_name == "SysML2Legacy::Behavior::initialClimb")
        .expect("initialClimb should be a real element");
    assert_eq!(el.frontmatter.element_type, Some(ElementType::Action));
}

#[test]
fn accept_and_send_actions_map_to_the_hand_authored_kind_vocabulary() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Mission.sysml",
        "package Behavior {\n\
         action def MissionExecution {\n\
         accept cmd : StartCmd;\n\
         send ack : AckCmd;\n\
         }\n\
         }\n",
    );

    let elements = walk_model(&root).unwrap();
    let el = elements.iter().find(|e| e.qualified_name == "SysML2Legacy::Behavior::MissionExecution").unwrap();
    let subs = el.frontmatter.sub_actions.as_deref().unwrap();
    let accept = subs.iter().find(|s| s.get("name").and_then(|v| v.as_str()) == Some("cmd")).expect("accept entry");
    assert_eq!(accept.get("kind").and_then(|v| v.as_str()), Some("AcceptAction"));
    assert_eq!(accept.get("payload").and_then(|v| v.as_str()), Some("StartCmd"));
    let send = subs.iter().find(|s| s.get("name").and_then(|v| v.as_str()) == Some("ack")).expect("send entry");
    assert_eq!(send.get("kind").and_then(|v| v.as_str()), Some("SendAction"));
    assert_eq!(send.get("payload").and_then(|v| v.as_str()), Some("AckCmd"));
}

#[test]
fn w080_sees_a_synthesized_action_defs_real_send_accept_sub_actions_via_a_sequence_diagram() {
    let root = tempdir();
    base_model(&root);
    write(
        &root,
        "SysML2Legacy/Mission.sysml",
        "package Behavior {\n\
         action def MissionExecution {\n\
         accept cmd : StartCmd;\n\
         send ack : AckCmd;\n\
         }\n\
         }\n",
    );
    // A Sequence diagram naming this ActionDef as `subject:` but only
    // covering one of its two message actions with an `edges:` entry -- W080
    // should fire for the uncovered one, exactly as it would for a
    // hand-authored ActionDef with the same subActions shape.
    write(
        &root,
        "Diagrams/Mission.md",
        "---\n\
         type: Diagram\n\
         name: MissionSeq\n\
         diagramKind: Sequence\n\
         subject: SysML2Legacy::Behavior::MissionExecution\n\
         edges:\n\
         \x20\x20- ref: SysML2Legacy::Behavior::MissionExecution::cmd\n\
         ---\n\
         \n\
         Mission sequence diagram.\n",
    );

    let elements = walk_model(&root).unwrap();
    let result = validate(&elements);
    let w080: Vec<_> = result.findings.iter().filter(|f| f.code == "W080").collect();
    assert_eq!(w080.len(), 1, "expected exactly one W080 for the uncovered 'ack' message action: {:#?}", result.findings);
    assert!(w080[0].message.contains("ack"), "{:#?}", w080[0]);
}
