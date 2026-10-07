//! `REQ-TRS-SYSMLV2-071`: a repository-level ratchet over `syscribe export-sysml`. Every
//! `.md`-native behavioural element (`ActionDef`/`Action`/`StateDef`/`State`) of the repository's
//! own models is exported, and the number of entries written as `// … not exported (…)` comments
//! is compared with a recorded budget: more is a regression, fewer means the budget must be
//! lowered here so the gain is locked in.

use std::path::{Path, PathBuf};

use syscribe_model::element::ElementType;
use syscribe_model::sysmlv2::export::export_sysml;
use syscribe_model::walker::walk_model;

/// Degraded behaviour entries over every repository model root. Four (`model_sil`'s guarded
/// `successionConnections:` entries) until `REQ-TRS-SYSMLV2-074`: the 0.57 parser reads
/// `first a if g then b;`, so they now export and read back identically.
const BUDGET: usize = 0;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn find_model_dirs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        // `wasm-plugins` is a parked, possibly untracked design: not part of the gate.
        if p.file_name().is_some_and(|n| n == "wasm-plugins") {
            continue;
        }
        if p.file_name().is_some_and(|n| n == "model") && p.join("_index.md").exists() {
            out.push(p.clone());
        }
        find_model_dirs(&p, out);
    }
}

fn roots() -> Vec<PathBuf> {
    let r = repo_root();
    let mut v: Vec<PathBuf> = ["model", "model_auto", "model_mg", "model_sil"].iter().map(|d| r.join(d)).filter(|p| p.is_dir()).collect();
    find_model_dirs(&r.join("examples"), &mut v);
    v
}

#[test]
fn behaviour_export_degradations_do_not_exceed_the_budget() {
    let mut total = 0usize;
    let mut behavioural = 0usize;
    let mut detail = Vec::new();
    for root in roots() {
        let mut els = walk_model(&root).unwrap();
        // Only `.md`-native elements: a `.sysml`-ingested one is the export's input form already.
        els.retain(|e| !e.file_path.ends_with(".sysml") && !e.file_path.ends_with(".kerml"));
        behavioural += els
            .iter()
            .filter(|e| {
                matches!(
                    e.frontmatter.element_type,
                    Some(ElementType::ActionDef | ElementType::Action | ElementType::StateDef | ElementType::State)
                )
            })
            .count();
        let out = export_sysml(&els, None).unwrap();
        if out.report.degraded_behaviour > 0 {
            for l in out.text.lines().filter(|l| l.trim_start().starts_with("// ") && l.contains(" not exported (")) {
                detail.push(format!("{}: {}", root.display(), l.trim()));
            }
        }
        total += out.report.degraded_behaviour;
    }
    assert!(behavioural >= 8, "the ratchet must see the repository's behavioural elements (saw {behavioural})");
    assert!(total <= BUDGET, "behaviour export regressed: {total} degraded entries, budget {BUDGET}\n{}", detail.join("\n"));
    assert!(total >= BUDGET, "behaviour export improved to {total}: lower BUDGET in this file from {BUDGET}\n{}", detail.join("\n"));
}
