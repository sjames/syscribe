//! `syscribe claim <PI-id> --by <agent-id>` / `syscribe release <PI-id>` —
//! advisory claim/ownership markers on `PlanningItem` for concurrent
//! multi-agent work (issue #115).
//!
//! Not a filesystem lock: the value is a coordination signal ("is anyone
//! already on this?") for an orchestrating process running multiple agents
//! against one model, so a second `claim` on an already-claimed item is
//! refused with a clear message rather than silently overwriting it.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::frontmatter::{splice_frontmatter, split_frontmatter, yaml_scalar};
use syscribe_model::mutate::file_unified_diff;
use syscribe_model::resolver::Resolver;

/// (year, month, day) from days since the Unix epoch (Howard Hinnant's civil
/// algorithm) -- same small no-dependency timestamp primitive `sbom`/`reqif`
/// already carry their own copy of.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn iso8601_now() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, rem / 3600, (rem % 3600) / 60, rem % 60)
}

/// Replace (in place) or append each `(key, Some(value))` pair as a
/// `key: value` line; drop the line entirely for `(key, None)`. Every other
/// line is preserved byte-for-byte -- the same splice primitive `set
/// status=`/`applies-when` use, generalized to more than one key in one pass
/// (`claim` writes two fields, `release` clears two).
fn splice_scalar_fields(content: &str, ops: &[(&str, Option<String>)]) -> Option<String> {
    let (yaml_opt, _) = split_frontmatter(content);
    let yaml = yaml_opt?;
    let mut lines: Vec<String> = Vec::new();
    let mut found = vec![false; ops.len()];
    for line in yaml.lines() {
        let matched = ops.iter().position(|(key, _)| line.starts_with(&format!("{key}:")));
        match matched {
            Some(idx) => {
                found[idx] = true;
                if let Some(v) = &ops[idx].1 {
                    lines.push(format!("{}: {}", ops[idx].0, yaml_scalar(v)));
                }
            }
            None => lines.push(line.to_string()),
        }
    }
    for (idx, (key, value)) in ops.iter().enumerate() {
        if !found[idx] {
            if let Some(v) = value {
                lines.push(format!("{key}: {}", yaml_scalar(v)));
            }
        }
    }
    let new_fm = lines.join("\n");
    Some(splice_frontmatter(content, yaml, &new_fm))
}

fn preview_or_write(model_root: &Path, elem: &RawElement, new_content: &str, dry_run: bool) -> bool {
    let rel = elem
        .file_path
        .strip_prefix(&*model_root.to_string_lossy())
        .map(|s| s.trim_start_matches(['/', '\\']))
        .unwrap_or(&elem.file_path);
    let old = std::fs::read_to_string(&elem.file_path).ok();
    let diff = file_unified_diff(rel, old.as_deref(), Some(new_content));
    if diff.is_empty() {
        println!("{}: no change.", elem.qualified_name);
        return false;
    }
    if dry_run {
        print!("{diff}");
        return false;
    }
    if let Err(e) = std::fs::write(&elem.file_path, new_content) {
        eprintln!("Write failed for {}: {e}", elem.file_path);
        std::process::exit(1);
    }
    print!("{diff}");
    true
}

fn require_planning_item<'a>(elements: &'a [RawElement], resolver: &Resolver, key: &str) -> &'a RawElement {
    let elem = match resolver.resolve_ref(elements, key) {
        Some(e) => e,
        None => {
            eprintln!("Element not found: {key}");
            std::process::exit(1);
        }
    };
    if !matches!(elem.frontmatter.element_type, Some(ElementType::PlanningItem)) {
        eprintln!("{key} is not a PlanningItem — claim/release only apply to PlanningItem.");
        std::process::exit(1);
    }
    elem
}

/// `syscribe claim <PI-id> --by <agent-id> [--dry-run]`
pub fn cmd_claim(
    model_root: &Path,
    elements: &[RawElement],
    resolver: &Resolver,
    target_key: &str,
    by: &str,
    dry_run: bool,
) {
    let elem = require_planning_item(elements, resolver, target_key);

    if elem.frontmatter.status.as_deref() == Some("done") {
        eprintln!("{} is already 'done' — nothing to claim.", elem.qualified_name);
        std::process::exit(1);
    }

    if let Some(existing) = elem.frontmatter.claimed_by.as_deref() {
        if existing != by {
            eprintln!(
                "Refusing to claim {} — already claimed by '{}' (at {}).",
                elem.qualified_name,
                existing,
                elem.frontmatter.claimed_at.as_deref().unwrap_or("unknown time")
            );
            std::process::exit(1);
        }
        // Same claimant re-claiming: allowed, refreshes claimedAt.
    }

    let now = iso8601_now();
    let content = std::fs::read_to_string(&elem.file_path).unwrap_or_default();
    let new_content = match splice_scalar_fields(
        &content,
        &[("claimedBy", Some(by.to_string())), ("claimedAt", Some(now.clone()))],
    ) {
        Some(c) => c,
        None => {
            eprintln!("{} has no YAML frontmatter to edit.", elem.file_path);
            std::process::exit(1);
        }
    };
    let committed = preview_or_write(model_root, elem, &new_content, dry_run);
    if committed {
        println!("Claimed {} by '{}' at {}", elem.qualified_name, by, now);
    }
}

/// `syscribe release <PI-id> [--dry-run]`
pub fn cmd_release(model_root: &Path, elements: &[RawElement], resolver: &Resolver, target_key: &str, dry_run: bool) {
    let elem = require_planning_item(elements, resolver, target_key);

    if elem.frontmatter.claimed_by.is_none() && elem.frontmatter.claimed_at.is_none() {
        println!("{} is not claimed — nothing to release.", elem.qualified_name);
        return;
    }

    let content = std::fs::read_to_string(&elem.file_path).unwrap_or_default();
    let new_content = match splice_scalar_fields(&content, &[("claimedBy", None), ("claimedAt", None)]) {
        Some(c) => c,
        None => {
            eprintln!("{} has no YAML frontmatter to edit.", elem.file_path);
            std::process::exit(1);
        }
    };
    let committed = preview_or_write(model_root, elem, &new_content, dry_run);
    if committed {
        println!("Released {}", elem.qualified_name);
    }
}
