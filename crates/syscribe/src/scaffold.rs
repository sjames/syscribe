//! `syscribe scaffold-gherkin <TC>` — generate/align Gherkin `Scenario:` blocks
//! from a TestCase's `testFunctions[].scenario` titles (issue #5).
//!
//! `E106` requires every `testFunctions[].scenario` to match a `Scenario:` (or
//! `Scenario Outline:`) title in the same file. Maintaining that 1:1 mapping by
//! hand is tedious; this command prints stub scenarios (default) or, with
//! `--fix`, inserts the missing ones into the file's first `gherkin` block.

use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;

/// Pull the `scenario:` titles out of a TestCase's `testFunctions` list, in order.
fn scenario_titles(elem: &RawElement) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(fns) = &elem.frontmatter.test_functions {
        let key = serde_yaml::Value::String("scenario".into());
        for tf in fns {
            if let serde_yaml::Value::Mapping(map) = tf {
                if let Some(serde_yaml::Value::String(s)) = map.get(&key) {
                    out.push(s.clone());
                }
            }
        }
    }
    out
}

/// Existing `Scenario:` / `Scenario Outline:` titles found in the doc body.
fn existing_scenarios(doc: &str) -> Vec<String> {
    doc.lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("Scenario Outline:")
                .or_else(|| t.strip_prefix("Scenario:"))
                .map(|s| s.trim().to_string())
        })
        .collect()
}

/// Render one stub scenario block (indented two spaces, as inside a Gherkin block).
fn stub(title: &str) -> String {
    format!(
        "  Scenario: {title}\n    Given <precondition>\n    When <action>\n    Then <expected outcome>\n",
        title = title
    )
}

pub fn cmd_scaffold_gherkin(elements: &[RawElement], resolver: &Resolver, key: &str, fix: bool) {
    let elem = match resolver.resolve_ref(elements, key).or_else(|| {
        elements.iter().find(|e| e.qualified_name == key)
    }) {
        Some(e) => e,
        None => {
            eprintln!("TestCase not found: {key}");
            std::process::exit(1);
        }
    };

    let titles = scenario_titles(elem);
    if titles.is_empty() {
        eprintln!(
            "No testFunctions[].scenario entries on {} — nothing to scaffold.",
            elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name)
        );
        std::process::exit(1);
    }

    let existing = existing_scenarios(&elem.doc);
    let missing: Vec<&String> = titles.iter().filter(|t| !existing.contains(t)).collect();

    if !fix {
        // Print a suggested block covering every testFunctions scenario.
        let feature = elem
            .frontmatter
            .title
            .as_deref()
            .or(elem.frontmatter.name.as_deref())
            .unwrap_or("Test scenarios");
        println!("```gherkin");
        println!("Feature: {feature}");
        println!();
        for t in &titles {
            let marker = if existing.contains(t) { " (exists)" } else { " (missing)" };
            print!("{}", stub(t));
            // annotate as a trailing comment line for the human reader
            println!("  # ^{}{}", t, marker);
            println!();
        }
        println!("```");
        if missing.is_empty() {
            eprintln!("All {} scenario(s) already present — re-run with --fix is a no-op.", titles.len());
        } else {
            eprintln!(
                "{} of {} scenario(s) missing; run `syscribe scaffold-gherkin {} --fix` to insert them.",
                missing.len(),
                titles.len(),
                elem.frontmatter.id.as_deref().unwrap_or(&elem.qualified_name)
            );
        }
        return;
    }

    // --fix: insert missing scenarios into the file.
    if missing.is_empty() {
        println!("Nothing to fix — all {} scenario(s) already present.", titles.len());
        return;
    }

    let path = &elem.file_path;
    let original = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Cannot read {path}: {e}");
            std::process::exit(1);
        }
    };

    let updated = insert_missing(&original, &missing, elem);
    if let Err(e) = std::fs::write(path, &updated) {
        eprintln!("Cannot write {path}: {e}");
        std::process::exit(1);
    }
    println!(
        "Inserted {} scenario(s) into {}:",
        missing.len(),
        path
    );
    for t in &missing {
        println!("  + Scenario: {t}");
    }
}

/// Insert the missing scenario stubs into the first `gherkin` fenced block,
/// creating one (with a `Feature:` line) when none exists.
fn insert_missing(original: &str, missing: &[&String], elem: &RawElement) -> String {
    let lines: Vec<&str> = original.lines().collect();

    // Locate the first ```gherkin fence and its closing ```.
    let mut open_idx = None;
    for (i, l) in lines.iter().enumerate() {
        if l.trim_start().starts_with("```gherkin") {
            open_idx = Some(i);
            break;
        }
    }

    let block: String = missing.iter().map(|t| stub(t)).collect();

    if let Some(open) = open_idx {
        // Find the closing fence after the opener.
        let mut close = lines.len();
        for (i, l) in lines.iter().enumerate().skip(open + 1) {
            if l.trim() == "```" {
                close = i;
                break;
            }
        }
        let mut out = String::new();
        for (i, l) in lines.iter().enumerate() {
            if i == close {
                // Ensure a blank line separating prior content from new stubs.
                if !out.ends_with("\n\n") && !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&block);
            }
            out.push_str(l);
            out.push('\n');
        }
        if close == lines.len() {
            // No closing fence was found; append stubs then close the block.
            out.push_str(&block);
            out.push_str("```\n");
        }
        out
    } else {
        // No gherkin block at all — append a fresh one.
        let feature = elem
            .frontmatter
            .title
            .as_deref()
            .or(elem.frontmatter.name.as_deref())
            .unwrap_or("Test scenarios");
        let mut out = original.to_string();
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("\n```gherkin\nFeature: {feature}\n\n{block}```\n"));
        out
    }
}
