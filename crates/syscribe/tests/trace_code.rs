//! TC-TRS-TRACECODE-001..004 (GH #451): `trace-code` lists code units that no `implementedBy:` or
//! `sourceFile:` names, against a shrink-only baseline; `[traceCode] validate = true` adds W326/W327;
//! `[repos]` peers are checked against their own elements.

mod common;
use common::Mcp;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const CONFIG: &str = r#"
[traceCode]
roots = ["crates", "scripts", "tools"]
exclude = ["crates/vendored/**"]
baseline = "trace-code-baseline.toml"
units = [
  { kind = "cargo-crate", under = "crates" },
  { kind = "file", under = "scripts", extensions = ["sh", "py"] },
  { kind = "build-dir", under = "tools" },
]
"#;

struct Repo {
    root: PathBuf,
}

impl Repo {
    fn bare() -> Repo {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-tracecode-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { root };
        r.git(&["init", "-q", "-b", "main"]);
        r
    }

    /// A model plus a source tree: `alpha` is named through a file inside it, `gamma` by a TestCase's
    /// `sourceFile:`, `tools/web` by a directory above it, `scripts/deploy.py` directly; `beta` and
    /// `scripts/build.sh` are named by nothing; `crates/vendored` is excluded; `target/` is ignored.
    fn new(config: &str) -> Repo {
        let r = Repo::bare();
        r.write("model/.syscribe.toml", config);
        r.write("model/_index.md", "---\ntype: Package\nname: Root\n---\n");
        r.write("model/REQ-TCD-001.md", "---\nid: REQ-TCD-001\ntype: Requirement\nname: Gamma\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nGamma shall work.\n");
        r.write("model/Alpha.md", "---\ntype: PartDef\nname: Alpha\ndomain: software\nimplementedBy: [repo:crates/alpha/src/lib.rs]\n---\n\nAlpha.\n");
        r.write("model/Web.md", "---\ntype: PartDef\nname: Web\ndomain: software\nimplementedBy: [repo:tools/]\n---\n\nThe tools tree.\n");
        r.write("model/Deploy.md", "---\ntype: PartDef\nname: Deploy\ndomain: software\nimplementedBy:\n  - repo:scripts/deploy.py\n  - crates.io:serde@1.0.0\n  - https://example.com/remote.c\n---\n\nDeploy script.\n");
        r.write("model/TC-TCD-001.md", "---\nid: TC-TCD-001\ntype: TestCase\nname: Gamma test\nstatus: active\ntestLevel: L2\nverifies: [REQ-TCD-001]\nsourceFile: repo:crates/gamma/tests/t.rs\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
        for c in ["alpha", "beta", "gamma"] {
            r.write(&format!("crates/{c}/Cargo.toml"), &format!("[package]\nname = \"{c}\"\nversion = \"0.1.0\"\n"));
            r.write(&format!("crates/{c}/src/lib.rs"), "pub fn f() {}\n");
        }
        r.write("crates/gamma/tests/t.rs", "#[test]\nfn t() {}\n");
        r.write("crates/beta/target/debug/Cargo.toml", "[package]\nname = \"junk\"\n");
        r.write("crates/vendored/Cargo.toml", "[package]\nname = \"vendored\"\n");
        r.write("scripts/build.sh", "#!/bin/sh\n");
        r.write("scripts/deploy.py", "print('x')\n");
        r.write("scripts/README.md", "not code\n");
        r.write("tools/web/package.json", "{}\n");
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
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap_or_default()
    }
    fn sy(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).current_dir(&self.root).arg("-m").arg(self.model()).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
    fn json(&self, args: &[&str], want: i32) -> Value {
        let mut a = vec!["trace-code"];
        a.extend_from_slice(args);
        a.push("--json");
        let (out, err, code) = self.sy(&a);
        assert_eq!(code, want, "{out}\n{err}");
        serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out} / {err}"))
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn paths(v: &Value) -> Vec<String> {
    let mut p: Vec<String> = v.as_array().unwrap().iter().map(|e| e["path"].as_str().unwrap().to_string()).collect();
    p.sort();
    p
}

fn assert_schema(v: &Value) {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/trace-code.schema.json")).unwrap();
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    let url = "https://syscribe.dev/schemas/trace-code.schema.json";
    compiler.add_resource(url, serde_json::from_str(&text).unwrap()).unwrap();
    let idx = compiler.compile(url, &mut schemas).unwrap();
    schemas.validate(v, idx).unwrap_or_else(|e| panic!("{e:#}\n{v}"));
}

#[test]
fn untraced_units_follow_the_unit_rules_and_the_containment_rule() {
    let r = Repo::new(CONFIG);
    let v = r.json(&[], 2);
    assert_schema(&v);
    // Named: alpha (a file inside), gamma (sourceFile), tools/web (a directory above it), deploy.py (itself).
    // Not units: README.md (extension), crates/vendored (excluded), anything under target/.
    assert_eq!(paths(&v["untraced"]), ["crates/beta", "scripts/build.sh"], "{v}");
    let beta = v["untraced"].as_array().unwrap().iter().find(|u| u["path"] == "crates/beta").unwrap();
    assert_eq!(beta["unit"], "cargo-crate");
    assert_eq!(beta["inBaseline"], false);
    assert_eq!(v["stale"], json!([]));
    assert!(v.get("traced").is_none(), "traced only with --all: {v}");

    let all = r.json(&["--all"], 2);
    assert_schema(&all);
    assert_eq!(paths(&all["traced"]), ["crates/alpha", "crates/gamma", "scripts/deploy.py", "tools/web"]);
    let gamma = all["traced"].as_array().unwrap().iter().find(|u| u["path"] == "crates/gamma").unwrap();
    assert_eq!(gamma["namedBy"][0]["element"], "TC-TCD-001");
    assert_eq!(gamma["namedBy"][0]["via"], "sourceFile");
    let web = all["traced"].as_array().unwrap().iter().find(|u| u["path"] == "tools/web").unwrap();
    assert_eq!(web["unit"], "build-dir");

    let (out, _, code) = r.sy(&["trace-code"]);
    assert_eq!(code, 2);
    assert!(out.contains("crates/beta") && out.contains("scripts/build.sh") && out.contains("--write-baseline"), "{out}");
}

#[test]
fn the_baseline_only_shrinks() {
    let r = Repo::new(CONFIG);
    let (out, err, code) = r.sy(&["trace-code", "--write-baseline"]);
    assert_eq!(code, 0, "{out}{err}");
    let b = r.read("model/trace-code-baseline.toml");
    assert!(b.contains("path = \"crates/beta\"") && b.contains("reason = \"TODO\"") && b.contains("scripts/build.sh"), "{b}");

    // Everything untraced is now in the baseline: clean.
    let v = r.json(&[], 0);
    assert!(v["untraced"].as_array().unwrap().iter().all(|u| u["inBaseline"] == true), "{v}");

    // A new untraced unit fails again.
    r.write("crates/delta/Cargo.toml", "[package]\nname = \"delta\"\n");
    let v = r.json(&[], 2);
    let delta = v["untraced"].as_array().unwrap().iter().find(|u| u["path"] == "crates/delta").unwrap();
    assert_eq!(delta["inBaseline"], false);

    // A baseline entry that becomes traced is stale (the list only shrinks), and so is one that vanished.
    std::fs::remove_dir_all(r.root.join("crates/delta")).unwrap();
    r.write("model/Beta.md", "---\ntype: PartDef\nname: Beta\ndomain: software\nimplementedBy: [repo:crates/beta]\n---\n\nBeta.\n");
    std::fs::remove_file(r.root.join("scripts/build.sh")).unwrap();
    let v = r.json(&[], 2);
    assert_schema(&v);
    let why: Vec<(String, String)> = v["stale"].as_array().unwrap().iter().map(|s| (s["path"].as_str().unwrap().into(), s["why"].as_str().unwrap().into())).collect();
    assert!(why.contains(&("crates/beta".into(), "traced".into())), "{why:?}");
    assert!(why.contains(&("scripts/build.sh".into(), "missing".into())), "{why:?}");
    assert_eq!(v["stale"][0]["reason"], "TODO");

    // Refreshing keeps hand-written reasons, drops stale entries and adds new ones as TODO.
    std::fs::remove_file(r.root.join("model/Beta.md")).unwrap();
    r.write("scripts/build.sh", "#!/bin/sh\n");
    let b = r.read("model/trace-code-baseline.toml").replace("reason = \"TODO\"", "reason = \"legacy, design pending\"");
    r.write("model/trace-code-baseline.toml", &b);
    r.write("scripts/new.sh", "#!/bin/sh\n");
    r.write("model/Build.md", "---\ntype: PartDef\nname: Build\ndomain: software\nimplementedBy: [repo:scripts/build.sh]\n---\n\nBuild.\n");
    let (_, err, code) = r.sy(&["trace-code", "--write-baseline"]);
    assert_eq!(code, 0, "{err}");
    let b = r.read("model/trace-code-baseline.toml");
    assert!(b.contains("crates/beta") && b.contains("legacy, design pending"), "reason kept: {b}");
    assert!(!b.contains("scripts/build.sh"), "stale entry dropped: {b}");
    assert!(b.contains("scripts/new.sh") && b.contains("reason = \"TODO\""), "new entry added: {b}");
    r.json(&[], 0);
}

#[test]
fn configuration_errors_exit_1() {
    // No [traceCode] table: exit 1 with a message saying how to configure it.
    let r = Repo::new("");
    let (_, err, code) = r.sy(&["trace-code"]);
    assert_eq!(code, 1);
    assert!(err.contains("[traceCode]") && err.contains("roots"), "{err}");

    // A malformed entry is a config error for trace-code and W328 for validate.
    let r = Repo::new("[traceCode]\nroots = [\"crates\"]\nunits = [{ kind = \"module\" }]\nbogus = 1\n");
    let (_, err, code) = r.sy(&["trace-code"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("module") && err.contains("bogus"), "{err}");
    let (out, _, _) = r.sy(&["validate"]);
    assert!(out.contains("W328"), "{out}");

    // An unreadable baseline is a config error too.
    let r = Repo::new(CONFIG);
    r.write("model/trace-code-baseline.toml", "[[untraced]]\nreason = \"no path\"\n");
    let (_, err, code) = r.sy(&["trace-code"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("trace-code-baseline.toml"), "{err}");

    // Bad flags.
    let r = Repo::new(CONFIG);
    let (_, _, code) = r.sy(&["trace-code", "--nope"]);
    assert_eq!(code, 1);
}

#[test]
fn validate_raises_w326_and_w327_only_when_switched_on() {
    let r = Repo::new(CONFIG);
    let (out, _, _) = r.sy(&["validate"]);
    assert!(!out.contains("W326") && !out.contains("W327") && !out.contains("W328"), "off by default: {out}");

    let r = Repo::new(&format!("{CONFIG}validate = true\n"));
    r.write("model/trace-code-baseline.toml", "[[untraced]]\npath = \"scripts/build.sh\"\nreason = \"legacy\"\n\n[[untraced]]\npath = \"crates/alpha\"\nreason = \"was untraced\"\n");
    let (out, _, code) = r.sy(&["validate"]);
    assert_eq!(code, 0, "warnings only: {out}");
    assert!(out.contains("W326") && out.contains("crates/beta"), "{out}");
    assert!(!out.lines().any(|l| l.contains("W326") && l.contains("scripts/build.sh")), "baselined: {out}");
    assert!(out.contains("W327") && out.contains("crates/alpha"), "{out}");
    let (_, _, code) = r.sy(&["validate", "--deny", "W326"]);
    assert_eq!(code, 2);
    let (_, _, code) = r.sy(&["validate", "--deny", "W327"]);
    assert_eq!(code, 2);
}

#[test]
fn write_baseline_is_a_guarded_write() {
    let r = Repo::new(&format!("{CONFIG}\n[agents]\nprotected_paths = [\"model/trace-code-baseline.toml\"]\n"));
    let (_, err, code) = r.sy(&["trace-code", "--write-baseline", "--by", "bot-1"]);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("protected_paths"), "{err}");
    assert!(!r.root.join("model/trace-code-baseline.toml").exists());
    // A person may.
    let (_, err, code) = r.sy(&["trace-code", "--write-baseline", "--by", "alice", "--as-human"]);
    assert_eq!(code, 0, "{err}");
    assert!(r.root.join("model/trace-code-baseline.toml").exists());
}

#[test]
fn composed_repositories_are_checked_against_their_own_elements() {
    let r = Repo::new(&format!("{CONFIG}\n[repos.comp]\npath = \"../../peer\"\nroot = \"model\"\n"));
    r.write("model/trace-code-baseline.toml", "[[untraced]]\npath = \"crates/beta\"\nreason = \"x\"\n\n[[untraced]]\npath = \"scripts/build.sh\"\nreason = \"x\"\n");
    // The peer is its own repository beside this one, with its own model, code and [traceCode].
    let peer = r.root.join("../peer");
    let peer = {
        let _ = std::fs::remove_dir_all(&peer);
        std::fs::create_dir_all(&peer).unwrap();
        peer.canonicalize().unwrap()
    };
    let w = |rel: &str, c: &str| {
        let p = peer.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    assert!(Command::new("git").arg("-C").arg(&peer).args(["init", "-q"]).status().unwrap().success());
    w("model/.syscribe.toml", "[traceCode]\nroots = [\"libs\"]\nunits = [\"cargo-crate\"]\n");
    w("model/_index.md", "---\ntype: Package\nname: Peer\n---\n");
    w("model/Engine.md", "---\ntype: PartDef\nname: Engine\ndomain: software\nimplementedBy: [repo:libs/engine]\n---\n\nE.\n");
    w("libs/engine/Cargo.toml", "[package]\nname = \"engine\"\n");
    // Named by an element of the *composing* model only: still untraced in the peer.
    w("libs/orphan/Cargo.toml", "[package]\nname = \"orphan\"\n");
    r.write("model/Orphan.md", &format!("---\ntype: PartDef\nname: Orphan\ndomain: software\nimplementedBy: [{}]\n---\n\nO.\n", peer.join("libs/orphan").display()));

    let v = r.json(&[], 2);
    assert_schema(&v);
    assert!(v["untraced"].as_array().unwrap().iter().all(|u| u["inBaseline"] == true), "local repo is clean: {v}");
    let peers = v["peers"].as_array().unwrap();
    assert_eq!(peers.len(), 1, "{v}");
    assert_eq!(peers[0]["alias"], "comp");
    assert_eq!(paths(&peers[0]["untraced"]), ["libs/orphan"]);
    let (out, _, _) = r.sy(&["trace-code"]);
    assert!(out.contains("comp") && out.contains("libs/orphan"), "{out}");
    let _ = std::fs::remove_dir_all(&peer);
}

#[test]
fn mcp_trace_code_tool() {
    let r = Repo::new(CONFIG);
    let mut mcp = Mcp::start(&r.model());
    mcp.initialize();
    let res = mcp.call_tool("trace_code", json!({"all": true}));
    assert_eq!(paths(&res["untraced"]), ["crates/beta", "scripts/build.sh"], "{res}");
    assert!(res["traced"].as_array().is_some_and(|t| t.len() == 4), "{res}");
}

#[test]
fn trace_code_is_discoverable() {
    let r = Repo::new(CONFIG);
    for kw in ["untraced", "code without design", "implementedBy", "trace code"] {
        let (out, _, code) = r.sy(&["help", "--search", kw]);
        assert_eq!(code, 0);
        assert!(out.contains("trace-code"), "{kw}: {out}");
    }
    for c in ["W326", "W327", "W328"] {
        let (out, _, code) = r.sy(&["explain", c]);
        assert_eq!(code, 0, "{c}");
        assert!(out.contains("trace-code") || out.contains("[traceCode]"), "{c}: {out}");
    }
    let (out, _, code) = r.sy(&["trace-code", "--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("SYNOPSIS"), "{out}");
}
