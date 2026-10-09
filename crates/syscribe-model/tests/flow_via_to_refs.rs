//! GH #207: `via:`/`to:` chains on send/accept steps and the endpoints of
//! `flowConnections:`/`successionConnections:` on structural elements are
//! resolved end to end through the full validator (E128, E127, W056).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::validator::{validate_with_config, ValidationResult};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "syscribe-flow-via-test-{}-{}",
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

fn validate(root: &Path) -> ValidationResult {
    let elements = walk_model(root).unwrap();
    validate_with_config(&elements, &ValidateConfig::with_model_root(root.to_path_buf()))
}

fn count(r: &ValidationResult, code: &str) -> usize {
    r.findings.iter().filter(|f| f.code == code).count()
}

fn base(root: &Path) {
    write(root, "_index.md", "---\ntype: Package\nname: M\n---\n");
    write(root, "Ctl.md", "---\ntype: PartDef\nfeatures:\n  - {name: cmdPort, type: Port}\n---\n");
}

#[test]
fn dangling_send_via_and_to_are_reported_once_each() {
    let root = tempdir();
    base(&root);
    write(
        &root,
        "Act.md",
        "---\ntype: ActionDef\nsubActions:\n  - {name: ok, kind: SendAction, via: cmdPort, to: Ctl}\n  - {name: bad, kind: SendAction, via: noSuchPort, to: noSuchPart}\n---\n",
    );
    let r = validate(&root);
    assert_eq!(count(&r, "E128"), 2, "{:?}", r.findings);
    let msgs: Vec<_> = r.findings.iter().filter(|f| f.code == "E128").map(|f| f.message.clone()).collect();
    assert!(msgs.iter().any(|m| m.contains("noSuchPort")));
    assert!(msgs.iter().any(|m| m.contains("noSuchPart")));
}

#[test]
fn accept_via_in_state_transitions_is_checked() {
    let root = tempdir();
    base(&root);
    write(
        &root,
        "SM.md",
        "---\ntype: StateDef\nsubStates:\n  - name: a\n    transitions:\n      - {target: b, accept: {payload: X, via: cmdPort}}\n      - {target: b, accept: {payload: X, via: ghost}}\n  - name: b\n---\n",
    );
    let r = validate(&root);
    assert_eq!(count(&r, "E128"), 1, "{:?}", r.findings);
}

#[test]
fn structural_flow_and_succession_endpoints_reuse_e127() {
    let root = tempdir();
    base(&root);
    write(
        &root,
        "Sys.md",
        "---\ntype: PartDef\nfeatures:\n  - {name: c, type: Part, typedBy: Ctl}\nflowConnections:\n  - {from: c.cmdPort, to: c.cmdPort}\n  - {from: ghost.p, to: c.cmdPort}\n  - {from: c.cmdPort, to: c.missing}\nsuccessionConnections:\n  - {after: c, before: c}\n  - {after: c, before: nope}\n---\n",
    );
    let r = validate(&root);
    assert_eq!(count(&r, "E127"), 2, "{:?}", r.findings);
    assert_eq!(count(&r, "W056"), 1, "{:?}", r.findings);
    assert_eq!(count(&r, "E128"), 0);
}

#[test]
fn valid_references_stay_clean() {
    let root = tempdir();
    base(&root);
    write(
        &root,
        "Act.md",
        "---\ntype: ActionDef\nsubActions:\n  - {name: s, kind: SendAction, via: cmdPort, to: Ctl}\n  - {name: a, kind: AcceptAction, via: Ctl::cmdPort}\n---\n",
    );
    let r = validate(&root);
    assert_eq!(count(&r, "E128"), 0, "{:?}", r.findings);
}
