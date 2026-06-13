//! `lint-docs` — scans external Markdown files for stable-ID tokens and reports
//! those that do not resolve to a known model element (W099).

use std::path::Path;
use syscribe_model::element::RawElement;

/// Regex-like pattern: tokens matching stable-ID scheme.
/// `^(REQ|TC|ADR|FEAT|FM|FTE|AOU|SG|CM)(-[A-Z0-9]{2,12})+(-[0-9]{3,8})?$`
fn is_stable_id(token: &str) -> bool {
    static PREFIXES: &[&str] = &["REQ", "TC", "ADR", "FEAT", "FM", "FTE", "AOU", "SG", "CM"];
    let Some(prefix) = PREFIXES.iter().find(|&&p| token.starts_with(p) && token[p.len()..].starts_with('-')) else {
        return false;
    };
    // Remaining after prefix: must be one or more -SEG groups, optionally ending in -NNN..
    let rest = &token[prefix.len()..]; // starts with '-'
    let parts: Vec<&str> = rest[1..].split('-').collect();
    if parts.is_empty() { return false; }
    for part in &parts {
        if part.is_empty() { return false; }
        let is_numeric = part.chars().all(|c| c.is_ascii_digit());
        let is_alphanum_upper = part.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
        if !is_alphanum_upper { return false; }
        // Segment length: 2-12 chars for named segments, 3-8 for numeric suffix
        if is_numeric {
            if part.len() < 3 || part.len() > 8 { return false; }
        } else if part.len() < 2 || part.len() > 12 {
            return false;
        }
    }
    true
}

/// Extract all stable-ID candidate tokens from a line.
fn extract_candidates(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // find a word boundary start
        if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
            let start = i;
            // extend to end of token (alphanumeric, -, _)
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
                i += 1;
            }
            let token = &line[start..i];
            // Only tokens starting with an uppercase letter are candidates.
            if token.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
                out.push(token);
            }
        } else {
            i += 1;
        }
    }
    let _ = chars; // suppress unused warning
    out
}

struct Finding {
    file: String,
    line: usize,
    token: String,
}

fn scan_file(path: &Path, elements: &[RawElement], findings: &mut Vec<Finding>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for (line_no, line) in content.lines().enumerate() {
        for token in extract_candidates(line) {
            if !is_stable_id(token) { continue; }
            // Resolve against the model: match by id or qualified name.
            let resolved = elements.iter().any(|e| {
                e.frontmatter.id.as_deref() == Some(token)
                    || e.qualified_name == token
            });
            if !resolved {
                findings.push(Finding {
                    file: path.display().to_string(),
                    line: line_no + 1,
                    token: token.to_string(),
                });
            }
        }
    }
}

fn scan_path(path: &Path, elements: &[RawElement], findings: &mut Vec<Finding>) {
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else { return; };
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            scan_path(&p, elements, findings);
        }
    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
        scan_file(path, elements, findings);
    }
}

pub fn cmd_lint_docs(elements: &[RawElement], paths: &[&str], json: bool) -> i32 {
    let mut findings: Vec<Finding> = Vec::new();

    for &path_str in paths {
        let p = Path::new(path_str);
        if !p.exists() {
            eprintln!("lint-docs: path '{}' does not exist", path_str);
        }
        scan_path(p, elements, &mut findings);
    }

    if findings.is_empty() {
        return 0;
    }

    if json {
        let items: Vec<serde_json::Value> = findings
            .iter()
            .map(|f| serde_json::json!({
                "file": f.file,
                "line": f.line,
                "code": "W099",
                "token": f.token,
            }))
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    } else {
        for f in &findings {
            println!("{}:{}: W099: unresolvable ID token '{}' referenced in external doc", f.file, f.line, f.token);
        }
    }

    1
}
