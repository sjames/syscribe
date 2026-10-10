//! `verification-depth` — per-requirement verification-level depth & independence
//! report with a `--min-levels` gate (REQ-TRS-OUT-011, GH #16).

use std::collections::BTreeSet;

use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::ValidationResult;

/// Render the report. Returns `true` when no `--min-levels` gate is violated.
pub fn cmd_verification_depth(
    elements: &[RawElement],
    resolver: &Resolver,
    val: &ValidationResult,
    sil: Option<&str>,
    status: Option<&str>,
    min_levels: Option<usize>,
    json: bool,
    results: Option<&syscribe_model::results::ResultsData>,
) -> bool {
    struct Row<'a> {
        id: &'a str,
        sil: Option<u8>,
        asil: Option<&'a str>,
        levels: Vec<String>,
        failing: Vec<String>,
        not_run: Vec<String>,
        flag: &'static str,
    }

    let mut rows: Vec<Row> = Vec::new();
    for e in elements {
        if !Resolver::is_native_requirement(e) {
            continue;
        }
        let fm = &e.frontmatter;
        if let Some(s) = status {
            if fm.status.as_deref() != Some(s) {
                continue;
            }
        }
        if let Some(v) = sil {
            let m = fm.sil_level.is_some_and(|n| n.to_string() == v)
                || fm.asil_level.as_deref() == Some(v);
            if !m {
                continue;
            }
        }
        let req_id = fm.id.as_deref().unwrap_or(&e.qualified_name);
        // Distinct testLevels among the *active* TestCases that verify this req.
        // With results ingested, a failing test, or an automated test whose functions did
        // not run, contributes no level and is listed separately (GH #257 b, c).
        let mut levels: BTreeSet<String> = BTreeSet::new();
        let mut failing: BTreeSet<String> = BTreeSet::new();
        let mut not_run: BTreeSet<String> = BTreeSet::new();
        if let Some(tcs) = val.verified_by.get(req_id) {
            for tc_id in tcs {
                if let Some(tc) = resolver.get_by_id(elements, tc_id) {
                    if tc.frontmatter.status.as_deref() == Some("active") {
                        if results.is_some() {
                            use syscribe_model::safety_case::Verdict;
                            match syscribe_model::results::testcase_verdict(tc, results) {
                                Verdict::Fail => {
                                    failing.insert(tc_id.clone());
                                    continue;
                                }
                                Verdict::Unknown if !syscribe_model::results::tc_function_refs(tc).is_empty() => {
                                    not_run.insert(tc_id.clone());
                                    continue;
                                }
                                _ => {}
                            }
                        }
                        if let Some(lvl) = tc.frontmatter.test_level.as_deref() {
                            levels.insert(lvl.to_string());
                        }
                    }
                }
            }
        }
        let levels: Vec<String> = levels.into_iter().collect();
        let flag = if levels.is_empty() {
            "none"
        } else if levels.len() == 1 && levels[0] == "L5" {
            "hil-only"
        } else if levels.len() == 1 {
            "single"
        } else {
            "ok"
        };
        rows.push(Row {
            id: req_id,
            sil: fm.sil_level,
            asil: fm.asil_level.as_deref(),
            levels,
            failing: failing.into_iter().collect(),
            not_run: not_run.into_iter().collect(),
            flag,
        });
    }
    rows.sort_by(|a, b| a.id.cmp(b.id));

    let ok = match min_levels {
        Some(n) => rows.iter().all(|r| r.levels.len() >= n),
        None => true,
    };

    if json {
        let items: Vec<_> = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "silLevel": r.sil,
                    "asilLevel": r.asil,
                    "levels": r.levels,
                    "count": r.levels.len(),
                    "failing": r.failing,
                    "notRun": r.not_run,
                    "flag": r.flag,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
        return ok;
    }

    println!("# Verification depth ({} requirements)", rows.len());
    println!();
    println!(
        "Counts `active` TestCases only (draft, approved and retired ones do not count){}.",
        if results.is_some() { "; with ingested results a failing test, and an automated test whose functions did not run, is excluded too" } else { "" }
    );
    println!();
    println!("| Requirement | SIL/ASIL | Levels | Count | Flag | Failing | Not run |");
    println!("|---|---|---|---|---|---|---|");
    for r in &rows {
        let integ = r
            .sil
            .map(|n| n.to_string())
            .or_else(|| r.asil.map(|s| s.to_string()))
            .unwrap_or_else(|| "—".into());
        let lv = if r.levels.is_empty() {
            "—".to_string()
        } else {
            r.levels.join(",")
        };
        let dash = |v: &Vec<String>| if v.is_empty() { "—".to_string() } else { v.join(",") };
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            r.id,
            integ,
            lv,
            r.levels.len(),
            r.flag,
            dash(&r.failing),
            dash(&r.not_run)
        );
    }
    if let Some(n) = min_levels {
        println!();
        if ok {
            println!("All {} requirements meet --min-levels {}.", rows.len(), n);
        } else {
            let bad = rows.iter().filter(|r| r.levels.len() < n).count();
            println!(
                "FAIL: {} requirement(s) have fewer than {} distinct verification levels.",
                bad, n
            );
        }
    }
    ok
}
