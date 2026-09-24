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
pub fn cmd_ingest_results(model_root: &Path, format: Option<&str>, file: &str) {
    let fmt = match pick_format(format, file) {
        Some(f) => f,
        None => std::process::exit(1),
    };
    let data = match parse_file(fmt, file) {
        Some(d) => d,
        None => std::process::exit(1),
    };
    match data.merge_into_sidecar(model_root) {
        Ok((path, _merged)) => {
            use syscribe_model::results::Verdict;
            let verdicts = data.by_leaf.values().chain(data.by_scenario.values());
            let pass = verdicts.clone().filter(|v| matches!(v, Verdict::Pass)).count();
            let fail = verdicts.clone().filter(|v| matches!(v, Verdict::Fail)).count();
            let ign = verdicts.filter(|v| matches!(v, Verdict::Ignored)).count();
            println!(
                "Ingested {} test result(s) from {} ({}): {} pass, {} fail, {} ignored.",
                data.count, file, fmt, pass, fail, ign
            );
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
