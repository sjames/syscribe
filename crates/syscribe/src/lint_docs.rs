//! `lint-docs` — scans external Markdown (and SVG) files for references to model elements
//! that do not resolve. Stable-ID tokens in prose (W099); qualified names inside
//! ```mermaid blocks (W100); `sysml:ref` in SVG (W101); local image/diagram embed paths
//! (W102). Qualified names in *prose* are intentionally not resolved (false-positive prone).
//!
//! Plus one advisory hint (ADR-SYS-PKG-001, REQ-TRS-PKG-002, GH #120): `W103` when a
//! package's own `_index.md` hand-enumerates three or more of that package's direct
//! members — membership is generated (`show <pkg>`), so such a list only drifts.
//! `W103` never changes the exit status; only `W099`–`W102` do — unless the caller opts
//! in with `--deny W103` (issue #130).

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use syscribe_model::element::RawElement;

/// REQ-TRS-PKG-002 — how many distinct direct-member ids an `_index.md` body may
/// mention before `W103` suggests the generated listing instead.
const W103_THRESHOLD: usize = 3;

/// Scan context shared by every file of one `lint-docs` run.
struct Ctx<'a> {
    elements: &'a [RawElement],
    /// Canonicalized `_index.md` path of each package element → its qualified name.
    package_index: HashMap<PathBuf, &'a str>,
}

impl<'a> Ctx<'a> {
    fn new(elements: &'a [RawElement]) -> Self {
        let package_index = elements
            .iter()
            .filter(|e| syscribe_model::members::is_package(e))
            .filter(|e| Path::new(&e.file_path).file_name().is_some_and(|n| n == "_index.md"))
            .filter_map(|e| Some((std::fs::canonicalize(&e.file_path).ok()?, e.qualified_name.as_str())))
            .collect();
        Ctx { elements, package_index }
    }
}

/// The body of a Markdown file with its leading `---` YAML frontmatter removed,
/// as `(first body line number (0-based), body lines)`.
fn body_lines(content: &str) -> (usize, Vec<&str>) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first().map(|l| l.trim_end()) == Some("---") {
        if let Some(end) = lines.iter().skip(1).position(|l| l.trim_end() == "---") {
            let start = end + 2;
            return (start, lines[start..].to_vec());
        }
    }
    (0, lines)
}

/// REQ-TRS-PKG-002 — `W103`: the distinct stable ids an `_index.md` body mentions
/// that resolve to direct members of package `pkg`, with the 1-based line of the
/// first such mention. Frontmatter is ignored; ids of other packages' elements
/// (including grandchildren) do not count.
fn enumerated_members(content: &str, pkg: &str, elements: &[RawElement]) -> (BTreeSet<String>, usize) {
    let member_ids: BTreeSet<&str> = syscribe_model::members::direct_members(elements, pkg)
        .iter()
        .filter_map(|e| e.frontmatter.id.as_deref())
        .collect();
    let (offset, lines) = body_lines(content);
    let mut found = BTreeSet::new();
    let mut first_line = 0;
    for (i, line) in lines.iter().enumerate() {
        for token in extract_candidates(line) {
            if member_ids.contains(token) && found.insert(token.to_string()) && first_line == 0 {
                first_line = offset + i + 1;
            }
        }
    }
    (found, first_line)
}

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

fn scan_md(path: &Path, ctx: &Ctx, findings: &mut Vec<Finding>) {
    let elements = ctx.elements;
    let Ok(content) = std::fs::read_to_string(path) else { return };
    // W103 (advisory): a package's own `_index.md` hand-enumerating its members.
    if path.file_name().is_some_and(|n| n == "_index.md") {
        let pkg = std::fs::canonicalize(path).ok().and_then(|c| ctx.package_index.get(&c).copied());
        if let Some(pkg) = pkg {
            let (ids, line) = enumerated_members(&content, pkg, elements);
            if ids.len() >= W103_THRESHOLD {
                let shown = if pkg.is_empty() { "<model root>" } else { pkg };
                findings.push(Finding {
                    file: path.display().to_string(),
                    line,
                    code: "W103",
                    detail: format!(
                        "_index.md hand-enumerates {} of this package's members — membership is generated; see `syscribe show {}`",
                        ids.len(),
                        shown
                    ),
                });
            }
        }
    }
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

fn scan_svg(path: &Path, ctx: &Ctx, findings: &mut Vec<Finding>) {
    let elements = ctx.elements;
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

fn scan_path(path: &Path, ctx: &Ctx, findings: &mut Vec<Finding>) {
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else { return };
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            scan_path(&p, ctx, findings);
        }
    } else {
        match path.extension().and_then(|e| e.to_str()) {
            Some("md") => scan_md(path, ctx, findings),
            Some("svg") => scan_svg(path, ctx, findings),
            _ => {}
        }
    }
}

fn message(code: &str, detail: &str) -> String {
    match code {
        "W100" => format!("Mermaid node reference '{}' does not resolve to a model element", detail),
        "W101" => format!("SVG sysml:ref '{}' does not resolve to a model element", detail),
        "W102" => format!("embedded image/diagram path '{}' does not exist", detail),
        "W103" => detail.to_string(),
        _ => format!("unresolvable ID token '{}' referenced in external doc", detail),
    }
}

/// JSON detail key per code: `token` (W099), `path` (W102), `detail` (W103, the
/// advisory message), `ref` otherwise — existing shapes unchanged.
fn detail_key(code: &str) -> &'static str {
    match code {
        "W099" => "token",
        "W102" => "path",
        "W103" => "detail",
        _ => "ref",
    }
}

/// Scan `paths` and return the findings as JSON values
/// (`{file, line, code, token|ref|path|detail}`), optionally filtered to `codes`.
/// The value producer shared by the CLI `lint-docs --json` and the MCP `lint_docs` tool.
pub fn lint_docs_findings(
    elements: &[RawElement],
    paths: &[&str],
    codes: Option<&[String]>,
) -> Vec<serde_json::Value> {
    let ctx = Ctx::new(elements);
    let mut findings: Vec<Finding> = Vec::new();
    for &path_str in paths {
        scan_path(Path::new(path_str), &ctx, &mut findings);
    }
    findings
        .iter()
        .filter(|f| codes.is_none_or(|cs| cs.iter().any(|c| c == f.code)))
        .map(|f| serde_json::json!({ "file": f.file, "line": f.line, "code": f.code, detail_key(f.code): f.detail }))
        .collect()
}

/// Every code `lint-docs` can emit, i.e. the valid `--deny` values.
pub const LINT_CODES: [&str; 5] = ["W099", "W100", "W101", "W102", "W103"];

/// Parsed `lint-docs` command line (issue #130).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct LintArgs {
    pub paths: Vec<String>,
    pub json: bool,
    /// Codes named by `--deny` (validated against [`LINT_CODES`]).
    pub deny: BTreeSet<String>,
}

/// Parse `lint-docs` arguments: `<path>...`, `--json`, `--deny <CODES>` /
/// `--deny=<CODES>` (comma-separated, repeatable). The `--deny` value is never a
/// path. An unknown option, a missing `--deny` value, or a code outside
/// [`LINT_CODES`] is `Err(message)` (a usage error).
pub fn parse_lint_args(args: &[String]) -> Result<LintArgs, String> {
    let mut out = LintArgs::default();
    let add_codes = |val: &str, out: &mut LintArgs| -> Result<(), String> {
        for c in val.split(',').map(str::trim).filter(|c| !c.is_empty()) {
            if !LINT_CODES.contains(&c) {
                return Err(format!(
                    "--deny code '{c}' is not a lint-docs code; valid codes: {}",
                    LINT_CODES.join(", ")
                ));
            }
            out.deny.insert(c.to_string());
        }
        Ok(())
    };
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--json" {
            out.json = true;
        } else if a == "--deny" {
            let val = args
                .get(i + 1)
                .ok_or_else(|| format!("--deny expects a comma-separated code list ({})", LINT_CODES.join(", ")))?;
            add_codes(val, &mut out)?;
            i += 1;
        } else if let Some(val) = a.strip_prefix("--deny=") {
            add_codes(val, &mut out)?;
        } else if a.starts_with('-') && a != "-" {
            return Err(format!("unknown option '{a}' for lint-docs (expected <path>..., --json, --deny <CODES>)"));
        } else {
            out.paths.push(a.to_string());
        }
        i += 1;
    }
    Ok(out)
}

/// The `paths` that do not exist on disk (issue #130: a usage error, not a clean run).
pub fn missing_paths<'a>(paths: &[&'a str]) -> Vec<&'a str> {
    paths.iter().copied().filter(|p| !Path::new(p).exists()).collect()
}

/// Run `lint-docs` over `paths` (all of which must exist — the caller checks with
/// [`missing_paths`]). Exit `1` when any `W099`–`W102` finding, or any finding whose
/// code is in `deny` (the way to make the advisory `W103` gating), is present.
pub fn cmd_lint_docs(elements: &[RawElement], paths: &[&str], json: bool, deny: &BTreeSet<String>) -> i32 {
    let ctx = Ctx::new(elements);
    let mut findings: Vec<Finding> = Vec::new();
    for &path_str in paths {
        scan_path(Path::new(path_str), &ctx, &mut findings);
    }
    if findings.is_empty() {
        return 0;
    }
    if json {
        let items: Vec<serde_json::Value> = findings
            .iter()
            // Detail key matches the existing W099 shape: token / ref / path by code.
            .map(|f| serde_json::json!({ "file": f.file, "line": f.line, "code": f.code, detail_key(f.code): f.detail }))
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    } else {
        for f in &findings {
            println!("{}:{}: {}: {}", f.file, f.line, f.code, message(f.code, &f.detail));
        }
    }
    // W103 is advisory (REQ-TRS-PKG-002): only an unresolvable reference fails,
    // unless W103 is explicitly denied (issue #130).
    if findings.iter().any(|f| f.code != "W103" || deny.contains(f.code)) { 1 } else { 0 }
}

#[cfg(test)]
mod arg_tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn deny_value_is_a_code_not_a_path() {
        let a = parse_lint_args(&s(&["x.md", "--deny", "W099,W103", "--json", "--deny=W100"])).unwrap();
        assert_eq!(a.paths, vec!["x.md".to_string()]);
        assert!(a.json);
        assert_eq!(a.deny.iter().map(String::as_str).collect::<Vec<_>>(), ["W099", "W100", "W103"]);
    }

    #[test]
    fn bad_codes_and_unknown_options_are_usage_errors() {
        assert!(parse_lint_args(&s(&["x.md", "--deny", "W999"])).is_err());
        assert!(parse_lint_args(&s(&["x.md", "--deny"])).is_err());
        assert!(parse_lint_args(&s(&["x.md", "--bogus"])).is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elem(qname: &str, yaml: &str) -> RawElement {
        RawElement {
            qualified_name: qname.to_string(),
            file_path: format!("{qname}.md"),
            frontmatter: serde_yaml::from_str(yaml).unwrap(),
            doc: String::new(),
            parse_issue: None,
            derived: Default::default(),
            derive_findings: vec![],
            locale_docs: Default::default(),
            about_notes: Default::default(),
        }
    }

    fn model() -> Vec<RawElement> {
        vec![
            elem("Enum", "type: Package\nname: Enum\n"),
            elem("Enum::A", "type: Requirement\nid: REQ-LE-001\n"),
            elem("Enum::B", "type: Requirement\nid: REQ-LE-002\n"),
            elem("Enum::C", "type: Requirement\nid: REQ-LE-003\n"),
            elem("Enum::Sub", "type: Package\nname: Sub\n"),
            elem("Enum::Sub::D", "type: Requirement\nid: REQ-LE-004\n"),
            elem("Other::E", "type: Requirement\nid: REQ-LF-001\n"),
        ]
    }

    #[test]
    fn counts_distinct_direct_member_ids_in_the_body() {
        let m = model();
        let body = "---\ntype: Package\n---\n\nHolds REQ-LE-001 and REQ-LE-002,\nREQ-LE-001 again and REQ-LE-003.\n";
        let (ids, line) = enumerated_members(body, "Enum", &m);
        assert_eq!(ids.len(), 3);
        assert_eq!(line, 5, "first mention is on file line 5");
        assert!(ids.len() >= W103_THRESHOLD);
    }

    #[test]
    fn below_threshold_foreign_and_grandchild_ids_do_not_count() {
        let m = model();
        let (two, _) = enumerated_members("Only REQ-LE-001 and REQ-LE-002.\n", "Enum", &m);
        assert_eq!(two.len(), 2);
        assert!(two.len() < W103_THRESHOLD);
        let (foreign, _) = enumerated_members("REQ-LF-001, REQ-LE-004 and REQ-LE-001.\n", "Enum", &m);
        assert_eq!(foreign.len(), 1, "a foreign id and a grandchild id are not members");
    }

    #[test]
    fn frontmatter_mentions_are_ignored() {
        let m = model();
        let text = "---\ntype: Package\nnote: REQ-LE-001 REQ-LE-002 REQ-LE-003\n---\n\nPurpose only.\n";
        assert!(enumerated_members(text, "Enum", &m).0.is_empty());
        let (start, lines) = body_lines(text);
        assert_eq!(start, 4);
        assert_eq!(lines, vec!["", "Purpose only."]);
    }
}
