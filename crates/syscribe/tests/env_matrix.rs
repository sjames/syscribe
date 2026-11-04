//! TC-TRS-ENVMATRIX-001 (GH #437): `matrix --by environment`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Repo {
    root: PathBuf,
}

const TE: &str = "---\nid: {id}\ntype: TestEnvironment\nname: {id}\nstatus: {st}\nenvironmentKind: bench\n---\n\nrig\n";

impl Repo {
    fn new() -> Repo {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-envmx-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "h@example.com"]);
        r.git(&["config", "user.name", "H"]);
        r.git(&["config", "commit.gpgsign", "false"]);
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r.write("model/TE-SIM-001.md", &TE.replace("{id}", "TE-SIM-001").replace("{st}", "available"));
        r.write("model/TE-RIG-001.md", &TE.replace("{id}", "TE-RIG-001").replace("{st}", "available"));
        r.write("model/REQ-EM-001.md", &req("REQ-EM-001", "approved", "The system shall do one."));
        r.write("model/REQ-EM-002.md", &req("REQ-EM-002", "approved", "The system shall do two."));
        r.write("model/TC-EM-001.md", &tc("TC-EM-001", "REQ-EM-001", "[TE-SIM-001, TE-RIG-001]"));
        r.write("model/TC-EM-002.md", &tc("TC-EM-002", "REQ-EM-002", "[TE-SIM-001]"));
        r.commit("base");
        r
    }
    fn git(&self, args: &[&str]) -> String {
        let o = Command::new("git").arg("-C").arg(&self.root).args(args).output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }
    fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"])
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
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(&self.root).arg("-m").arg(self.root.join("model")).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
    /// Ingest records: (test, verdict, environment, build commit, finishedAt).
    fn ingest(&self, recs: &[(&str, &str, &str, &str, &str)]) {
        let body: Vec<String> = recs
            .iter()
            .map(|(t, v, e, b, at)| {
                format!("{{\"testCase\":\"{t}\",\"verdict\":\"{v}\",\"provenance\":{{\"environment\":\"{e}\",\"build\":{{\"commit\":\"{b}\"}},\"finishedAt\":\"{at}\"}}}}")
            })
            .collect();
        self.write("results.json", &format!("[{}]", body.join(",")));
        let f = self.root.join("results.json");
        let (o, e, c) = self.run(&["ingest-results", f.to_str().unwrap(), "--format", "records"]);
        assert_eq!(c, 0, "{o}{e}");
    }
    fn json(&self, extra: &[&str]) -> serde_json::Value {
        let mut a = vec!["matrix", "--by", "environment", "--json"];
        a.extend_from_slice(extra);
        let (o, e, c) = self.run(&a);
        assert_eq!(c, 0, "{o}{e}");
        serde_json::from_str(&o).unwrap_or_else(|x| panic!("{x}: {o}"))
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn req(id: &str, st: &str, body: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: R {id}\nstatus: {st}\nreqDomain: system\n---\n\n{body}\n")
}
fn tc(id: &str, verifies: &str, runs_on: &str) -> String {
    format!("---\nid: {id}\ntype: TestCase\nname: T {id}\nstatus: active\ntestLevel: L4\nverifies: [{verifies}]\nrunsOn: {runs_on}\n---\n\nbody\n")
}
fn row<'a>(j: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    j["rows"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap_or_else(|| panic!("no row {id} in {j}"))
}

#[test]
fn a_simulator_pass_with_no_rig_run_is_a_gap_on_the_rig() {
    let r = Repo::new();
    let h = r.head();
    r.ingest(&[("TC-EM-001", "pass", "TE-SIM-001", &h, "2026-10-01T08:00:00Z")]);
    let j = r.json(&[]);
    let row1 = row(&j, "REQ-EM-001");
    assert_eq!(row1["cells"]["TE-SIM-001"]["state"], "pass");
    assert_eq!(row1["cells"]["TE-SIM-001"]["buildCommit"], h.as_str());
    assert_eq!(row1["cells"]["TE-RIG-001"]["state"], "gap");
    assert_eq!(row1["rollup"], "gap");
    // REQ-EM-002 declares only the simulator, where it has no result: also a gap there; the rig column is empty.
    let row2 = row(&j, "REQ-EM-002");
    assert_eq!(row2["cells"]["TE-RIG-001"]["state"], "none");
    assert_eq!(row2["cells"]["TE-SIM-001"]["state"], "gap");
    let (text, _e, c) = r.run(&["matrix", "--by", "environment"]);
    assert_eq!(c, 0);
    assert!(text.contains("| REQ-EM-001 |") && text.contains("pass @") && text.contains("gap"), "{text}");
}

#[test]
fn both_rigs_verified_and_a_failure_fails_the_rollup() {
    let r = Repo::new();
    let h = r.head();
    r.ingest(&[
        ("TC-EM-001", "pass", "TE-SIM-001", &h, "2026-10-01T08:00:00Z"),
        ("TC-EM-001", "pass", "TE-RIG-001", &h, "2026-10-01T09:00:00Z"),
        ("TC-EM-002", "fail", "TE-SIM-001", &h, "2026-10-01T09:00:00Z"),
    ]);
    let j = r.json(&[]);
    assert_eq!(row(&j, "REQ-EM-001")["rollup"], "verified");
    assert_eq!(row(&j, "REQ-EM-002")["rollup"], "failing");
    assert_eq!(j["summary"]["verified"], 1);
    let gaps = r.json(&["--gaps-only"]);
    let ids: Vec<_> = gaps["rows"].as_array().unwrap().iter().map(|x| x["id"].as_str().unwrap().to_string()).collect();
    assert_eq!(ids, vec!["REQ-EM-002"]);
}

#[test]
fn a_result_for_a_build_older_than_the_last_change_is_stale() {
    let r = Repo::new();
    let old = r.head();
    r.ingest(&[
        ("TC-EM-001", "pass", "TE-SIM-001", &old, "2026-10-01T08:00:00Z"),
        ("TC-EM-001", "pass", "TE-RIG-001", &old, "2026-10-01T08:00:00Z"),
    ]);
    r.commit("results");
    // Same content, but a later commit touches the requirement file only in a way the content check ignores
    // (editorial field), so only the history check can see it.
    r.write("model/REQ-EM-001.md", &req("REQ-EM-001", "approved", "The system shall do one.").replace("reqDomain: system\n", "reqDomain: system\nrationale: reworded\n"));
    r.commit("touch requirement");
    let j = r.json(&[]);
    let c = &row(&j, "REQ-EM-001")["cells"]["TE-SIM-001"];
    assert_eq!(c["state"], "stale", "{j}");
    assert!(c["staleReasons"].to_string().contains("predates"), "{c}");
    assert_eq!(row(&j, "REQ-EM-001")["rollup"], "stale");
    assert_eq!(r.json(&["--gaps-only"])["rows"].as_array().unwrap().len(), 2, "stale and the other gap are both listed");
}

#[test]
fn a_changed_requirement_since_ingest_is_stale_by_content() {
    let r = Repo::new();
    let h = r.head();
    r.ingest(&[("TC-EM-002", "pass", "TE-SIM-001", &h, "2026-10-01T08:00:00Z")]);
    r.write("model/REQ-EM-002.md", &req("REQ-EM-002", "approved", "The system shall do two, faster."));
    // Uncommitted: the history check cannot see it, the content check does.
    let j = r.json(&[]);
    let c = &row(&j, "REQ-EM-002")["cells"]["TE-SIM-001"];
    assert_eq!(c["state"], "stale", "{j}");
    assert!(c["staleReasons"].to_string().contains("REQ-EM-002 changed"), "{c}");
}

#[test]
fn an_unknown_build_is_noted_not_judged_and_a_retired_rig_does_not_count() {
    let r = Repo::new();
    r.ingest(&[
        ("TC-EM-001", "pass", "TE-SIM-001", "deadbee", "2026-10-01T08:00:00Z"),
        ("TC-EM-001", "pass", "TE-RIG-001", "deadbee", "2026-10-01T08:00:00Z"),
    ]);
    let j = r.json(&[]);
    let c = &row(&j, "REQ-EM-001")["cells"]["TE-SIM-001"];
    assert_eq!(c["state"], "pass", "{c}");
    assert!(c["note"].as_str().unwrap().contains("not a commit"), "{c}");
    // Retire the rig: its result no longer counts.
    r.write("model/TE-RIG-001.md", &TE.replace("{id}", "TE-RIG-001").replace("{st}", "retired"));
    let j = r.json(&[]);
    let rig = &row(&j, "REQ-EM-001")["cells"]["TE-RIG-001"];
    assert_eq!(rig["state"], "gap", "{rig}");
    assert!(rig["note"].as_str().unwrap().contains("retired"), "{rig}");
    assert_eq!(row(&j, "REQ-EM-001")["rollup"], "gap");
    assert!(j["environments"].to_string().contains("\"usable\":false"));
}

#[test]
fn filters_and_unknown_grouping() {
    let r = Repo::new();
    r.write("model/REQ-EM-003.md", &req("REQ-EM-003", "draft", "Three."));
    let j = r.json(&["--status", "draft"]);
    assert_eq!(j["rows"].as_array().unwrap().len(), 1);
    assert_eq!(row(&j, "REQ-EM-003")["rollup"], "none");
    let (_o, e, c) = r.run(&["matrix", "--by", "colour"]);
    assert_eq!(c, 2, "{e}");
    assert!(e.contains("environment"));
}

#[test]
fn the_mcp_tool_returns_the_same_data() {
    let r = Repo::new();
    let h = r.head();
    r.ingest(&[("TC-EM-001", "pass", "TE-SIM-001", &h, "2026-10-01T08:00:00Z")]);
    let cli = r.json(&[]);
    let mut mcp = common::Mcp::start(&r.root.join("model"));
    mcp.initialize();
    let v = mcp.call_tool("coverage_matrix", serde_json::json!({"by": "environment"}));
    assert_eq!(v, cli);
}

mod common;
