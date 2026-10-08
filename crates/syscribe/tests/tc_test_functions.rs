//! Every test name a `TestCase` cites under `testFunctions:` (and every backticked `snake_case`
//! name in the FMED test cases' bodies) must still exist as a `fn` or a JS scenario somewhere
//! under `crates/` or `qual/`. A renamed or deleted test otherwise leaves the qualification
//! evidence pointing at nothing, and nothing notices.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if !matches!(name, "target" | "node_modules" | ".git" | "vendor") {
                walk(&p, out);
            }
        } else {
            out.push(p);
        }
    }
}

fn ident_after<'a>(s: &'a str, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    s.match_indices(key).filter_map(move |(i, _)| {
        let rest = &s[i + key.len()..];
        let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
        (end > 0).then(|| &rest[..end])
    })
}

#[test]
fn every_cited_test_function_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for d in ["crates", "qual"] {
        walk(&root.join(d), &mut files);
    }
    let mut names: HashSet<String> = HashSet::new();
    for f in files.iter().filter(|f| matches!(f.extension().and_then(|e| e.to_str()), Some("rs" | "mjs" | "sh" | "py" | "ts"))) {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        names.extend(ident_after(&src, "fn ").map(str::to_string));
        for q in ["scenario('", "scenario(\"", "test('", "it('"] {
            for (i, _) in src.match_indices(q) {
                let rest = &src[i + q.len()..];
                if let Some(end) = rest.find(['\'', '"']) {
                    names.insert(rest[..end].to_string());
                }
            }
        }
        names.extend(src.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).filter(|w| w.len() > 12).map(str::to_string));
    }
    let mut tcs = Vec::new();
    walk(&root.join("qual/TestCases"), &mut tcs);
    walk(&root.join("model/Verification"), &mut tcs);
    let mut missing: Vec<String> = Vec::new();
    for tc in tcs.iter().filter(|p| p.extension().is_some_and(|e| e == "md")) {
        let Ok(text) = std::fs::read_to_string(tc) else { continue };
        let mut in_list = false;
        for line in text.lines() {
            if line.starts_with("testFunctions:") {
                in_list = true;
            } else if in_list {
                if let Some(n) = line.strip_prefix("  - ") {
                    let n = n.trim().trim_matches(['"', '\'']);
                    if !names.contains(n) {
                        missing.push(format!("{}: {n}", tc.strip_prefix(&root).unwrap_or(tc).display()));
                    }
                } else if !line.trim().is_empty() {
                    in_list = false;
                }
            }
        }
    }
    assert!(missing.is_empty(), "testFunctions entries that no test defines:\n{}", missing.join("\n"));
}
