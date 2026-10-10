//! TC-TRS-FAILNOTE-001 / GH #258.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(fail: bool) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-fnote-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let m = d.join("model");
    let w = |rel: &str, c: &str| {
        let p = m.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("S/SG-FN-001.md", "---\nid: SG-FN-001\ntype: SafetyGoal\nname: g\nstatus: approved\nasilLevel: B\n---\n\nGoal.\n");
    w("S/REQ-FN-001.md", "---\nid: REQ-FN-001\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\nreqClass: system\nderivedFromSafetyGoal: SG-FN-001\n---\n\nShall.\n");
    w("S/TC-FN-001.md", "---\nid: TC-FN-001\ntype: TestCase\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-FN-001]\ntestFunctions:\n  - function: t_lat\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    let body = if fail { "<failure message=\"latency 61 ms &gt; 50 ms\"/>" } else { "" };
    let x = d.join("r.xml");
    std::fs::write(&x, format!("<testsuite><testcase classname=\"C\" name=\"t_lat\" time=\"1.25\">{body}</testcase></testsuite>")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&m).args(["ingest-results", "--format", "junit"]).arg(&x).output().unwrap();
    assert!(o.status.success());
    m
}

fn run(m: &Path, args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(m).args(args).output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn trace_lists_the_failing_function_message_and_time() {
    let t = run(&model(true), &["trace", "REQ-FN-001"]);
    assert!(t.contains("Failing tests") && t.contains("t_lat") && t.contains("latency 61 ms > 50 ms") && t.contains("1.25"), "{t}");
    assert!(!run(&model(false), &["trace", "REQ-FN-001"]).contains("Failing tests"));
}

#[test]
fn safety_case_prints_and_serialises_the_details() {
    let m = model(true);
    let t = run(&m, &["safety-case"]);
    assert!(t.contains("Failure details") && t.contains("TC-FN-001") && t.contains("latency 61 ms > 50 ms"), "{t}");
    let j: serde_json::Value = serde_json::from_str(&run(&m, &["safety-case", "--format", "json"])).unwrap();
    let d = &j["failureDetails"][0];
    assert_eq!((d["testCase"].as_str(), d["function"].as_str(), d["message"].as_str()), (Some("TC-FN-001"), Some("t_lat"), Some("latency 61 ms > 50 ms")), "{j}");
    assert_eq!(d["time"], 1.25);
    let ok: serde_json::Value = serde_json::from_str(&run(&model(false), &["safety-case", "--format", "json"])).unwrap();
    assert_eq!(ok["failureDetails"].as_array().map(|a| a.len()), Some(0), "{ok}");
    assert!(!run(&model(false), &["safety-case"]).contains("Failure details"));
}

#[test]
fn control_characters_a_bare_failure_and_class_collisions_are_handled() {
    let d = std::env::temp_dir().join(format!("syscribe-fnote2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let m = d.join("model");
    let w = |rel: &str, c: &str| {
        let p = m.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("S/REQ-FN-001.md", "---\nid: REQ-FN-001\ntype: Requirement\nname: r\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
    w("S/TC-FN-001.md", "---\nid: TC-FN-001\ntype: TestCase\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-FN-001]\ntestFunctions:\n  - function: \"B#t\"\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    let x = d.join("r.xml");
    // A::t fails with an ANSI-laden message; B::t fails with no message and no time
    std::fs::write(&x, "<testsuite><testcase classname=\"A\" name=\"t\" time=\"2\"><failure message=\"boom \u{1b}[2K\u{7}A\"/></testcase><testcase classname=\"B\" name=\"t\"><failure/></testcase></testsuite>").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&m).args(["ingest-results", "--format", "junit"]).arg(&x).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let t = run(&m, &["trace", "REQ-FN-001"]);
    assert!(!t.contains('\u{1b}') && !t.contains('\u{7}'), "no raw control characters: {t:?}");
    assert!(!t.contains("boom"), "B#t must not inherit A's message: {t}");
    // A's own message is shown, sanitised
    std::fs::write(m.join("S/TC-FN-001.md"), std::fs::read_to_string(m.join("S/TC-FN-001.md")).unwrap().replace("B#t", "A#t")).unwrap();
    let t = run(&m, &["trace", "REQ-FN-001"]);
    assert!(t.contains("boom") && !t.contains('\u{1b}'), "{t:?}");
}
