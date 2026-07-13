//! Integration harness for release baselines (REQ-TRS-BL-000..009, verified by
//! TC-TRS-BL-001..009 / 100).
//!
//! Black-box: each test builds a **git-backed** temp model and drives the `syscribe`
//! binary. Baselines need a git anchor, so the harness `git init`s a repo and makes
//! an initial clean commit before sealing.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git").arg("-C").arg(dir).args(args).output().expect("git").status.success();
    assert!(ok, "git {args:?} failed in {}", dir.display());
}

/// Fresh git repo with a `model/` tree carrying two approved requirements; returns
/// (repo_root, model_root) after an initial clean commit.
fn new_git_model() -> (PathBuf, PathBuf) {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let repo = std::env::temp_dir().join(format!("syscribe-bl-{}-{}-{}", std::process::id(), nanos, n));
    let model = repo.join("model");
    std::fs::create_dir_all(model.join("Requirements")).unwrap();
    write(&model, "_index.md", "---\ntype: Package\nname: M\n---\n\nroot\n");
    write_req(&model, "REQ-MOD-001", "Original body one.");
    write_req(&model, "REQ-MOD-002", "Original body two.");
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.email", "t@t"]);
    git(&repo, &["config", "user.name", "t"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "init"]);
    (repo, model)
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}
fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}
fn write_req(root: &Path, id: &str, body: &str) {
    write(
        root,
        &format!("Requirements/{id}.md"),
        &format!("---\ntype: Requirement\nid: {id}\nname: \"{id}\"\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\n{body}\n"),
    );
}

fn run(model: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(model).args(args).output().expect("spawn syscribe");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}
fn validate_out(model: &Path) -> String {
    let (o, e, _) = run(model, &["validate"]);
    format!("{o}\n{e}")
}
/// `baseline create` on a clean tree, committing the result so the tree stays clean.
fn create(repo: &Path, model: &Path, tag: &str, extra: &[&str]) -> (String, String, i32) {
    let mut a = vec!["baseline", "create", "--tag", tag];
    a.extend_from_slice(extra);
    let r = run(model, &a);
    git(repo, &["add", "-A"]);
    // may be a no-op if create refused; ignore failure
    let _ = Command::new("git").arg("-C").arg(repo).args(["commit", "-qm", "baseline"]).output();
    r
}
fn set_status(model: &Path, id: &str, status: &str) {
    let rel = format!("Baselines/{id}.md");
    let s: String = read(model, &rel)
        .lines()
        .map(|l| if l.starts_with("status:") { format!("status: {status}") } else { l.to_string() })
        .collect::<Vec<_>>()
        .join("\n");
    write(model, &rel, &format!("{s}\n"));
}

// TC-TRS-BL-004 — create writes element + manifest, captures HEAD.
#[test]
fn create_writes_element_and_manifest_and_captures_commit() {
    let (repo, model) = new_git_model();
    // Use `run` (not the committing helper) so HEAD stays at the sealed commit.
    let (o, _e, code) = run(&model, &["baseline", "create", "--tag", "REL-2026-06", "--approver", "J. Roe"]);
    assert_eq!(code, 0, "create exits 0: {o}");
    assert!(model.join("Baselines/BL-2026-06.md").exists(), "element written");
    assert!(repo.join("baselines/BL-2026-06.manifest.json").exists(), "manifest written");
    let el = read(&model, "Baselines/BL-2026-06.md");
    assert!(el.contains("aggregateHash: blake3:"), "seal written:\n{el}");
    let head = String::from_utf8_lossy(&Command::new("git").arg("-C").arg(&repo).args(["rev-parse", "HEAD"]).output().unwrap().stdout).trim().to_string();
    assert!(el.contains(&head), "gitCommit == HEAD:\n{el}");
}

// TC-TRS-BL-004 — dirty tree is refused unless --allow-dirty.
#[test]
fn create_refuses_dirty_tree() {
    let (_repo, model) = new_git_model();
    write_req(&model, "REQ-MOD-003", "Uncommitted addition.");
    let (_o, _e, code) = run(&model, &["baseline", "create", "--tag", "REL-2026-06"]);
    assert_ne!(code, 0, "dirty tree refused without --allow-dirty");
    let (_o2, _e2, code2) = run(&model, &["baseline", "create", "--tag", "REL-2026-06", "--allow-dirty"]);
    assert_eq!(code2, 0, "--allow-dirty proceeds");
}

// TC-TRS-BL-003 — an empty resolved scope is refused.
#[test]
fn create_refuses_empty_scope() {
    let (_repo, model) = new_git_model();
    let (_o, _e, code) = run(&model, &["baseline", "create", "--tag", "REL-2026-06", "--frozen-scope", "types=ViewDef"]);
    assert_ne!(code, 0, "empty scope refused");
    assert!(!model.join("Baselines/BL-2026-06.md").exists(), "nothing written");
}

// TC-TRS-BL-002/003 — baselines are excluded from scope, so no mutual drift.
#[test]
fn baselines_excluded_no_mutual_drift() {
    let (repo, model) = new_git_model();
    create(&repo, &model, "REL-2026-06", &[]);
    set_status(&model, "BL-2026-06", "released");
    git(&repo, &["add", "-A"]);
    let _ = Command::new("git").arg("-C").arg(&repo).args(["commit", "-qm", "release A"]).output();
    // Seal a second whole-model baseline; it must not include BL-2026-06, and BL-2026-06 must not drift.
    create(&repo, &model, "REL-2026-07", &[]);
    let out = validate_out(&model);
    assert!(!out.contains("E520"), "sealing BL-2026-07 must not drift released BL-2026-06:\n{out}");
    let manifest = read(&repo, "baselines/BL-2026-07.manifest.json");
    assert!(!manifest.contains("BL-2026-06"), "BL-2026-07 manifest excludes the other baseline:\n{manifest}");
}

// TC-TRS-BL-005 — drift severity is status-graded.
#[test]
fn drift_severity_follows_status() {
    let (repo, model) = new_git_model();
    create(&repo, &model, "REL-2026-06", &[]);
    // change an in-scope requirement
    write_req(&model, "REQ-MOD-001", "CHANGED body.");

    // draft → silent
    assert!(!validate_out(&model).contains("E520") && !validate_out(&model).contains("W520"), "draft is silent");
    // approved → W520
    set_status(&model, "BL-2026-06", "approved");
    let out = validate_out(&model);
    assert!(out.contains("W520") && !out.contains("E520"), "approved drift is W520:\n{out}");
    // released → E520
    set_status(&model, "BL-2026-06", "released");
    assert!(validate_out(&model).contains("E520"), "released drift is E520");
    // superseded → skipped
    set_status(&model, "BL-2026-06", "superseded");
    let out = validate_out(&model);
    assert!(!out.contains("E520") && !out.contains("W520"), "superseded is not drift-checked:\n{out}");
}

// TC-TRS-BL-005 — an unresolved supersedes is E522.
#[test]
fn unresolved_supersedes_is_e522() {
    let (repo, model) = new_git_model();
    create(&repo, &model, "REL-2026-06", &[]);
    let rel = "Baselines/BL-2026-06.md";
    let s = read(&model, rel).replace("status: draft", "status: draft\nsupersedes: BL-DOES-NOT-EXIST");
    write(&model, rel, &s);
    assert!(validate_out(&model).contains("E522"), "unresolved supersedes is E522");
}

// TC-TRS-BL-002 — a full-content (editorial) edit drifts the seal.
#[test]
fn editorial_edit_drifts_the_seal() {
    let (repo, model) = new_git_model();
    create(&repo, &model, "REL-2026-06", &[]);
    set_status(&model, "BL-2026-06", "released");
    assert!(!validate_out(&model).contains("E520"), "precondition: clean");
    // change ONLY an editorial field (name) of an in-scope element
    let rel = "Requirements/REQ-MOD-001.md";
    let s = read(&model, rel).replace("name: \"REQ-MOD-001\"", "name: \"A different editorial label\"");
    write(&model, rel, &s);
    assert!(validate_out(&model).contains("E520"), "editorial edit must drift the full-content seal");
}

// TC-TRS-BL-008 — verify passes clean, fails on drift, exits non-zero.
#[test]
fn verify_passes_then_fails_on_drift() {
    let (repo, model) = new_git_model();
    create(&repo, &model, "REL-2026-06", &[]);
    let (_o, _e, code) = run(&model, &["baseline", "verify", "BL-2026-06"]);
    assert_eq!(code, 0, "verify passes for an intact baseline");
    write_req(&model, "REQ-MOD-001", "CHANGED.");
    let (_o2, _e2, code2) = run(&model, &["baseline", "verify", "BL-2026-06"]);
    assert_eq!(code2, 2, "verify fails (exit 2) on drift");
}

// TC-TRS-BL-006 — diff reports the changed element, keyed by id.
#[test]
fn diff_reports_changed_element() {
    let (repo, model) = new_git_model();
    create(&repo, &model, "REL-2026-06", &[]);
    write_req(&model, "REQ-MOD-001", "CHANGED for the second baseline.");
    git(&repo, &["add", "-A"]);
    let _ = Command::new("git").arg("-C").arg(&repo).args(["commit", "-qm", "change"]).output();
    create(&repo, &model, "REL-2026-07", &[]);
    let (o, _e, _c) = run(&model, &["baseline", "diff", "BL-2026-06", "BL-2026-07"]);
    assert!(o.contains("changed (1)"), "one element changed:\n{o}");
    assert!(o.contains("REQ-MOD-001"), "the changed element is named:\n{o}");
}

// TC-TRS-BL-007 — list and show are read-only.
#[test]
fn list_and_show_are_read_only() {
    let (repo, model) = new_git_model();
    create(&repo, &model, "REL-2026-06", &["--approver", "J. Roe"]);
    let (lo, _e, _c) = run(&model, &["baseline", "list"]);
    assert!(lo.contains("BL-2026-06"), "list names the baseline:\n{lo}");
    let (so, _e2, _c2) = run(&model, &["baseline", "show", "BL-2026-06"]);
    assert!(so.contains("aggregate:") && so.contains("J. Roe"), "show prints provenance:\n{so}");
    // read-only: status should still be sealed as draft
    assert!(read(&model, "Baselines/BL-2026-06.md").contains("status: draft"), "list/show mutate nothing");
}

/// A git-backed model with a two-feature product line: FeatA/FeatB, two
/// Configurations, a base requirement plus one requirement gated to each feature.
fn new_variability_model() -> (PathBuf, PathBuf) {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let repo = std::env::temp_dir().join(format!("syscribe-blv-{}-{}-{}", std::process::id(), nanos, n));
    let model = repo.join("model");
    std::fs::create_dir_all(model.join("Requirements")).unwrap();
    write(&model, "_index.md", "---\ntype: Package\nname: M\n---\n\nroot\n");
    write(&model, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n\nfeatures\n");
    write(&model, "Features/FeatA.md", "---\ntype: FeatureDef\nid: FEAT-FA\nname: FeatA\n---\n\nFeature A.\n");
    write(&model, "Features/FeatB.md", "---\ntype: FeatureDef\nid: FEAT-FB\nname: FeatB\n---\n\nFeature B.\n");
    write(&model, "Configurations/CONF-VA-001.md",
        "---\ntype: Configuration\nid: CONF-VA-001\nname: \"Variant A\"\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::FeatA: true\n  Features::FeatB: false\n---\n\nA.\n");
    write(&model, "Configurations/CONF-VB-001.md",
        "---\ntype: Configuration\nid: CONF-VB-001\nname: \"Variant B\"\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::FeatA: false\n  Features::FeatB: true\n---\n\nB.\n");
    write_req(&model, "REQ-BASE-001", "Always-active base requirement.");
    write(&model, "Requirements/REQ-AONLY-001.md",
        "---\ntype: Requirement\nid: REQ-AONLY-001\nname: \"A only\"\nstatus: approved\nreqDomain: software\nreqClass: system\nappliesWhen: Features::FeatA\n---\n\nActive only in variant A.\n");
    write(&model, "Requirements/REQ-BONLY-001.md",
        "---\ntype: Requirement\nid: REQ-BONLY-001\nname: \"B only\"\nstatus: approved\nreqDomain: software\nreqClass: system\nappliesWhen: Features::FeatB\n---\n\nActive only in variant B.\n");
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.email", "t@t"]);
    git(&repo, &["config", "user.name", "t"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "init"]);
    (repo, model)
}

// TC-TRS-BL-011 — a config-scoped seal covers only the active variant.
#[test]
fn config_scope_seals_only_the_variant() {
    let (repo, model) = new_variability_model();
    let (o, _e, code) = create(&repo, &model, "REL-2026-06", &["--frozen-scope", "config=CONF-VA-001"]);
    assert_eq!(code, 0, "config-scoped create exits 0: {o}");
    let manifest = read(&repo, "baselines/BL-2026-06.manifest.json");
    assert!(manifest.contains("REQ-AONLY-001"), "variant-A seal includes the FeatA element:\n{manifest}");
    assert!(!manifest.contains("REQ-BONLY-001"), "variant-A seal excludes the FeatB element:\n{manifest}");
}

// TC-TRS-BL-011 — two configs seal different content.
#[test]
fn two_configs_seal_different_aggregates() {
    let (repo, model) = new_variability_model();
    create(&repo, &model, "REL-2026-06", &["--frozen-scope", "config=CONF-VA-001"]);
    create(&repo, &model, "REL-2026-07", &["--frozen-scope", "config=CONF-VB-001"]);
    let a = read(&repo, "baselines/BL-2026-06.manifest.json");
    let b = read(&repo, "baselines/BL-2026-07.manifest.json");
    let agg = |m: &str| m.lines().find(|l| l.contains("aggregateHash")).unwrap().to_string();
    assert_ne!(agg(&a), agg(&b), "different variants seal different aggregates");
}

// TC-TRS-BL-011 — drift follows the variant; out-of-variant change does not drift.
#[test]
fn config_scope_drift_follows_the_variant() {
    let (repo, model) = new_variability_model();
    create(&repo, &model, "REL-2026-06", &["--frozen-scope", "config=CONF-VA-001"]);
    set_status(&model, "BL-2026-06", "released");

    // change an element NOT active in variant A → no drift
    write(&model, "Requirements/REQ-BONLY-001.md",
        "---\ntype: Requirement\nid: REQ-BONLY-001\nname: \"B only\"\nstatus: approved\nreqDomain: software\nreqClass: system\nappliesWhen: Features::FeatB\n---\n\nCHANGED but out of variant A.\n");
    assert!(!validate_out(&model).contains("E520"), "out-of-variant change must not drift a config-scoped baseline");

    // change an element active in variant A → drift
    write(&model, "Requirements/REQ-AONLY-001.md",
        "---\ntype: Requirement\nid: REQ-AONLY-001\nname: \"A only\"\nstatus: approved\nreqDomain: software\nreqClass: system\nappliesWhen: Features::FeatA\n---\n\nCHANGED and in variant A.\n");
    assert!(validate_out(&model).contains("E520"), "in-variant change must drift the config-scoped baseline");
}

// TC-TRS-BL-011 — an unresolvable config is refused.
#[test]
fn unresolvable_config_is_refused() {
    let (repo, model) = new_variability_model();
    let (_o, e, code) = run(&model, &["baseline", "create", "--tag", "REL-2026-06", "--frozen-scope", "config=CONF-NONEXISTENT"]);
    assert_ne!(code, 0, "unresolvable config refused");
    assert!(e.to_lowercase().contains("config"), "error mentions the config: {e}");
    let _ = repo;
}

// TC-TRS-BL-010 — [baselines] config redirects element and manifest output.
#[test]
fn configured_dirs_redirect_output() {
    let (repo, model) = new_git_model();
    write(&model, ".syscribe.toml", "[baselines]\nelement_dir = \"Releases\"\nmanifest_dir = \"evidence\"\n");
    git(&repo, &["add", "-A"]);
    let _ = Command::new("git").arg("-C").arg(&repo).args(["commit", "-qm", "cfg"]).output();
    let (o, _e, code) = run(&model, &["baseline", "create", "--tag", "REL-2026-06"]);
    assert_eq!(code, 0, "create with configured dirs exits 0: {o}");
    assert!(model.join("Releases/BL-2026-06.md").exists(), "element under configured Releases/");
    assert!(repo.join("evidence/BL-2026-06.manifest.json").exists(), "manifest under configured evidence/");
    assert!(!model.join("Baselines").exists(), "default Baselines/ not used");
    // Self-recorded manifest → verify resolves it.
    let (_o2, _e2, code2) = run(&model, &["baseline", "verify", "BL-2026-06"]);
    assert_eq!(code2, 0, "verify resolves the redirected manifest");
}

// TC-TRS-BL-010 — an element_dir escaping the model root is rejected.
#[test]
fn escaping_element_dir_is_rejected() {
    let (repo, model) = new_git_model();
    write(&model, ".syscribe.toml", "[baselines]\nelement_dir = \"../outside\"\n");
    git(&repo, &["add", "-A"]);
    let _ = Command::new("git").arg("-C").arg(&repo).args(["commit", "-qm", "cfg"]).output();
    let (_o, e, code) = run(&model, &["baseline", "create", "--tag", "REL-2026-06"]);
    assert_ne!(code, 0, "escaping element_dir is refused");
    assert!(e.contains("escapes the model root"), "clear error: {e}");
    assert!(!repo.join("outside").exists(), "nothing written outside");
}

// TC-TRS-BL-001 — a malformed BL id is rejected.
#[test]
fn malformed_bl_id_is_rejected() {
    let (repo, model) = new_git_model();
    // single-char final segment fails ^BL(-[A-Z0-9]{2,12})+$
    let (_o, _e, code) = create(&repo, &model, "REL-2026-06", &["--id", "BL-X"]);
    assert_ne!(code, 0, "an invalid BL id is refused");
}
