//! `syscribe ingest-results` — parse an external test report and persist a
//! verdict sidecar (issue #4). The validator then surfaces `W010` for
//! active/verified TestCases whose functions failed or did not run.

use std::path::Path;

use syscribe_model::results::ResultsData;

/// Parse `--format`, defaulting from the file extension when omitted.
/// `session-log` (issue #113) is never inferred — its JSON shape isn't
/// reliably distinguishable from `cargo-json`'s from the extension alone, so
/// it must always be named explicitly.
fn pick_format(explicit: Option<&str>, file: &str) -> Option<&'static str> {
    match explicit {
        Some("cargo-json") => Some("cargo-json"),
        Some("junit") => Some("junit"),
        Some("session-log") => Some("session-log"),
        Some(other) => {
            eprintln!("Unknown --format '{}': expected cargo-json | junit | session-log", other);
            None
        }
        None => {
            if file.ends_with(".xml") {
                Some("junit")
            } else if file.ends_with(".json") || file.ends_with(".ndjson") {
                Some("cargo-json")
            } else {
                eprintln!("Cannot infer --format from '{}'; pass --format cargo-json|junit", file);
                None
            }
        }
    }
}

/// Parse `file` in `format` into a [`ResultsData`].
pub fn parse_file(format: &str, file: &str) -> Option<ResultsData> {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Cannot read results file '{}': {}", file, e);
            return None;
        }
    };
    match format {
        "cargo-json" => Some(ResultsData::parse_cargo_json(&text, file)),
        "junit" => Some(ResultsData::parse_junit(&text, file)),
        // Unlike cargo-json/junit's tolerant line-skipping, session-log is
        // strict: a malformed record is a hard error here, not an empty
        // result set (issue #113's own acceptance bar).
        "session-log" => match ResultsData::parse_session_log(&text, file) {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("Cannot parse session-log results: {e}");
                None
            }
        },
        _ => None,
    }
}

/// `ingest-results` subcommand: parse, then **merge** into the sidecar under
/// `model_root` (REQ-TRS-INGEST-001). The format's own section (`by_leaf` for
/// cargo-json/junit, `by_scenario` for session-log) is replaced; the other is
/// kept. A malformed input exits before anything is written.
pub fn cmd_ingest_results(model_root: &Path, format: Option<&str>, file: &str, run: Option<&str>, elems: &[syscribe_model::element::RawElement]) {
    let fmt = match pick_format(format, file) {
        Some(f) => f,
        None => std::process::exit(1),
    };
    let data = match parse_file(fmt, file) {
        Some(d) => d,
        None => std::process::exit(1),
    };
    // Load the history first so a corrupt one aborts before anything is written.
    let mut history = None;
    if let Some(r) = run {
        match syscribe_model::results::RunHistory::load(model_root) {
            Ok(mut h) => {
                h.record(r, &data);
                history = Some(h);
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
    match data.merge_into_sidecar(model_root) {
        Ok((path, _merged)) => {
            if let (Some(h), Some(r)) = (&history, run) {
                match h.save(model_root) {
                    Ok(p) => println!("Recorded run '{r}' in {}", p.display()),
                    Err(e) => {
                        eprintln!("Cannot write run history: {e}");
                        std::process::exit(1);
                    }
                }
            }
            use syscribe_model::results::Verdict;
            // Class-qualified keys (`classname::name`, GH #259) duplicate their leaf entry;
            // count each test once. A leaf never contains `:`.
            let verdicts = data
                .by_leaf
                .iter()
                .filter(|(k, _)| !k.contains("::"))
                .map(|(_, v)| v)
                .chain(data.by_scenario.values());
            let pass = verdicts.clone().filter(|v| matches!(v, Verdict::Pass)).count();
            let fail = verdicts.clone().filter(|v| matches!(v, Verdict::Fail)).count();
            let flaky = verdicts.clone().filter(|v| matches!(v, Verdict::Flaky)).count();
            let ign = verdicts.filter(|v| matches!(v, Verdict::Ignored)).count();
            let flaky_note = if flaky > 0 { format!(", {flaky} flaky") } else { String::new() };
            println!(
                "Ingested {} test result(s) from {} ({}): {} pass, {} fail{}, {} ignored.",
                data.count, file, fmt, pass, fail, flaky_note, ign
            );
            if fmt != "session-log" {
                print_expected_summary(elems, &data);
            }
            // Worded without a parenthesised format so the only `(<format>)`
            // in the output stays the "Ingested … (<format>)" line above.
            let (own, other) = if fmt == "session-log" {
                ("session-log scenario", "function-level")
            } else {
                ("function-level", "session-log scenario")
            };
            println!(
                "Merged into sidecar: {} (replaced the {} verdicts; kept any {} verdicts)",
                path.display(),
                own,
                other
            );
            if fmt == "session-log" {
                println!("Re-run `trace`/`matrix`/`audit` to see scenarios annotated with their session-log verdict.");
            } else {
                println!("Re-run `validate` (or `validate --deny W010`) to gate on failing/missing tests.");
            }
        }
        Err(e) => {
            eprintln!("Cannot write results sidecar: {}", e);
            std::process::exit(1);
        }
    }
}

/// One line on the `testFunctions` of native `active` TestCases that the run did not execute:
/// missing (not in the report) vs skipped (reported as ignored). Silent when there are none.
fn print_expected_summary(elems: &[syscribe_model::element::RawElement], data: &syscribe_model::results::ResultsData) {
    use syscribe_model::element::ElementType;
    use syscribe_model::results::FnVerdict;
    let mut funcs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for e in elems {
        if e.frontmatter.element_type != Some(ElementType::TestCase) || e.frontmatter.status.as_deref() != Some("active") {
            continue;
        }
        for f in e.frontmatter.test_functions.iter().flatten() {
            if let Some(name) = f.get("function").and_then(|x| x.as_str()) {
                funcs.insert(name.to_string());
            }
        }
    }
    if funcs.is_empty() {
        return;
    }
    let (mut missing, mut skipped) = (Vec::new(), Vec::new());
    for f in &funcs {
        match data.verdict_for(f) {
            FnVerdict::Missing => missing.push(f.as_str()),
            FnVerdict::Ignored => skipped.push(f.as_str()),
            _ => {}
        }
    }
    println!("Expected functions: {}; not run: {} missing, {} skipped", funcs.len(), missing.len(), skipped.len());
    for (label, list) in [("missing", &missing), ("skipped", &skipped)] {
        if !list.is_empty() {
            let shown: Vec<&str> = list.iter().take(10).copied().collect();
            let more = if list.len() > 10 { format!(" … +{}", list.len() - 10) } else { String::new() };
            println!("  {label}: {}{more}", shown.join(", "));
        }
    }
}
