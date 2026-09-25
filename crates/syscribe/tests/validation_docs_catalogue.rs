//! The human-facing validation-code references must agree with the code.
//!
//! `prompts/spec/validation.md` (the one-row-per-code catalogue behind
//! `syscribe spec validation`) is kept complete by `catalogue_tests` in
//! `src/mcp/mod.rs`. This test holds the other references to the same bar:
//!
//! * `docs/validation/rules.md` lists every emitted code, and exactly the
//!   catalogue's code set;
//! * a code listed but not emitted is visibly marked retired / not
//!   implemented, and a code marked that way is really not emitted;
//! * a stated severity (a `Severity` column, or an "errors"/"warnings"
//!   section heading) never contradicts the code's `E`/`W`/`I` prefix — in
//!   `rules.md` and the format spec;
//! * spec §11.12 either tabulates an emitted code or names it inside a code
//!   family its "specified elsewhere" table delegates;
//! * the range table in `docs/validation/index.md` covers every emitted code.
//!
//! The table parser is layout-tolerant: any Markdown table row whose first
//! cell is a single code (optionally in backticks or bold) counts, whatever
//! the number or order of the other columns; escaped `\|` stays inside a cell
//! and fenced code blocks are skipped.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

// ── emitted codes (same scan as `catalogue_tests::emitted_codes`) ─────────

/// `text` with every `#[cfg(test)]`-annotated item removed (naive brace
/// matching, as in `catalogue_tests`).
fn strip_test_items(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("#[cfg(test)]") {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        let Some(open) = after.find('{') else { return out };
        let mut depth = 0usize;
        let mut end = after.len();
        for (i, ch) in after[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// Every `"E###"`/`"W###"`/`"I###"` string literal in non-test Rust source
/// under `crates/`.
fn emitted_codes() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for entry in walkdir::WalkDir::new(root().join("crates")).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("rs")
            || p.components().any(|c| matches!(c.as_os_str().to_str(), Some("tests") | Some("target")))
        {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(p) else { continue };
        let text = strip_test_items(&text);
        let b = text.as_bytes();
        for i in 0..b.len().saturating_sub(5) {
            if b[i] == b'"'
                && matches!(b[i + 1], b'E' | b'W' | b'I')
                && b[i + 2..i + 5].iter().all(u8::is_ascii_digit)
                && b[i + 5] == b'"'
            {
                out.insert(text[i + 1..i + 5].to_string());
            }
        }
    }
    out
}

// ── layout-tolerant table parsing ─────────────────────────────────────────

#[derive(Debug, Clone)]
struct Row {
    /// Line number (1-based) in the parsed text.
    line: usize,
    /// Nearest preceding Markdown heading.
    heading: String,
    /// Header cells of the table this row belongs to.
    header: Vec<String>,
    /// All cells, first one included.
    cells: Vec<String>,
}

impl Row {
    fn first(&self) -> &str {
        &self.cells[0]
    }
    fn text(&self) -> String {
        self.cells[1..].join(" | ")
    }
}

/// Split a table line into trimmed cells; `\|` does not split.
fn split_cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|');
    let inner = inner.strip_suffix('|').filter(|s| !s.ends_with('\\')).unwrap_or(inner);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut prev_backslash = false;
    for ch in inner.chars() {
        if ch == '|' && !prev_backslash {
            cells.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
        prev_backslash = ch == '\\';
    }
    cells.push(cur.trim().to_string());
    cells
}

fn is_separator(cells: &[String]) -> bool {
    cells.iter().all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

/// Every table row (header and separator rows excluded) with its context.
fn table_rows(md: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut heading = String::new();
    let mut header: Vec<String> = Vec::new();
    let mut in_table = false;
    let mut in_fence = false;
    let lines: Vec<&str> = md.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            in_table = false;
            continue;
        }
        if in_fence {
            continue;
        }
        if t.starts_with('#') {
            heading = t.trim_start_matches('#').trim().to_string();
            in_table = false;
            continue;
        }
        if !t.starts_with('|') {
            in_table = false;
            continue;
        }
        let cells = split_cells(t);
        if !in_table {
            // A table starts with a header row followed by a separator row.
            let next = lines.get(i + 1).map(|l| split_cells(l.trim_start()));
            if next.as_deref().is_some_and(is_separator) {
                header = cells;
                in_table = true;
            }
            continue;
        }
        if is_separator(&cells) {
            continue;
        }
        rows.push(Row { line: i + 1, heading: heading.clone(), header: header.clone(), cells });
    }
    rows
}

/// The code a first cell names, if it is exactly one code (`E101`, `` `E101` ``, `**E101**`).
fn single_code(cell: &str) -> Option<String> {
    let c = cell.trim_matches(|ch: char| ch == '`' || ch == '*' || ch.is_whitespace());
    let b = c.as_bytes();
    (b.len() == 4 && matches!(b[0], b'E' | b'W' | b'I') && b[1..].iter().all(u8::is_ascii_digit)).then(|| c.to_string())
}

fn code_rows(md: &str) -> Vec<(String, Row)> {
    table_rows(md).into_iter().filter_map(|r| single_code(r.first()).map(|c| (c, r))).collect()
}

fn codes_in(md: &str) -> BTreeSet<String> {
    code_rows(md).into_iter().map(|(c, _)| c).collect()
}

/// Every code named in `text`, with `A###`–`A###` ranges expanded.
fn codes_and_ranges(text: &str) -> BTreeSet<String> {
    let clean: String = text.chars().filter(|&c| c != '`' && c != '*').collect();
    let b = clean.as_bytes();
    let is_code = |i: usize| {
        i + 4 <= b.len()
            && matches!(b[i], b'E' | b'W' | b'I')
            && b[i + 1..i + 4].iter().all(u8::is_ascii_digit)
            && (i == 0 || !b[i - 1].is_ascii_alphanumeric())
            && (i + 4 == b.len() || !b[i + 4].is_ascii_alphanumeric())
    };
    let mut out = BTreeSet::new();
    let mut i = 0;
    while i < b.len() {
        if !is_code(i) {
            i += 1;
            continue;
        }
        let lo = &clean[i..i + 4];
        // A range: `<code> – <code>` / `<code>-<code>` with the same prefix letter.
        let mut j = i + 4;
        while j < b.len() && b[j] == b' ' {
            j += 1;
        }
        let dash = if clean[j..].starts_with('–') {
            '–'.len_utf8()
        } else if clean[j..].starts_with('-') {
            1
        } else {
            0
        };
        if dash > 0 {
            let mut k = j + dash;
            while k < b.len() && b[k] == b' ' {
                k += 1;
            }
            if is_code(k) && b[k] == b[i] {
                let (a, z): (u32, u32) = (lo[1..].parse().unwrap(), clean[k + 1..k + 4].parse().unwrap());
                for n in a.min(z)..=a.max(z) {
                    out.insert(format!("{}{n:03}", &lo[..1]));
                }
                i = k + 4;
                continue;
            }
        }
        out.insert(lo.to_string());
        i += 4;
    }
    out
}

/// Row is visibly marked as a code that is not (or no longer) emitted.
fn marked_inactive(row: &Row) -> bool {
    let t = row.text().to_lowercase();
    t.contains("**retired") || t.contains("*not yet implemented*") || t.contains("*not emitted*") || t.contains("never emitted")
}

fn severity_word(s: &str) -> Option<char> {
    match s.trim_matches(|c: char| c == '*' || c == '`' || c.is_whitespace()).to_lowercase().as_str() {
        "error" => Some('E'),
        "warning" => Some('W'),
        "info" | "informational" => Some('I'),
        _ => None,
    }
}

/// Severity contradictions in `md`: a severity cell that disagrees with the
/// code prefix, or a code sitting under a heading that names only the other
/// severity ("… errors" / "… warnings").
fn severity_contradictions(name: &str, md: &str) -> Vec<String> {
    let mut bad = Vec::new();
    for (code, row) in code_rows(md) {
        let prefix = code.as_bytes()[0] as char;
        for cell in &row.cells[1..] {
            if let Some(sev) = severity_word(cell) {
                if sev != prefix {
                    bad.push(format!("{name}:{} {code} has severity cell {cell:?}", row.line));
                }
            }
        }
        let h = row.heading.to_lowercase();
        let (errs, warns) = (h.contains("error"), h.contains("warning"));
        if errs && !warns && prefix != 'E' {
            bad.push(format!("{name}:{} {code} is listed under error heading {:?}", row.line, row.heading));
        }
        if warns && !errs && prefix == 'E' {
            bad.push(format!("{name}:{} {code} is listed under warning heading {:?}", row.line, row.heading));
        }
    }
    bad
}

/// The text of spec §11.12 (up to the next top-level section).
fn spec_11_12(spec: &str) -> &str {
    let start = spec.find("\n### 11.12 ").expect("spec has §11.12");
    let end = spec[start..].find("\n## 12 ").map_or(spec.len(), |e| start + e);
    &spec[start..end]
}

// ── tests ─────────────────────────────────────────────────────────────────

#[test]
fn parser_is_layout_tolerant() {
    let md = "## Some errors (E001)\n\n\
              | Code | Severity | Condition |\n|---|---|---|\n\
              | `E001` | Error | a \\| b |\n\
              | **W002** | Warning | x |\n\n\
              ```\n| E003 | fenced, ignored |\n```\n\n\
              | Code | Element | Condition |\n|:--|:--|--|\n| I004 | Any | y |\n\
              | not a code | z | w |\n";
    let rows = code_rows(md);
    let codes: Vec<&str> = rows.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(codes, ["E001", "W002", "I004"]);
    assert_eq!(rows[0].1.cells[2], "a \\| b", "escaped pipe stays in the cell");
    assert_eq!(rows[0].1.header, ["Code", "Severity", "Condition"]);
    // W002 under an "errors" heading is flagged; its Warning cell is fine.
    let bad = severity_contradictions("t", md);
    assert!(bad.len() == 2 && bad.iter().all(|b| b.contains("W002") || b.contains("I004")), "{bad:#?}");
    assert_eq!(
        codes_and_ranges("`E500`–`E503`, W930 and E016-E018; E1234 no"),
        ["E016", "E017", "E018", "E500", "E501", "E502", "E503", "W930"].map(String::from).into()
    );
}

#[test]
fn rules_md_lists_every_emitted_code() {
    let listed = codes_in(&read("docs/validation/rules.md"));
    let missing: Vec<_> = emitted_codes().difference(&listed).cloned().collect();
    assert!(missing.is_empty(), "docs/validation/rules.md has no row for emitted code(s): {missing:?}");
}

#[test]
fn rules_md_and_catalogue_list_the_same_codes() {
    let rules = codes_in(&read("docs/validation/rules.md"));
    let catalogue = codes_in(&read("prompts/spec/validation.md"));
    let only_rules: Vec<_> = rules.difference(&catalogue).collect();
    let only_catalogue: Vec<_> = catalogue.difference(&rules).collect();
    assert!(
        only_rules.is_empty() && only_catalogue.is_empty(),
        "code sets differ — only in rules.md: {only_rules:?}; only in prompts/spec/validation.md: {only_catalogue:?}"
    );
}

#[test]
fn listed_but_unemitted_codes_are_marked_and_marked_codes_are_unemitted() {
    let emitted = emitted_codes();
    let spec = read("spec/markdown-sysml-format.md");
    let mut bad = Vec::new();
    for (name, md) in [
        ("docs/validation/rules.md", read("docs/validation/rules.md")),
        ("prompts/spec/validation.md", read("prompts/spec/validation.md")),
        ("spec/markdown-sysml-format.md", spec),
    ] {
        for (code, row) in code_rows(&md) {
            match (emitted.contains(&code), marked_inactive(&row)) {
                (false, false) => bad.push(format!("{name}:{} {code} is not emitted but not marked retired/not implemented", row.line)),
                (true, true) => bad.push(format!("{name}:{} {code} is marked retired/not implemented but is emitted", row.line)),
                _ => {}
            }
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn stated_severities_match_code_prefixes() {
    let mut bad = Vec::new();
    // The grouped references and the catalogue: a row under an "errors"/"warnings"
    // heading (or with a Severity column) must match its code prefix. The
    // catalogue's completeness is `catalogue_tests`' job.
    for rel in ["docs/validation/rules.md", "spec/markdown-sysml-format.md", "prompts/spec/validation.md"] {
        bad.extend(severity_contradictions(rel, &read(rel)));
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn spec_11_12_tabulates_or_delegates_every_emitted_code() {
    let spec = read("spec/markdown-sysml-format.md");
    let section = spec_11_12(&spec);
    assert!(
        section.contains("syscribe spec validation") && section.contains("Code families specified elsewhere"),
        "§11.12 must point at the catalogue and carry the delegation table"
    );
    let mut covered = BTreeSet::new();
    for row in table_rows(section) {
        match single_code(row.first()) {
            Some(code) => {
                covered.insert(code);
            }
            None => covered.extend(codes_and_ranges(row.first())),
        }
    }
    let missing: Vec<_> = emitted_codes().difference(&covered).cloned().collect();
    assert!(
        missing.is_empty(),
        "spec §11.12 neither tabulates nor delegates emitted code(s) {missing:?} — add a row, or add the family to its \
         'Code families specified elsewhere' table"
    );
}

#[test]
fn index_md_range_table_covers_every_emitted_code() {
    let index = read("docs/validation/index.md");
    let covered: BTreeSet<String> = table_rows(&index)
        .into_iter()
        .filter(|r| r.header.first().is_some_and(|h| h == "Range"))
        .flat_map(|r| codes_and_ranges(r.first()))
        .collect();
    let missing: Vec<_> = emitted_codes().difference(&covered).cloned().collect();
    assert!(missing.is_empty(), "docs/validation/index.md 'Rule groups' ranges miss emitted code(s): {missing:?}");
}
