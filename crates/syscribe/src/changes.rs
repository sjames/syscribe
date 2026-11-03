//! `changes <revA> [<revB>]` — what changed in the model between two git
//! revisions (GH #433, `REQ-TRS-MODELDIFF-001`). Read-only; no baseline needed.

use std::path::Path;
use syscribe_model::element::RawElement;
use syscribe_model::gitrev;
use syscribe_model::modeldiff::{self, ChangeKind, Filter};
use syscribe_model::walker;

const USAGE: &str = "Usage: syscribe -m <root> changes <revA> [<revB>] [--types T1,T2] [--package <qname>] [--json]";

/// `args` are the tokens after `changes`. Returns the process exit code.
pub fn cmd_changes(model_root: &Path, working: &[RawElement], args: &[String]) -> i32 {
    let mut revs: Vec<&str> = Vec::new();
    let mut json = false;
    let mut filter = Filter::default();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--json" => json = true,
            "--types" | "--package" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("Error: {a} expects a value\n{USAGE}");
                    return 1;
                };
                if a == "--types" {
                    filter.types.extend(v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
                } else {
                    filter.package = Some(v.clone());
                }
                i += 1;
            }
            _ if a.starts_with("--types=") => {
                filter.types.extend(a["--types=".len()..].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
            }
            _ if a.starts_with("--package=") => filter.package = Some(a["--package=".len()..].to_string()),
            _ if a.starts_with("--") => {
                eprintln!("Error: unknown option `{a}`\n{USAGE}");
                return 1;
            }
            _ => revs.push(a),
        }
        i += 1;
    }
    if revs.is_empty() || revs.len() > 2 {
        eprintln!("{USAGE}");
        return 1;
    }

    let load = |rev: &str| -> Result<(gitrev::RevModel, Vec<RawElement>), String> {
        let m = gitrev::materialize(model_root, rev)?;
        let elems = walker::walk_model(&m.root).map_err(|e| format!("cannot load the model at `{rev}`: {e}"))?;
        Ok((m, elems))
    };
    let (_old_guard, old) = match load(revs[0]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };
    let old_root = _old_guard.root.clone();
    let mut _new_guard: Option<gitrev::RevModel> = None;
    let mut new_loaded: Vec<RawElement> = Vec::new();
    let to_label = match revs.get(1) {
        Some(r) => match load(r) {
            Ok((g, e)) => {
                _new_guard = Some(g);
                new_loaded = e;
                (*r).to_string()
            }
            Err(e) => {
                eprintln!("Error: {e}");
                return 1;
            }
        },
        None => "working tree".to_string(),
    };
    let new: &[RawElement] = if revs.len() == 2 { &new_loaded } else { working };

    let mut set = modeldiff::diff(&old, new, &filter);
    // A revision is exported to a temporary directory; show every file relative to its model root.
    let roots: Vec<String> = [Some(old_root.clone()), _new_guard.as_ref().map(|g| g.root.clone())]
        .into_iter()
        .flatten()
        .map(|p| format!("{}/", p.display()))
        .collect();
    for c in &mut set.changes {
        if let Some(r) = roots.iter().find(|r| c.file.starts_with(r.as_str())) {
            c.file = c.file[r.len()..].to_string();
        }
    }

    if json {
        let doc = serde_json::json!({
            "schemaVersion": "1.0",
            "from": revs[0],
            "to": to_label,
            "changes": set.changes,
            "newlyBroken": set.newly_broken,
            "newlyResolved": set.newly_resolved,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        return 0;
    }

    if set.changes.is_empty() && set.newly_broken.is_empty() && set.newly_resolved.is_empty() {
        println!("No model changes between {} and {}.", revs[0], to_label);
        return 0;
    }
    println!("# Model changes: {} → {}", revs[0], to_label);
    println!();
    println!("{} element change(s).", set.changes.len());
    println!();
    for c in &set.changes {
        let mark = match c.kind {
            ChangeKind::Added => "+",
            ChangeKind::Removed => "-",
            ChangeKind::Moved => ">",
            ChangeKind::Modified => "~",
        };
        println!("- {mark} `{}` ({}) {}", c.id, c.element_type, c.summary);
    }
    let refs = |label: &str, rs: &[modeldiff::RefChange]| {
        if !rs.is_empty() {
            println!();
            println!("{label} ({}):", rs.len());
            for r in rs {
                println!("- `{}` {} → `{}`", r.element, r.field, r.target);
            }
        }
    };
    refs("Newly broken references", &set.newly_broken);
    refs("Newly resolved references", &set.newly_resolved);
    0
}
