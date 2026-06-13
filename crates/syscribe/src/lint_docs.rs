//! `lint-docs` — scans external Markdown (and SVG) files for references to model elements
//! that do not resolve. Stable-ID tokens in prose (W099); qualified names inside
//! ```mermaid blocks (W100); `sysml:ref` in SVG (W101); local image/diagram embed paths
//! (W102). Qualified names in *prose* are intentionally not resolved (false-positive prone).

use std::path::Path;
use syscribe_model::element::RawElement;

/// `^(REQ|TC|ADR|FEAT|FM|FTE|AOU|SG|CM)(-[A-Z0-9]{2,12})+(-[0-9]{3,8})?$`
fn is_stable_id(token: &str) -> bool {
    static PREFIXES: &[&str] = &["REQ", "TC", "ADR", "FEAT", "FM", "FTE", "AOU", "SG", "CM"];
    let Some(prefix) = PREFIXES.iter().find(|&&p| token.starts_with(p) && token[p.len()..].starts_with('-')) else {
        return false;
    };
    let rest = &token[prefix.len()..];
    let parts: Vec<&str> = rest[1..].split('-').collect();
    if parts.is_empty() { return false; }
    for part in &parts {
        if part.is_empty() { return false; }
        let is_numeric = part.chars().all(|c| c.is_ascii_digit());
        let is_alphanum_upper = part.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
        if !is_alphanum_upper { return false; }
        if is_numeric {
            if part.len() < 3 || part.len() > 8 { return false; }
        } else if part.len() < 2 || part.len() > 12 {
            return false;
        }
    }
    true
}

/// Stable-ID candidate tokens on a line (uppercase-initial, `[A-Za-z0-9_-]`).
fn extract_candidates(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
                i += 1;
            }
            let token = &line[start..i];
            if token.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
                out.push(token);
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Qualified-name-like tokens (`A::B::C`) on a line, excluding mermaid `:::class` syntax.
fn extract_qnames(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b':') {
                i += 1;
            }
            let tok = line[start..i].trim_matches(':');
            if tok.contains("::") && !tok.contains(":::") {
                out.push(tok.to_string());
            }
        } else {
            i += 1;
        }
    }
    out
}

struct Finding {
    file: String,
    line: usize,
    code: &'static str,
    detail: String,
}

fn resolves(token: &str, elements: &[RawElement]) -> bool {
    elements.iter().any(|e| e.frontmatter.id.as_deref() == Some(token) || e.qualified_name == token)
}

/// True for an embed path we should existence-check (a local relative/absolute file path).
fn is_local_path(p: &str) -> bool {
    let p = p.trim();
    !(p.is_empty()
        || p.starts_with("http://")
        || p.starts_with("https://")
        || p.starts_with("data:")
        || p.starts_with("mailto:")
        || p.starts_with('#'))
}

fn scan_md(path: &Path, elements: &[RawElement], findings: &mut Vec<Finding>) {
    let Ok(content) = std::fs::read_to_string(path) else { return };
    let dir = path.parent().unwrap_or(Path::new("."));
    let md_img = regex::Regex::new(r"!\[[^\]]*\]\(([^)\s]+)").unwrap();
    let html_img = regex::Regex::new(r#"<img[^>]*\bsrc=["']([^"']+)["']"#).unwrap();
    let mut in_mermaid = false;
    for (line_no, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if in_mermaid {
                in_mermaid = false; // any closing fence ends the block
            } else if trimmed.contains("mermaid") {
                in_mermaid = true;
            }
            continue;
        }
        // W099: unresolvable stable-ID tokens (prose + mermaid).
        for token in extract_candidates(line) {
            if is_stable_id(token) && !resolves(token, elements) {
                findings.push(Finding { file: path.display().to_string(), line: line_no + 1, code: "W099", detail: token.to_string() });
            }
        }
        // W100: qualified names inside mermaid blocks must resolve.
        if in_mermaid {
            for qn in extract_qnames(line) {
                if !resolves(&qn, elements) {
                    findings.push(Finding { file: path.display().to_string(), line: line_no + 1, code: "W100", detail: qn });
                }
            }
        }
        // W102: local image/diagram embed paths must exist.
        for cap in md_img.captures_iter(line).chain(html_img.captures_iter(line)) {
            let p = &cap[1];
            if is_local_path(p) && !p.starts_with('/') && !dir.join(p).exists() {
                findings.push(Finding { file: path.display().to_string(), line: line_no + 1, code: "W102", detail: p.to_string() });
            }
        }
    }
}

fn scan_svg(path: &Path, elements: &[RawElement], findings: &mut Vec<Finding>) {
    let Ok(content) = std::fs::read_to_string(path) else { return };
    let re = regex::Regex::new(r#"sysml:ref=["']([^"']+)["']"#).unwrap();
    for (line_no, line) in content.lines().enumerate() {
        for cap in re.captures_iter(line) {
            let r = &cap[1];
            if !resolves(r, elements) {
                findings.push(Finding { file: path.display().to_string(), line: line_no + 1, code: "W101", detail: r.to_string() });
            }
        }
    }
}

fn scan_path(path: &Path, elements: &[RawElement], findings: &mut Vec<Finding>) {
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else { return };
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            scan_path(&p, elements, findings);
        }
    } else {
        match path.extension().and_then(|e| e.to_str()) {
            Some("md") => scan_md(path, elements, findings),
            Some("svg") => scan_svg(path, elements, findings),
            _ => {}
        }
    }
}

fn message(code: &str, detail: &str) -> String {
    match code {
        "W100" => format!("Mermaid node reference '{}' does not resolve to a model element", detail),
        "W101" => format!("SVG sysml:ref '{}' does not resolve to a model element", detail),
        "W102" => format!("embedded image/diagram path '{}' does not exist", detail),
        _ => format!("unresolvable ID token '{}' referenced in external doc", detail),
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
            .map(|f| {
                // Detail key matches the existing W099 shape: token / ref / path by code.
                let key = match f.code {
                    "W099" => "token",
                    "W102" => "path",
                    _ => "ref",
                };
                serde_json::json!({ "file": f.file, "line": f.line, "code": f.code, key: f.detail })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    } else {
        for f in &findings {
            println!("{}:{}: {}: {}", f.file, f.line, f.code, message(f.code, &f.detail));
        }
    }
    1
}
