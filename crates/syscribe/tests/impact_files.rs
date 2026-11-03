//! TC-TRS-IMPACTFILES-001 (GH #434): `impact --files` / `--since`, against a throwaway git repository.

mod common;
use common::Mcp;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Repo {
    root: PathBuf,
}
impl Repo {
    fn new() -> Repo {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-impf-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "t@example.com"]);
        r.git(&["config", "user.name", "T"]);
        r.git(&["config", "commit.gpgsign", "false"]);
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r.write("model/REQ-IMP-001.md", "---\nid: REQ-IMP-001\ntype: Requirement\nname: Sched\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nThe scheduler shall run tasks.\n");
        r.write("model/Sched.md", "---\ntype: PartDef\nname: Sched\ndomain: software\nsatisfies: [REQ-IMP-001]\nimplementedBy: [repo:src/sched.c]\n---\n\nScheduler.\n");
        r.write("model/Driver.md", "---\ntype: PartDef\nname: Driver\ndomain: software\nimplementedBy: [repo:src/driver/]\n---\n\nDriver, a whole directory.\n");
        r.write("model/TC-IMP-001.md", "---\nid: TC-IMP-001\ntype: TestCase\nname: Sched test\nstatus: active\ntestLevel: L2\nverifies: [REQ-IMP-001]\nsourceFile: repo:tests/test_sched.c\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
        r.write("model/PI-IMP-001.md", "---\nid: PI-IMP-001\ntype: PlanningItem\nname: Doc work\nstatus: done\nachieves: [REQ-IMP-001]\nevidence:\n  - path: repo:docs/spec.md\n---\n\nx\n");
        r.write("src/sched.c", "int sched(void) { return 0; }\n");
        r.write("src/driver/uart.c", "int uart(void) { return 0; }\n");
        r.write("tests/test_sched.c", "void test(void) {}\n");
        r.write("docs/spec.md", "# spec\n");
        r.write("docs/notes.txt", "notes\n");
        r.commit("base");
        r
    }
    fn model(&self) -> PathBuf {
        self.root.join("model")
    }
    fn git(&self, args: &[&str]) {
        let o = Command::new("git").arg("-C").arg(&self.root).args(args).output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    }
    fn write(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn commit(&self, msg: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
    }
    /// Runs from the repository root, so relative `--files` paths are repo-relative.
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(&self.root).arg("-m").arg(self.model()).arg("impact").args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
    fn report(&self, args: &[&str]) -> Value {
        let mut a = args.to_vec();
        a.extend(["--format", "json"]);
        let (out, err, code) = self.run(&a);
        assert_eq!(code, 0, "{err}");
        serde_json::from_str::<Value>(&out).unwrap_or_else(|e| panic!("{e}: {out}"))["report"].clone()
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn ids(v: &Value) -> Vec<String> {
    v.as_array().unwrap().iter().map(|a| a["id"].as_str().unwrap().to_string()).collect()
}
fn find<'a>(v: &'a Value, id: &str) -> &'a Value {
    v.as_array().unwrap().iter().find(|a| a["id"] == id).unwrap_or_else(|| panic!("{id} not in {v}"))
}

#[test]
fn a_changed_source_file_lists_its_unit_the_requirement_it_satisfies_and_the_tests_that_verify_it() {
    let r = Repo::new();
    let rep = r.report(&["--files", "src/sched.c"]);
    assert_eq!(rep["mapped"][0]["file"], "src/sched.c");
    assert_eq!(rep["mapped"][0]["elements"][0]["id"], "Sched");
    assert_eq!(rep["mapped"][0]["elements"][0]["via"], "implementedBy");
    let a = &rep["affected"];
    assert_eq!(find(a, "Sched")["distance"], 0);
    assert_eq!(find(a, "REQ-IMP-001")["distance"], 1);
    assert_eq!(find(a, "REQ-IMP-001")["via"], "satisfies");
    assert_eq!(find(a, "TC-IMP-001")["distance"], 2);
    assert!(rep["unmapped"].as_array().unwrap().is_empty());
    // Text output groups by element type.
    let (out, _e, code) = r.run(&["--files", "src/sched.c"]);
    assert_eq!(code, 0);
    assert!(out.contains("src/sched.c  →  Sched (PartDef, implementedBy)") && out.contains("Requirement (1)") && out.contains("TestCase (1)"), "{out}");
}

#[test]
fn files_that_map_to_nothing_are_listed_not_ignored() {
    let r = Repo::new();
    let rep = r.report(&["--files", "docs/notes.txt", "src/sched.c"]);
    assert_eq!(rep["unmapped"], json!(["docs/notes.txt"]));
    assert_eq!(rep["mapped"].as_array().unwrap().len(), 1);
    let (out, _e, _c) = r.run(&["--files", "docs/notes.txt"]);
    assert!(out.contains("Not mapped to any model element (1)") && out.contains("docs/notes.txt") && out.contains("No model elements affected"), "{out}");
}

#[test]
fn every_binding_kind_maps_a_file() {
    let r = Repo::new();
    // sourceFile → the TestCase; evidence path → the PlanningItem; a directory implementedBy covers files below it.
    assert_eq!(r.report(&["--files", "tests/test_sched.c"])["mapped"][0]["elements"][0], json!({"id": "TC-IMP-001", "qualifiedName": "TC-IMP-001", "type": "TestCase", "via": "sourceFile"}));
    assert_eq!(r.report(&["--files", "docs/spec.md"])["mapped"][0]["elements"][0]["via"], "evidence");
    let d = r.report(&["--files", "src/driver/uart.c"]);
    assert_eq!(d["mapped"][0]["elements"][0]["id"], "Driver");
    // A model file maps to its own element.
    let m = r.report(&["--files", "model/REQ-IMP-001.md"]);
    assert_eq!(m["mapped"][0]["elements"][0]["via"], "file");
    // A deleted file still maps (the binding is by path, not by existence).
    std::fs::remove_file(r.root.join("src/sched.c")).unwrap();
    assert_eq!(r.report(&["--files", "src/sched.c"])["mapped"][0]["elements"][0]["id"], "Sched");
}

#[test]
fn since_marks_what_changed_and_flags_unreviewed_requirements_and_tests() {
    let r = Repo::new();
    r.write("src/sched.c", "int sched(void) { return 1; }\n"); // code changed
    r.write("model/REQ-IMP-001.md", "---\nid: REQ-IMP-001\ntype: Requirement\nname: Sched\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\nThe scheduler shall run tasks, fast.\n"); // requirement updated
    r.write("brand_new.txt", "untracked\n");
    let rep = r.report(&["--since", "HEAD"]);
    assert!(rep["changedFiles"].as_array().unwrap().iter().any(|f| f == "brand_new.txt"), "untracked files count: {rep}");
    let a = &rep["affected"];
    assert_eq!(find(a, "REQ-IMP-001")["changed"], true);
    assert_eq!(find(a, "Sched")["changed"], false, "its file is unchanged; its implementation changed");
    assert_eq!(find(a, "TC-IMP-001")["changed"], false);
    assert_eq!(ids(&rep["unreviewed"]), ["TC-IMP-001"], "the test was not touched along with the code: {rep}");
    assert!(rep["unmapped"].as_array().unwrap().iter().any(|f| f == "brand_new.txt"));

    // The gate: exit 2 while the test is untouched, 0 once it moves with the code.
    let (_o, err, code) = r.run(&["--since", "HEAD", "--fail-on-unreviewed"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("not changed"), "{err}");
    r.write("model/TC-IMP-001.md", &std::fs::read_to_string(r.model().join("TC-IMP-001.md")).unwrap().replace("name: Sched test", "name: Sched test, updated"));
    let (_o, err, code) = r.run(&["--since", "HEAD", "--fail-on-unreviewed"]);
    assert_eq!(code, 0, "{err}");
    // Without the flag the exit status is 0 whatever is unreviewed.
    r.write("src/driver/uart.c", "changed\n");
    assert_eq!(r.run(&["--since", "HEAD"]).2, 0);
}

#[test]
fn depth_and_direction_control_the_reach() {
    let r = Repo::new();
    let shallow = r.report(&["--files", "src/sched.c", "--depth", "1"]);
    assert!(ids(&shallow["affected"]).contains(&"REQ-IMP-001".to_string()) && !ids(&shallow["affected"]).contains(&"TC-IMP-001".to_string()));
    let up = r.report(&["--files", "src/sched.c", "--direction", "upstream", "--depth", "5"]);
    assert!(ids(&up["affected"]).contains(&"REQ-IMP-001".to_string()) && !ids(&up["affected"]).contains(&"TC-IMP-001".to_string()), "a test is downstream of a requirement");
    let down = r.report(&["--files", "src/sched.c", "--direction", "downstream", "--depth", "5"]);
    assert_eq!(ids(&down["affected"]), ["Sched"], "nothing depends on the unit");
}

#[test]
fn usage_errors_exit_one() {
    let r = Repo::new();
    for args in [
        vec!["--since", "no-such-rev"],
        vec!["--files"],
        vec!["REQ-IMP-001", "--files", "src/sched.c"],
        vec!["--files", "src/sched.c", "--format", "dot"],
        vec!["--files", "src/sched.c", "--direction", "sideways"],
        vec!["--files", "src/sched.c", "--depth", "x"],
        vec!["--bogus"],
    ] {
        let (_o, e, code) = r.run(&args);
        assert_eq!(code, 1, "{args:?}: {e}");
        assert!(!e.is_empty(), "{args:?} explains itself");
    }
    // Naming one element still works as before.
    let (out, _e, code) = r.run(&["REQ-IMP-001"]);
    assert_eq!(code, 0);
    assert!(out.contains("Impact of REQ-IMP-001"), "{out}");
}

#[test]
fn outside_a_git_repository_files_mode_is_refused() {
    let d = std::env::temp_dir().join(format!("syscribe-impf-nogit-{}", std::process::id())).join("model");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&d).args(["impact", "--files", "x"]).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("not inside a git repository"));
}

#[test]
fn the_mcp_impact_tool_takes_files_and_since() {
    let r = Repo::new();
    let mut mcp = Mcp::start(&r.model());
    mcp.initialize();
    let res = mcp.call_tool("impact", json!({"files": ["src/sched.c"]}));
    let rep = &res["report"];
    assert_eq!(rep["mapped"][0]["elements"][0]["id"], "Sched");
    assert!(ids(&rep["affected"]).contains(&"TC-IMP-001".to_string()));

    r.write("src/sched.c", "int sched(void) { return 2; }\n");
    let since = mcp.call_tool("impact", json!({"since": "HEAD", "depth": 1}));
    assert!(ids(&since["report"]["affected"]).contains(&"REQ-IMP-001".to_string()));
    assert!(!ids(&since["report"]["affected"]).contains(&"TC-IMP-001".to_string()), "depth applies");

    // The element form is unchanged, and the arguments are checked.
    assert!(mcp.call_tool("impact", json!({"ref": "REQ-IMP-001"}))["affected"].is_array());
    for bad in [json!({}), json!({"ref": "REQ-IMP-001", "files": ["x"]}), json!({"since": "no-such-rev"})] {
        assert_eq!(mcp.call_tool_raw("impact", bad.clone())["isError"], true, "{bad}");
    }
}

#[allow(dead_code)]
fn _p(_: &Path) {}
