//! TC-TRS-PHOLD-002 / GH #266.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Repo {
    root: PathBuf,
    model: PathBuf,
}

fn git(root: &Path, args: &[&str]) {
    let o = Command::new("git")
        .current_dir(root)
        .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn conf(id: &str, v: &str) -> String {
    format!("---\nid: {id}\ntype: Configuration\nname: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Display: true\nparameterBindings:\n  Features::Display.sizeInch: {v}\n---\n\nC.\n")
}

fn repo() -> Repo {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-phbl-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let model = root.join("model");
    let w = |rel: &str, c: &str| {
        let p = model.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Features/Display.md", "---\ntype: FeatureDef\nid: FEAT-PB-100\nname: Display\ngroupKind: optional\nparameters:\n  - {name: sizeInch, type: ScalarValues::Real, unit: in}\n---\n\nD.\n");
    w("Configs/CONF-PB-ALPHA-001.md", &conf("CONF-PB-ALPHA-001", "8"));
    w("Configs/CONF-PB-BRAVO-001.md", &conf("CONF-PB-BRAVO-001", "12"));
    w(
        "Reqs/REQ-PB-001.md",
        "---\nid: REQ-PB-001\ntype: Requirement\nname: r\nstatus: draft\nreqDomain: software\nappliesWhen: Features::Display\n---\n\nThe panel shall be {{Features::Display.sizeInch|unit}}.\n",
    );
    git(&root, &["init", "-q"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "init"]);
    Repo { root, model }
}

fn run(r: &Repo, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&r.model).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn commit(r: &Repo, m: &str) {
    git(&r.root, &["add", "-A"]);
    git(&r.root, &["commit", "-q", "-m", m]);
}

fn baseline(r: &Repo, id: &str, scope: Option<&str>) {
    let mut args = vec!["baseline", "create", "--tag", id, "--id", id];
    if let Some(s) = scope {
        args.extend(["--frozen-scope", s]);
    }
    let (o, c) = run(r, &args);
    assert_eq!(c, 0, "{o}");
    commit(r, id);
}

fn setup() -> Repo {
    let r = repo();
    baseline(&r, "BL-FULL", None);
    baseline(&r, "BL-ALPHA", Some("config=CONF-PB-ALPHA-001"));
    baseline(&r, "BL-BRAVO", Some("config=CONF-PB-BRAVO-001"));
    r
}

fn rebind_alpha(r: &Repo, v: &str) {
    std::fs::write(r.model.join("Configs/CONF-PB-ALPHA-001.md"), conf("CONF-PB-ALPHA-001", v)).unwrap();
}

/// The ids listed under "changed" in a `baseline diff <BL> --current` report.
fn changed(r: &Repo, bl: &str) -> Vec<String> {
    let (o, c) = run(r, &["baseline", "diff", bl, "--current"]);
    assert_eq!(c, 0, "{o}");
    let sec = o.split("## changed").nth(1).unwrap_or("");
    sec.lines().filter_map(|l| l.trim().strip_prefix('[')).filter_map(|l| l.split_once("] ").map(|(_, id)| id.to_string())).collect()
}

#[test]
fn a_binding_change_drifts_the_requirement_only_in_the_changed_configurations_baseline() {
    let r = setup();
    let (o, c) = run(&r, &["baseline", "verify", "--all"]);
    assert_eq!(c, 0, "{o}");
    rebind_alpha(&r, "9");
    // The Configuration file itself is content of every baseline that includes it; what differs
    // is the requirement's text: substituted (so changed) only for the alpha-scoped baseline.
    let alpha = changed(&r, "BL-ALPHA");
    assert!(alpha.iter().any(|i| i == "REQ-PB-001"), "{alpha:?}");
    let bravo = changed(&r, "BL-BRAVO");
    assert!(!bravo.iter().any(|i| i == "REQ-PB-001"), "{bravo:?}");
    let full = changed(&r, "BL-FULL");
    assert!(!full.iter().any(|i| i == "REQ-PB-001"), "the full-model baseline hashes the symbolic text: {full:?}");
}

#[test]
fn verify_detail_lists_the_requirement_for_the_changed_configuration() {
    let r = setup();
    rebind_alpha(&r, "9");
    let (o, c) = run(&r, &["baseline", "verify", "BL-ALPHA", "--detail"]);
    assert_eq!(c, 2, "{o}");
    assert!(o.contains("REQ-PB-001"), "{o}");
}

#[test]
fn a_binding_change_raises_no_suspect_link_finding_on_the_base_model() {
    let r = setup();
    rebind_alpha(&r, "9");
    let (o, _) = run(&r, &["validate"]);
    assert!(!o.contains("W090") && !o.contains("E520"), "{o}");
}
