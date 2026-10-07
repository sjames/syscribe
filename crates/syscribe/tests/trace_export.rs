//! `syscribe trace-export` (ADR-SYS-TREX-001; `REQ-TRS-TREX-001`..`-004`).
//! Black-box: runs the binary against a small model written to a temp directory —
//! a parent requirement with an L4 TestCase, two derived children (one satisfied by
//! a PartDef, one with an active L2 TestCase plus a retired one and an `appliesWhen`
//! on an optional feature), a use case that `refines:` the parent, a requirement
//! whose `derivedFrom` dangles, a results sidecar and a Configuration that
//! deactivates the optional feature.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("syscribe-trace-export-{tag}-{}-{}-{}", std::process::id(), nanos, n))
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// The test model. Walker (directory) order of the requirements is
/// `REQ-TX-001`, `REQ-TX-002`, `REQ-TX-003` (depth 2) then `Core::REQ-TX-000`
/// (depth 3); ascending qname order puts `Requirements::Core::REQ-TX-000` first.
fn model() -> PathBuf {
    let root = tmp_dir("model");
    write(&root, "_index.md", "---\ntype: Package\nname: TX\n---\n\nTrace-export fixture.\n");
    write(&root, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n\nFeature model.\n");
    write(
        &root,
        "Features/Opt.md",
        "---\ntype: FeatureDef\nid: FEAT-TX-OPT\nname: Opt\nmandatory: false\n---\n\nOptional feature.\n",
    );
    write(
        &root,
        "Configurations/CONF-TX-001.md",
        "---\ntype: Configuration\nid: CONF-TX-001\nname: \"Base variant without Opt\"\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: false\n---\n\nBase variant.\n",
    );
    write(
        &root,
        "Decisions/ADR-TX-001.md",
        "---\ntype: ADR\nid: ADR-TX-001\nname: \"Breakdown of the parent\"\nstatus: accepted\n---\n\nDecision.\n",
    );
    write(&root, "Requirements/_index.md", "---\ntype: Package\nname: Requirements\n---\n\nRequirements.\n");
    write(&root, "Requirements/Core/_index.md", "---\ntype: Package\nname: Core\n---\n\nTop-level requirements.\n");
    write(
        &root,
        "Requirements/Core/REQ-TX-000.md",
        "---\ntype: Requirement\nid: REQ-TX-000\nname: \"Parent requirement\"\nstatus: approved\nreqDomain: system\nreqClass: stakeholder\n---\n\nThe system shall do the whole thing.\n",
    );
    write(
        &root,
        "Requirements/REQ-TX-001.md",
        "---\ntype: Requirement\nid: REQ-TX-001\nname: \"Satisfied child\"\nstatus: approved\nreqDomain: software\nreqClass: system\nderivedFrom: [REQ-TX-000]\nbreakdownAdr: Decisions::ADR-TX-001\n---\n\nThe software shall do part one.\n",
    );
    write(
        &root,
        "Requirements/REQ-TX-002.md",
        "---\ntype: Requirement\nid: REQ-TX-002\nname: \"Verified optional child\"\nstatus: approved\nreqDomain: software\nreqClass: system\nderivedFrom: [REQ-TX-000]\nbreakdownAdr: Decisions::ADR-TX-001\nappliesWhen: Features::Opt\n---\n\nThe software shall do part two when Opt is selected.\n",
    );
    write(
        &root,
        "Requirements/REQ-TX-003.md",
        "---\ntype: Requirement\nid: REQ-TX-003\nname: \"Dangling child\"\nstatus: draft\nreqDomain: software\nreqClass: system\nderivedFrom: [REQ-TX-999]\nbreakdownAdr: Decisions::ADR-TX-001\n---\n\nDerived from a requirement that does not exist.\n",
    );
    write(
        &root,
        "Arch/Alpha.md",
        "---\ntype: PartDef\nname: Alpha\ndomain: software\nsatisfies: [REQ-TX-001]\n---\n\nSatisfies part one.\n",
    );
    write(
        &root,
        "UseCases/Login.md",
        "---\ntype: UseCase\nname: Login\nrefines: [REQ-TX-000]\n---\n\nRefines the parent.\n",
    );
    write(
        &root,
        "Verification/TC-TX-000.md",
        "---\ntype: TestCase\nid: TC-TX-000\nname: \"Integration test of the parent\"\nstatus: active\ntestLevel: L4\nverifies: [REQ-TX-000]\n---\n\n```gherkin\nFeature: parent\n  Scenario: whole thing\n    Given the system\n    When it runs\n    Then it does the whole thing\n```\n",
    );
    write(
        &root,
        "Verification/TC-TX-002.md",
        "---\ntype: TestCase\nid: TC-TX-002\nname: \"Unit test of part two\"\nstatus: active\ntestLevel: L2\nverifies: [REQ-TX-002]\nappliesWhen: Features::Opt\ntestFunctions:\n  - function: \"tests::tx_002\"\n---\n\n```gherkin\nFeature: part two\n  Scenario: part two\n    Given Opt\n    When it runs\n    Then part two happens\n```\n",
    );
    write(
        &root,
        "Verification/TC-TX-003.md",
        "---\ntype: TestCase\nid: TC-TX-003\nname: \"Retired test of part two\"\nstatus: retired\ntestLevel: L2\nverifies: [REQ-TX-002]\nappliesWhen: Features::Opt\n---\n\n```gherkin\nFeature: part two (old)\n  Scenario: old\n    Given Opt\n    When it ran\n    Then it used to pass\n```\n",
    );
    write(
        &root,
        ".syscribe/results.json",
        r#"{"schema_version":"1","format":"cargo-json","source":"run.json","ingested_at_unix":0,"count":1,"by_leaf":{"tx_002":"pass"}}"#,
    );
    root
}

fn run_in(model: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(model)
        .arg("trace-export")
        .args(args)
        .output()
        .expect("spawn syscribe")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn doc(model: &Path, args: &[&str]) -> Value {
    let o = run_in(model, args);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    serde_json::from_str(&stdout(&o)).unwrap_or_else(|e| panic!("not JSON: {e}\n{}", stdout(&o)))
}

fn req<'a>(d: &'a Value, id: &str) -> &'a Value {
    d["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("no requirement {id} in {d}"))
}

fn ids(d: &Value) -> Vec<String> {
    d["requirements"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap().to_string()).collect()
}

fn qnames(list: &Value) -> Vec<String> {
    list.as_array().unwrap().iter().map(|r| r["qname"].as_str().unwrap().to_string()).collect()
}

/// Top-level keys of the first JSON object in `text`, in document order.
fn keys_in_order(text: &str, keys: &[&str]) -> Vec<usize> {
    keys.iter().map(|k| text.find(&format!("\"{k}\":")).unwrap_or_else(|| panic!("missing key {k}"))).collect()
}

// ── REQ-TRS-TREX-001: schema, references, coverage, unresolved ──────────────

#[test]
fn document_has_the_normative_shape_and_field_order() {
    let m = model();
    let o = run_in(&m, &[]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let text = stdout(&o);
    let d: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(d["version"], 1);
    assert_eq!(d["modelRoot"].as_str(), Some(m.to_str().unwrap()));
    assert!(d["config"].is_null());
    assert_eq!(d["sort"], "directory");
    assert_eq!(d["summary"]["requirements"], 4);
    // Top-level order.
    let pos = keys_in_order(&text, &["version", "modelRoot", "config", "sort", "requirements", "summary"]);
    assert!(pos.windows(2).all(|w| w[0] < w[1]), "top-level field order:\n{text}");
    // Entry order (the first entry is REQ-TX-001, whose breakdownAdr is an object).
    let first = &text[text.find("\"requirements\":").unwrap()..];
    let pos = keys_in_order(
        first,
        &[
            "qname", "id", "name", "type", "status", "reqClass", "reqDomain", "file", "derivedFrom",
            "derivedChildren", "breakdownAdr", "satisfiedBy", "verifiedBy", "refinedBy", "coverage",
        ],
    );
    assert!(pos.windows(2).all(|w| w[0] < w[1]), "entry field order:\n{first}");
    let cov = keys_in_order(first, &["leaf", "satisfied", "verified", "integrationVerified"]);
    assert!(cov.windows(2).all(|w| w[0] < w[1]), "coverage field order:\n{first}");
    let summary = &text[text.find("\"summary\":").unwrap()..];
    let pos = keys_in_order(summary, &["requirements", "leaves", "satisfied", "verified", "integrationVerified"]);
    assert!(pos.windows(2).all(|w| w[0] < w[1]), "summary field order:\n{summary}");
    // Identity fields.
    let r = req(&d, "REQ-TX-001");
    assert_eq!(r["qname"], "Requirements::REQ-TX-001");
    assert_eq!(r["name"], "Satisfied child");
    assert_eq!(r["type"], "Requirement");
    assert_eq!(r["status"], "approved");
    assert_eq!(r["reqClass"], "system");
    assert_eq!(r["reqDomain"], "software");
    assert_eq!(r["file"], "Requirements/REQ-TX-001.md");
}

#[test]
fn every_reference_is_a_full_qualified_name_with_id() {
    let m = model();
    let d = doc(&m, &[]);
    let child = req(&d, "REQ-TX-001");
    assert_eq!(child["derivedFrom"], serde_json::json!([{"qname": "Requirements::Core::REQ-TX-000", "id": "REQ-TX-000"}]));
    assert_eq!(
        child["breakdownAdr"],
        serde_json::json!({"qname": "Decisions::ADR-TX-001", "id": "ADR-TX-001", "status": "accepted"})
    );
    assert_eq!(
        child["satisfiedBy"],
        serde_json::json!([{"qname": "Arch::Alpha", "id": null, "type": "PartDef", "domain": "software"}])
    );
    let parent = req(&d, "REQ-TX-000");
    assert_eq!(qnames(&parent["derivedChildren"]), ["Requirements::REQ-TX-001", "Requirements::REQ-TX-002"]);
    assert_eq!(parent["derivedChildren"][0]["id"], "REQ-TX-001");
    assert_eq!(parent["refinedBy"], serde_json::json!([{"qname": "UseCases::Login", "id": null}]));
    assert!(parent["breakdownAdr"].is_null());
    assert_eq!(
        parent["verifiedBy"],
        serde_json::json!([{"qname": "Verification::TC-TX-000", "id": "TC-TX-000", "testLevel": "L4", "status": "active", "verdict": "unknown"}])
    );
    let opt = req(&d, "REQ-TX-002");
    assert_eq!(qnames(&opt["verifiedBy"]), ["Verification::TC-TX-002", "Verification::TC-TX-003"]);
    assert_eq!(opt["verifiedBy"][1]["status"], "retired");
}

#[test]
fn coverage_mirrors_w300_w002_w305_and_summary_counts() {
    let m = model();
    let d = doc(&m, &[]);
    let cov = |id: &str| req(&d, id)["coverage"].clone();
    // Parent: not a leaf, unsatisfied, verified by an active L4 → integration-verified.
    assert_eq!(cov("REQ-TX-000"), serde_json::json!({"leaf": false, "satisfied": false, "verified": true, "integrationVerified": true}));
    // Leaf satisfied by a PartDef, no TestCase.
    assert_eq!(cov("REQ-TX-001"), serde_json::json!({"leaf": true, "satisfied": true, "verified": false, "integrationVerified": false}));
    // Leaf verified by an active L2 (the retired one is listed but never counts).
    assert_eq!(cov("REQ-TX-002"), serde_json::json!({"leaf": true, "satisfied": false, "verified": true, "integrationVerified": false}));
    // Leaf with nothing.
    assert_eq!(cov("REQ-TX-003"), serde_json::json!({"leaf": true, "satisfied": false, "verified": false, "integrationVerified": false}));
    assert_eq!(
        d["summary"],
        serde_json::json!({"requirements": 4, "leaves": 3, "satisfied": 1, "verified": 2, "integrationVerified": 1})
    );
}

#[test]
fn a_dangling_reference_is_kept_as_unresolved() {
    let m = model();
    let d = doc(&m, &[]);
    let r = req(&d, "REQ-TX-003");
    assert_eq!(r["derivedFrom"], serde_json::json!([{"qname": "REQ-TX-999", "unresolved": true}]));
    assert_eq!(r["derivedFrom"][0].as_object().unwrap().len(), 2, "exactly {{qname, unresolved}}");
}

#[test]
fn verdicts_come_from_the_results_sidecar_and_are_null_without_one() {
    let m = model();
    let d = doc(&m, &[]);
    let opt = req(&d, "REQ-TX-002");
    assert_eq!(opt["verifiedBy"][0]["verdict"], "pass", "tx_002 passed in the sidecar");
    assert_eq!(opt["verifiedBy"][1]["verdict"], "unknown", "no testFunctions → unknown");
    std::fs::remove_file(m.join(".syscribe/results.json")).unwrap();
    let d = doc(&m, &[]);
    let opt = req(&d, "REQ-TX-002");
    assert!(opt["verifiedBy"][0]["verdict"].is_null(), "no sidecar → null: {opt}");
    assert!(req(&d, "REQ-TX-000")["verifiedBy"][0]["verdict"].is_null());
}

// ── REQ-TRS-TREX-002: configuration projection ──────────────────────────────

#[test]
fn config_is_null_and_nothing_filtered_without_the_lens() {
    let m = model();
    let d = doc(&m, &[]);
    assert!(d["config"].is_null());
    assert_eq!(ids(&d), ["REQ-TX-001", "REQ-TX-002", "REQ-TX-003", "REQ-TX-000"]);
}

#[test]
fn config_projection_omits_inactive_elements_and_records_the_configuration() {
    let m = model();
    let d = doc(&m, &["--config", "CONF-TX-001"]);
    assert_eq!(
        d["config"],
        serde_json::json!({"id": "CONF-TX-001", "qname": "Configurations::CONF-TX-001", "name": "Base variant without Opt", "activeFeatures": []})
    );
    // REQ-TX-002 (appliesWhen Features::Opt) is gone from the list…
    assert_eq!(ids(&d), ["REQ-TX-001", "REQ-TX-003", "REQ-TX-000"]);
    // …and from the parent's children; coverage is computed over the projected lists.
    let parent = req(&d, "REQ-TX-000");
    assert_eq!(qnames(&parent["derivedChildren"]), ["Requirements::REQ-TX-001"]);
    assert_eq!(parent["coverage"]["leaf"], false);
    assert_eq!(d["summary"], serde_json::json!({"requirements": 3, "leaves": 2, "satisfied": 1, "verified": 1, "integrationVerified": 1}));
    // The same configuration by qualified name.
    let d2 = doc(&m, &["--config", "Configurations::CONF-TX-001"]);
    assert_eq!(d2["config"]["id"], "CONF-TX-001");
    assert_eq!(ids(&d2), ids(&d));
}

#[test]
fn an_ad_hoc_feature_set_is_recorded_without_id() {
    let m = model();
    let d = doc(&m, &["--config", "Features::Opt"]);
    assert_eq!(
        d["config"],
        serde_json::json!({"id": null, "qname": null, "name": "Features::Opt", "activeFeatures": ["Features::Opt"]})
    );
    assert_eq!(ids(&d), ["REQ-TX-001", "REQ-TX-002", "REQ-TX-003", "REQ-TX-000"]);
    assert_eq!(req(&d, "REQ-TX-002")["coverage"]["verified"], true);
    // The FEAT-* id is accepted and normalised to the qualified name.
    let d = doc(&m, &["--config", "FEAT-TX-OPT"]);
    assert_eq!(d["config"]["activeFeatures"], serde_json::json!(["Features::Opt"]));
}

#[test]
fn an_invalid_configuration_is_a_usage_error() {
    let m = model();
    let o = run_in(&m, &["--config", "CONF-NOPE-001"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(stdout(&o), "", "nothing on stdout");
    let e = stderr(&o);
    assert!(e.contains("CONF-NOPE-001"), "names the configuration: {e}");
    assert!(e.contains("neither a known Configuration nor a set of FeatureDef qualified names"), "{e}");
}

// ── REQ-TRS-TREX-003: sort orders ───────────────────────────────────────────

#[test]
fn directory_sort_follows_the_walker_order() {
    let m = model();
    // `export` lists elements in walker order; the default sort must match it.
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&m).arg("export").output().unwrap();
    let export: Value = serde_json::from_slice(&o.stdout).unwrap();
    let export_reqs: Vec<String> = export["elements"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "Requirement")
        .map(|e| e["qname"].as_str().unwrap().to_string())
        .collect();
    let d = doc(&m, &["--sort", "directory"]);
    assert_eq!(d["sort"], "directory");
    assert_eq!(qnames(&d["requirements"]), export_reqs);
    assert_eq!(ids(&d), ["REQ-TX-001", "REQ-TX-002", "REQ-TX-003", "REQ-TX-000"]);
    // Nested lists follow the same order.
    assert_eq!(qnames(&req(&d, "REQ-TX-002")["verifiedBy"]), ["Verification::TC-TX-002", "Verification::TC-TX-003"]);
    assert_eq!(doc(&m, &[])["requirements"], d["requirements"], "directory is the default");
}

#[test]
fn asc_and_desc_sort_every_list_by_qualified_name() {
    let m = model();
    let asc = doc(&m, &["--sort", "asc"]);
    assert_eq!(asc["sort"], "asc");
    assert_eq!(ids(&asc), ["REQ-TX-000", "REQ-TX-001", "REQ-TX-002", "REQ-TX-003"]);
    assert_eq!(qnames(&req(&asc, "REQ-TX-000")["derivedChildren"]), ["Requirements::REQ-TX-001", "Requirements::REQ-TX-002"]);
    assert_eq!(qnames(&req(&asc, "REQ-TX-002")["verifiedBy"]), ["Verification::TC-TX-002", "Verification::TC-TX-003"]);
    let desc = doc(&m, &["--sort", "desc"]);
    assert_eq!(desc["sort"], "desc");
    assert_eq!(ids(&desc), ["REQ-TX-003", "REQ-TX-002", "REQ-TX-001", "REQ-TX-000"]);
    assert_eq!(qnames(&req(&desc, "REQ-TX-000")["derivedChildren"]), ["Requirements::REQ-TX-002", "Requirements::REQ-TX-001"]);
    assert_eq!(qnames(&req(&desc, "REQ-TX-002")["verifiedBy"]), ["Verification::TC-TX-003", "Verification::TC-TX-002"]);
    // Only the order differs: the same entries, the same summary.
    assert_eq!(asc["summary"], desc["summary"]);
    let mut a = qnames(&asc["requirements"]);
    let mut b = qnames(&desc["requirements"]);
    a.sort();
    b.sort();
    assert_eq!(a, b);
}

#[test]
fn a_bad_sort_is_a_usage_error_naming_the_valid_values() {
    let m = model();
    let o = run_in(&m, &["--sort", "random"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(stdout(&o), "");
    let e = stderr(&o);
    assert!(e.contains("invalid value 'random' for --sort"), "{e}");
    assert!(e.contains("directory, asc, desc"), "{e}");
}

#[test]
fn output_is_byte_identical_across_runs() {
    let m = model();
    for args in [vec![], vec!["--sort", "desc"], vec!["--config", "CONF-TX-001"]] {
        let a = run_in(&m, &args);
        let b = run_in(&m, &args);
        assert_eq!(a.status.code(), Some(0), "{}", stderr(&a));
        assert_eq!(a.stdout, b.stdout, "{args:?}");
    }
}

// ── REQ-TRS-TREX-004: the CLI surface ───────────────────────────────────────

#[test]
fn out_writes_the_file_and_creates_parents() {
    let m = model();
    let file = tmp_dir("out").join("nested/deeper/trace.json");
    let o = run_in(&m, &["--sort", "asc", "--out", file.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    assert_eq!(stdout(&o), "", "with --out nothing goes to stdout");
    let written = std::fs::read_to_string(&file).expect("file written");
    let d: Value = serde_json::from_str(&written).unwrap();
    assert_eq!(d["sort"], "asc");
    assert_eq!(d["summary"]["requirements"], 4);
    // Same bytes as stdout would carry (plus the trailing newline).
    assert_eq!(written, stdout(&run_in(&m, &["--sort", "asc"])));
}

#[test]
fn an_unknown_option_is_rejected_before_the_model_loads() {
    let o = run_in(Path::new("/nonexistent/model/root"), &["--bogus"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(stdout(&o), "");
    let e = stderr(&o);
    assert!(e.contains("trace-export: unknown option '--bogus'"), "{e}");
    assert!(e.contains("--config") && e.contains("--sort") && e.contains("--out"), "lists the valid options: {e}");
    // A value-taking option with no value is the same kind of error.
    let o = run_in(Path::new("/nonexistent/model/root"), &["--sort"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("option '--sort' expects a value"), "{}", stderr(&o));
}

#[test]
fn help_page_is_registered() {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["help", "trace-export"]).output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    let page = stdout(&o);
    assert!(page.contains("## SYNOPSIS") && page.contains("trace-export [--config <C>] [--sort <order>] [--out <file>]"), "{page}");
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["trace-export", "--help"]).output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert!(stdout(&o).contains("## SYNOPSIS"));
}
