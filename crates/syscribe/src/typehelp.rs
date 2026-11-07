//! Type-centric help and keyword search (GH #438): `help <Type>`, `help --search <kw>`,
//! `help tasks`. Everything here is derived from data the tool already ships — the man
//! pages, the spec sections, the validation catalogue, the MCP type schema and the
//! `template` skeletons — so it cannot drift from them.

use crate::help;
use std::fmt::Write as _;

/// Commands that act on any element, whatever its type.
const UNIVERSAL: &[&str] = &["show", "list", "find", "ls", "links", "trace", "impact", "set", "move", "validate"];

/// The canonical spelling of an element-type name, matched case-insensitively.
pub fn resolve_type(name: &str) -> Option<&'static str> {
    crate::mcp::known_type_names().into_iter().find(|t| t.eq_ignore_ascii_case(name))
}

fn mentions(text: &str, word: &str) -> bool {
    let b = text.as_bytes();
    let mut from = 0;
    while let Some(i) = text[from..].find(word) {
        let s = from + i;
        let e = s + word.len();
        let ok_before = s == 0 || !(b[s - 1].is_ascii_alphanumeric() || b[s - 1] == b'_');
        let ok_after = e >= b.len() || !(b[e].is_ascii_alphanumeric() || b[e] == b'_');
        if ok_before && ok_after {
            return true;
        }
        from = e;
    }
    false
}

/// The commands that read or mutate elements of type `ty`: the universal ones, plus every
/// command whose man page names the type. `(command, one-line summary, specific?)`.
pub fn commands_for(ty: &str) -> Vec<(&'static str, &'static str, bool)> {
    let mut out: Vec<(&'static str, &'static str, bool)> = Vec::new();
    for (name, body) in help::entries() {
        if name == "help" {
            continue;
        }
        let universal = UNIVERSAL.contains(&name);
        // The SEE ALSO block only lists neighbours; it must not make a page "about" the type.
        let text = body.split("\n## SEE ALSO").next().unwrap_or(body);
        let specific = mentions(text, ty);
        if universal || specific {
            out.push((name, help::summary(body), specific));
        }
    }
    // Specific ones first, then the universal ones, each in registry order.
    out.sort_by_key(|(_, _, specific)| !*specific);
    out
}

/// Validation-catalogue rows that name `ty`: `(code, first sentence)`.
pub fn codes_for(ty: &str) -> Vec<(String, String)> {
    let Some((_, text)) = crate::spec::SECTIONS.iter().find(|(n, _)| *n == "validation") else { return Vec::new() };
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        let Some(code) = cells.first().map(|c| c.trim_matches('`')) else { continue };
        let is_code = code.len() >= 4
            && matches!(code.as_bytes()[0], b'E' | b'W' | b'I')
            && code[1..].bytes().all(|b| b.is_ascii_digit());
        if is_code && mentions(line, ty) {
            let desc = cells.last().copied().unwrap_or("");
            let first = desc.split(". ").next().unwrap_or(desc);
            out.push((code.to_string(), truncate(first, 110)));
        }
    }
    out
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n - 1).collect::<String>().trim_end())
    }
}

/// The `help <Type>` page.
pub fn type_page(ty: &str) -> String {
    let mut o = String::new();
    let _ = writeln!(o, "# {ty} — element type\n");
    let _ = writeln!(o, "## FIELDS");
    let _ = writeln!(o, "Core fields (`syscribe spec fields` has the full reference):");
    for f in crate::mcp::type_field_specs(ty) {
        let name = f["name"].as_str().unwrap_or("");
        let req = if f["required"].as_bool().unwrap_or(false) { "required" } else { "optional" };
        let kind = f["type"].as_str().unwrap_or("");
        let domain = f["enum"]
            .as_array()
            .map(|a| format!(": {}", a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" | ")))
            .unwrap_or_default();
        let _ = writeln!(o, "    {name:<16} {req:<9} {kind}{domain}");
    }
    let _ = writeln!(o);
    if let Some(t) = crate::query::template_str(ty) {
        let _ = writeln!(o, "## EXAMPLE");
        let _ = writeln!(o, "A skeleton (`syscribe template {ty}` prints it; no model needed):\n");
        for line in t.trim_end().lines() {
            let _ = writeln!(o, "    {line}");
        }
        let _ = writeln!(o);
    }
    let value_notes = value_ref_notes(ty);
    if type_notes(ty).is_some() || value_notes.is_some() {
        let _ = writeln!(o, "## NOTES");
        let notes: Vec<&str> = [type_notes(ty), value_notes].into_iter().flatten().map(str::trim_end).collect();
        let _ = writeln!(o, "{}", notes.join("\n\n"));
        let _ = writeln!(o);
    }
    let codes = codes_for(ty);
    if !codes.is_empty() {
        let _ = writeln!(o, "## VALIDATION CODES");
        let _ = writeln!(o, "Catalogue rows that name {ty} (`syscribe explain <code>` for any code):");
        for (c, d) in &codes {
            let _ = writeln!(o, "    {c:<6} {d}");
        }
        let _ = writeln!(o);
    }
    let _ = writeln!(o, "## COMMANDS");
    let cmds = commands_for(ty);
    let _ = writeln!(o, "Commands that read or change a {ty}:");
    for (name, summary, specific) in &cmds {
        let mark = if *specific { "" } else { "  (any element)" };
        let _ = writeln!(o, "    {name:<18} {summary}{mark}");
    }
    let _ = writeln!(o, "    {:<18} list only {ty} elements", format!("list {ty}"));
    let _ = writeln!(o, "    {:<18} print the skeleton above", format!("template {ty}"));
    let _ = writeln!(o, "    {:<18} findings for one element's file", "validate --file");
    let _ = writeln!(o, "\nDetails: `syscribe help <command>`.");
    o
}

/// Type-specific notes that no field schema conveys: body conventions and how to cite parts
/// of an element (GH #453).
fn type_notes(ty: &str) -> Option<&'static str> {
    match ty {
        "Requirement" => Some(
            "Item anchors: each top-level numbered item of the body is citable. Write `{#slug}` after the\n\
             number to anchor it (`1. {#grant-keys} **Server keys.** ...`; slug `[a-z][a-z0-9-]*`); show\n\
             and export-html hide the token. Cite an item from any element body as `REQ-X#grant-keys`\n\
             (preferred), `REQ-X#5` or `REQ-X item 5` (by position — shifts when an item is inserted).\n\
             validate: W092 dangling item reference or duplicate/malformed anchor. Resolve with\n\
             `check-ref REQ-X#grant-keys`; list citers with `refs REQ-X#grant-keys`; `impact REQ-X#grant-keys`;\n\
             baseline one item with `suspect accept <src> REQ-X#grant-keys` (anchors only).\n\n\
             Amendments: once approved (or implemented/verified), record each change to the text in\n\
             `amendments:` instead of editing silently — `syscribe set REQ-X amendment.add\n\
             kind=clarification|scope-change reason=\"…\" [ref=PI-X#Q1|RR-id|<commit>] --by <you>` (date = today).\n\
             E330 malformed entry or unresolved ref; W325 a scope-change without an owner reference\n\
             (answered question, ReviewRecord, or a person's commit); W324 (opt-in, [traceability]\n\
             require_amendments = true) the normative content changed since approval with no amendment\n\
             dated on or after the change. An amendment alone never makes links suspect.",
        ),
        "ADR" => Some(
            "Amendments: an accepted decision is amended, never silently edited. When implementation shows\n\
             it must change, edit the text and record why in `amendments:` — `syscribe set ADR-X amendment.add\n\
             kind=clarification|scope-change reason=\"…\" [ref=PI-X#Q1|RR-id|<commit>] --by <you>` fills\n\
             `date` (today) and `by`. `kind: clarification` keeps the meaning; `kind: scope-change` changes it\n\
             and needs `ref:` to the owner's decision — an answered question, a ReviewRecord, or a commit a\n\
             person made (else W325). E330: malformed entry or a ref that resolves to nothing. W324 (opt-in,\n\
             [traceability] require_amendments = true): the body or normative fields changed since the commit\n\
             that set `accepted`, with no amendment dated on or after the change. show and export-html print\n\
             the history; to replace a decision instead, write a new ADR with `supersedes:`.",
        ),
        "PlanningItem" => Some(
            "Questions are citable from any element body as `PI-X#Q3`. validate: W092 the question does not\n\
             exist; W093 an element past draft cites a question that is still open; W094 an answered question\n\
             is cited with \"to confirm\"-style wording ([references] open_markers). `check-ref PI-X#Q3` prints\n\
             its status and answer; `refs PI-X#Q3` lists who cites it.",
        ),
        _ => None,
    }
}

/// Value references (GH #452): how a body states a design value without copying it — on the
/// types whose bodies commonly state values, and on the types that hold them.
fn value_ref_notes(ty: &str) -> Option<&'static str> {
    match ty {
        "Requirement" | "TestCase" | "PlanningItem" | "ADR" => Some(
            "Value references: state a design value as `{{Elem.attr}}` instead of copying the number —\n\
             `Elem` is a stable id, qualified name or unique basic name, `attr` an attribute of it (a `features:` entry with a\n\
             value, an Attribute child, a tagged value `Stereotype.key`, or a budget field such as `ram`;\n\
             dotted for nested features). It resolves in the body and in ```gherkin scenarios, not in other\n\
             code; show and export-html render the value with its unit (`47244 B`). validate: E331 an\n\
             unresolvable reference (lists the attributes); W096 a literal equal to a previous value of an\n\
             attribute of a linked element (a stale copy, from git history); W095 (opt-in, [references]\n\
             copied_values = true) a literal equal to its current value. `refs Elem.attr` lists the uses,\n\
             literal copies and stale copies.",
        ),
        "PartDef" | "Part" | "AttributeDef" | "Attribute" | "InterfaceDef" | "ItemDef" | "PortDef" => Some(
            "Value references: an attribute with a single value — a `features:` entry's `value:` (else\n\
             `default:`) and `unit:`, an Attribute child's `value:`, a tagged value, or a budget field\n\
             (`ram`, `flash`, `period`, `wcet`, …) — is cited from any body as `{{Elem.attr}}` (dotted for\n\
             nested features, `{{Elem.Stereotype.key}}` for a tagged value) and rendered with its unit by\n\
             show and export-html. `refs Elem.attr` lists every use, literal copy and stale copy.",
        ),
        _ => None,
    }
}

/// The `help tasks` page: intents mapped to commands.
pub const TASKS: &str = include_str!("../../../prompts/help/tasks.md");

/// One search result.
pub struct Hit {
    pub kind: &'static str,
    pub name: String,
    pub line: String,
    pub score: u32,
}

/// Ranked matches of `keyword` over command names, synopses, page text, element-type
/// names and spec sections. Case-insensitive; every word of a multi-word keyword must occur.
pub fn search(keyword: &str) -> Vec<Hit> {
    let words: Vec<String> = keyword.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return Vec::new();
    }
    let all = |s: &str| {
        let l = s.to_lowercase();
        words.iter().all(|w| l.contains(w))
    };
    let mut hits: Vec<Hit> = Vec::new();
    for (name, body) in help::entries() {
        let mut score = 0;
        if name.eq_ignore_ascii_case(keyword) {
            score += 100;
        } else if all(name) {
            score += 60;
        }
        let summary = help::summary(body);
        if all(summary) {
            score += 40;
        }
        // The best line: a synopsis/option line beats prose; take the first match of the best kind.
        let mut best: Option<(u32, &str)> = None;
        let mut section = "";
        for line in body.lines() {
            if let Some(h) = line.strip_prefix("## ") {
                section = h.trim();
                continue;
            }
            if line.trim().is_empty() || !all(line) {
                continue;
            }
            let w = match section {
                "SYNOPSIS" => 30,
                "OPTIONS" => 25,
                "SEE ALSO" => 2,
                _ => 10,
            };
            if best.is_none_or(|(b, _)| w > b) {
                best = Some((w, line.trim()));
            }
        }
        if let Some((w, _)) = best {
            score += w;
        }
        if score > 0 {
            let line = best.map(|(_, l)| l.to_string()).unwrap_or_else(|| summary.to_string());
            hits.push(Hit { kind: "command", name: name.to_string(), line, score });
        }
    }
    for t in crate::mcp::known_type_names() {
        if all(t) {
            let score = if t.eq_ignore_ascii_case(keyword) { 90 } else { 50 };
            hits.push(Hit { kind: "type", name: t.to_string(), line: format!("element type — `syscribe help {t}`"), score });
        }
    }
    for (sec, text) in crate::spec::SECTIONS {
        if let Some(line) = text.lines().find(|l| !l.trim().is_empty() && all(l)) {
            hits.push(Hit { kind: "spec", name: format!("spec {sec}"), line: truncate(line.trim(), 110), score: 5 });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
    hits
}

/// Render [`search`] results.
pub fn search_page(keyword: &str) -> String {
    let hits = search(keyword);
    let mut o = String::new();
    if hits.is_empty() {
        let _ = writeln!(o, "No help matches '{keyword}'. Try `syscribe help` for the command index or `syscribe help tasks`.");
        return o;
    }
    let _ = writeln!(o, "Matches for '{keyword}' ({}):\n", hits.len());
    for h in hits.iter().take(25) {
        let _ = writeln!(o, "  {:<22} [{}]", h.name, h.kind);
        let _ = writeln!(o, "      {}", truncate(&h.line, 100));
    }
    if hits.len() > 25 {
        let _ = writeln!(o, "\n  … {} more; narrow the keyword.", hits.len() - 25);
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_names_resolve_case_insensitively() {
        assert_eq!(resolve_type("planningitem"), Some("PlanningItem"));
        assert_eq!(resolve_type("nope"), None);
    }

    #[test]
    fn a_planning_item_page_lists_claim_and_release() {
        let p = type_page("PlanningItem");
        assert!(p.contains("    claim "), "{p}");
        assert!(p.contains("    release "), "{p}");
        assert!(p.contains("todo | in_progress | blocked | done"), "{p}");
        assert!(p.contains("## EXAMPLE") && p.contains("## VALIDATION CODES"), "{p}");
        assert!(p.contains("E708"), "{p}");
    }

    #[test]
    fn searching_evidence_finds_set_ingest_results_and_results() {
        let names: Vec<String> = search("evidence").into_iter().map(|h| h.name).collect();
        for want in ["set", "ingest-results", "results"] {
            assert!(names.iter().any(|n| n == want), "{want} missing from {names:?}");
        }
    }

    #[test]
    fn questions_reserved_to_a_person_are_discoverable() {
        // GH #457: `answerBy`, `human_only_answers` and W323 reach the type page, the template and search.
        let p = type_page("PlanningItem");
        assert!(p.contains("answerBy: human") && p.contains("W323"), "{p}");
        for kw in ["answerBy", "human_only_answers", "owner", "answer human"] {
            let names: Vec<String> = search(kw).into_iter().map(|h| h.name).collect();
            assert!(names.iter().any(|n| n == "questions" || n == "policy"), "{kw}: {names:?}");
        }
    }

    #[test]
    fn value_references_are_discoverable() {
        // GH #452: `{{Elem.attr}}`, `refs Elem.attr`, W095/W096 reach search and the type pages.
        for kw in ["value reference", "attribute", "copied value", "stale"] {
            let names: Vec<String> = search(kw).into_iter().map(|h| h.name).collect();
            assert!(names.iter().any(|n| n == "refs"), "{kw}: {names:?}");
        }
        for t in ["TestCase", "Requirement", "PartDef"] {
            let p = type_page(t);
            assert!(p.contains("{{Elem.attr}}") && p.contains("refs Elem.attr"), "{t}: {p}");
        }
    }

    #[test]
    fn an_exact_command_name_ranks_first() {
        assert_eq!(search("claim")[0].name, "claim");
    }

    #[test]
    fn every_type_page_renders() {
        for t in crate::mcp::known_type_names() {
            assert!(type_page(t).contains("## COMMANDS"), "{t}");
        }
    }
}
