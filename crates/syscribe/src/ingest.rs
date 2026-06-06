//! `syscribe ingest-results` — parse an external test report and persist a
//! verdict sidecar (issue #4). The validator then surfaces `W010` for
//! active/verified TestCases whose functions failed or did not run.

use std::path::Path;

use syscribe_model::results::ResultsData;

/// Parse `--format`, defaulting from the file extension when omitted.
fn pick_format(explicit: Option<&str>, file: &str) -> Option<&'static str> {
    match explicit {
        Some("cargo-json") => Some("cargo-json"),
        Some("junit") => Some("junit"),
        Some(other) => {
            eprintln!("Unknown --format '{}': expected cargo-json | junit", other);
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
    Some(match format {
        "cargo-json" => ResultsData::parse_cargo_json(&text, file),
        "junit" => ResultsData::parse_junit(&text, file),
        _ => return None,
    })
}

/// `ingest-results` subcommand: parse and write the sidecar under `model_root`.
pub fn cmd_ingest_results(model_root: &Path, format: Option<&str>, file: &str) {
    let fmt = match pick_format(format, file) {
        Some(f) => f,
        None => std::process::exit(1),
    };
    let data = match parse_file(fmt, file) {
        Some(d) => d,
        None => std::process::exit(1),
    };
    match data.write_sidecar(model_root) {
        Ok(path) => {
            let pass = data.by_leaf.values().filter(|v| matches!(v, syscribe_model::results::Verdict::Pass)).count();
            let fail = data.by_leaf.values().filter(|v| matches!(v, syscribe_model::results::Verdict::Fail)).count();
            let ign = data.by_leaf.values().filter(|v| matches!(v, syscribe_model::results::Verdict::Ignored)).count();
            println!(
                "Ingested {} test result(s) from {} ({}): {} pass, {} fail, {} ignored.",
                data.count, file, fmt, pass, fail, ign
            );
            println!("Wrote sidecar: {}", path.display());
            println!("Re-run `validate` (or `validate --deny W010`) to gate on failing/missing tests.");
        }
        Err(e) => {
            eprintln!("Cannot write results sidecar: {}", e);
            std::process::exit(1);
        }
    }
}
