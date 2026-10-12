//! GH #328: a machine expression that uses a parameter with no `value:`/`default:`
//! is checked in the base model with a stand-in value and the parameter is
//! reported once (W248), instead of a parse error at the `{`. With `--config`
//! binding the parameter, the machine is checked at the bound value.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(limit_default: &str, binding: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-mpnd-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&r);
    std::fs::create_dir_all(&r).unwrap();
    let w = |n: &str, c: &str| std::fs::write(r.join(n), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features.md", &format!("---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - name: Profile\n    id: FEAT-ND-PROFILE\n    mandatory: true\n    parameters:\n      - {{name: delayS, type: ScalarValues::Integer, default: 3, isRequired: true, bindingTime: load}}\n      - {{name: limit, type: ScalarValues::Integer, {limit_default}isRequired: true, bindingTime: load}}\n---\n"));
    w("CONF-ND-AA-001.md", &format!("---\ntype: Configuration\nid: CONF-ND-AA-001\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Profile: true\nparameterBindings:\n  Features::Profile.delayS: 5\n{binding}---\n"));
    // the parameter appears in a guard, an assignment and the guard of a second machine
    w("M.md", "---\ntype: StateDef\nname: M\nvariables:\n  - {name: level, value: \"0\"}\nsubStates:\n  - name: Wait\n    isInitial: true\n    transitions:\n      - {target: Armed, accept: {after: \"{{Features::Profile.delayS}} s\"}, assign: \"level := {{Features::Profile.limit}} + 1\"}\n  - name: Armed\n    transitions:\n      - {target: Over, accept: Check, guard: \"level > {{Features::Profile.limit}}\"}\n  - {name: Over, isFinal: true}\n---\n");
    w("N.md", "---\ntype: StateDef\nname: N\nvariables:\n  - {name: x, value: \"0\"}\nsubStates:\n  - {name: A, isInitial: true, transitions: [{target: B, accept: Go, guard: \"x < {{Features::Profile.limit}}\"}]}\n  - {name: B, isFinal: true}\n---\n");
    r
}

fn validate(root: &PathBuf, extra: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).arg("validate").args(extra).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn count(t: &str, code: &str) -> usize {
    t.lines().filter(|l| l.starts_with(&format!("| {code} |"))).count()
}

#[test]
fn a_parameter_without_a_default_is_one_finding_not_a_parse_error() {
    let t = validate(&model("", "  Features::Profile.limit: 10\n"), &[]);
    for c in ["E882", "E884", "W933", "E935"] {
        assert_eq!(count(&t, c), 0, "{c}: {t}");
    }
    assert_eq!(count(&t, "W248"), 1, "one finding for the parameter, however many uses: {t}");
    let l = t.lines().find(|l| l.starts_with("| W248 |")).unwrap();
    assert!(l.contains("Features.md") && l.contains("'Features::Profile.limit'") && l.contains("--config"), "{l}");
    assert!(l.contains("M, N"), "names both machines that use it: {l}");
    assert!(!l.contains("delayS"), "a parameter with a default is not reported: {l}");
}

#[test]
fn the_rest_of_the_expression_is_still_checked() {
    let root = model("", "  Features::Profile.limit: 10\n");
    let m = std::fs::read_to_string(root.join("M.md")).unwrap().replace("level > {{Features::Profile.limit}}", "levle > {{Features::Profile.limit}}");
    std::fs::write(root.join("M.md"), m).unwrap();
    let t = validate(&root, &[]);
    assert!(t.lines().any(|l| l.starts_with("| W933 |") && l.contains("'levle'")), "{t}");
}

#[test]
fn with_config_the_machine_is_checked_at_the_bound_value() {
    let root = model("", "  Features::Profile.limit: 10\n");
    let t = validate(&root, &["--config", "CONF-ND-AA-001"]);
    assert_eq!(count(&t, "W248"), 0, "{t}");
    assert_eq!(count(&t, "W933"), 0, "{t}");
    // a bound value that is not an expression is reported against the machine
    let root = model("", "  Features::Profile.limit: \"ten\"\n");
    let t = validate(&root, &["--config", "CONF-ND-AA-001"]);
    assert!(t.lines().any(|l| l.starts_with("| W933 |") && l.contains("'ten'")), "the bound value is what gets checked: {t}");
}

#[test]
fn a_parameter_with_a_default_keeps_todays_behaviour() {
    let t = validate(&model("default: 10, ", ""), &[]);
    for c in ["W248", "E882", "E884", "W933", "E935"] {
        assert_eq!(count(&t, c), 0, "{c}: {t}");
    }
}
