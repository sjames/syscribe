//! `syscribe ingest-results` — parse an external test report and persist a
//! verdict sidecar (issue #4). The validator then surfaces `W010` for
//! active/verified TestCases whose functions failed or did not run.

use std::path::Path;

use syscribe_model::results::ResultsData;

/// Parse `--format`, defaulting from the file extension when omitted.
/// `session-log` (issue #113) is never inferred — its JSON shape isn't
/// reliably distinguishable from `cargo-json`'s from the extension alone, so
/// it must always be named explicitly. The same holds for `records`.
fn pick_format(explicit: Option<&str>, file: &str) -> Option<String> {
    match explicit {
        Some("cargo-json") => Some("cargo-json".into()),
        Some("junit") => Some("junit".into()),
        Some("session-log") => Some("session-log".into()),
        Some("records") => Some("records".into()),
        Some("robot") => Some("robot".into()),
        Some("cucumber") => Some("cucumber".into()),
        // A results plugin converts any report into records (REQ-TRS-RESPLUG-001); never inferred.
        Some(p) if p.strip_prefix("plugin:").is_some_and(|a| !a.is_empty()) => Some(p.to_string()),
        Some(other) => {
            eprintln!("Unknown --format '{}': expected cargo-json | junit | session-log | records | robot | cucumber | plugin:<alias>", other);
            None
        }
        None => {
            if file.ends_with(".xml") {
                // a Robot Framework result is also `.xml`; its root element tells it from a JUnit report
                let head = std::fs::read_to_string(file).map(|t| t.chars().take(2048).collect::<String>()).unwrap_or_default();
                if head.contains("<robot ") || head.contains("<robot>") { Some("robot".into()) } else { Some("junit".into()) }
            } else if file.ends_with(".json") || file.ends_with(".ndjson") {
                // the Cucumber `json` formatter writes one array of features, each with `elements`
                let head: String = std::fs::read_to_string(file).map(|t| t.chars().take(4096).collect()).unwrap_or_default();
                let looks_like_cucumber = head.trim_start().starts_with('[') && head.contains("\"elements\"") && head.contains("\"keyword\"");
                if looks_like_cucumber { Some("cucumber".into()) } else { Some("cargo-json".into()) }
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
        // The normalised record (REQ-TRS-RESREC-001) is strict in the same way.
        "records" => match ResultsData::parse_records(&text, file) {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("Cannot parse records: {e}");
                None
            }
        },
        _ => None,
    }
}

/// The first few keys of the records that joined nothing, for a message.
fn unmatched_keys(data: &ResultsData) -> String {
    let mut keys: Vec<&str> = data.unmatched.iter().map(|r| r.test_case.as_str()).collect();
    keys.dedup();
    let more = keys.len().saturating_sub(5);
    let shown = keys.into_iter().take(5).collect::<Vec<_>>().join(", ");
    if more > 0 { format!("{shown}, and {more} more") } else { shown }
}

/// `ingest-results` subcommand: parse, then **merge** into the sidecar under
/// `model_root` (REQ-TRS-INGEST-001). The format's own section (`by_leaf` for
/// cargo-json/junit, `by_scenario` for session-log) is replaced; the other is
/// kept. A malformed input exits before anything is written.
/// The options that only the record formats (`records`, `plugin:<alias>`) take.
#[derive(Default)]
pub struct RecordOpts<'a> {
    pub replace: bool,
    pub strict: bool,
    pub environment: Option<&'a str>,
    pub build_commit: Option<&'a str>,
    pub operator: Option<&'a str>,
    /// Robot only: the zone its zone-less local timestamps were written in (`Z` or `+HH:MM`).
    pub utc_offset: Option<&'a str>,
    /// Files to reference (by hash, never copied) from every record.
    pub artefacts: Vec<&'a str>,
}

/// Read a Cucumber `json` formatter report and map its scenarios to records by tag against the model.
fn parse_cucumber(file: &str, elems: &[syscribe_model::element::RawElement]) -> ResultsData {
    use syscribe_model::result_cucumber;
    let fail = |msg: String| -> ! {
        eprintln!("Cannot parse cucumber results: {msg}\nNothing was written.");
        std::process::exit(1)
    };
    let text = std::fs::read_to_string(file).unwrap_or_else(|e| fail(format!("cannot read '{file}': {e}")));
    let report = result_cucumber::parse(&text).unwrap_or_else(|e| fail(e));
    let mut records = result_cucumber::to_records(&report, elems);
    syscribe_model::result_records::validate_all(&mut records).unwrap_or_else(|e| fail(e));
    let mut data = ResultsData::from_records(records, report.scenarios.len(), file);
    data.format = "records".to_string();
    data
}

/// Read a Robot Framework result and map its tests to records by tag against the model.
fn parse_robot(file: &str, utc_offset: &str, elems: &[syscribe_model::element::RawElement]) -> ResultsData {
    use syscribe_model::result_robot;
    let fail = |msg: String| -> ! {
        eprintln!("Cannot parse robot results: {msg}\nNothing was written.");
        std::process::exit(1)
    };
    let text = std::fs::read_to_string(file).unwrap_or_else(|e| fail(format!("cannot read '{file}': {e}")));
    let report = result_robot::parse(&text).unwrap_or_else(|e| fail(e));
    let mut records = result_robot::to_records(&report, elems, utc_offset).unwrap_or_else(|e| fail(e));
    syscribe_model::result_records::validate_all(&mut records).unwrap_or_else(|e| fail(e));
    let mut data = ResultsData::from_records(records, report.tests.len(), file);
    data.format = "records".to_string();
    data
}

/// Convert `file` with the results plugin `alias` into a records ingest (REQ-TRS-RESPLUG-001). Every failure is
/// reported with its code and the ingest stops before anything is written.
fn parse_with_plugin(model_root: &Path, alias: &str, file: &str, run: Option<&str>, config: Option<&str>, environment: Option<&str>) -> ResultsData {
    use syscribe_model::plugins::{config::load_plugins, results};
    let fail = |code: &str, msg: String| -> ! {
        eprintln!("{code}: {msg}\nNothing was written.");
        std::process::exit(1)
    };
    let plugins = load_plugins(model_root);
    let Some(entry) = plugins.get(alias) else {
        fail("E553", format!("ingest-results --format plugin:{alias}: there is no [plugins.{alias}] entry in .syscribe.toml"));
    };
    if !entry.is_results() {
        fail(
            "E552",
            format!("[plugins.{alias}] is not a results plugin (it has {}); add kind = \"results\" to use it with --format plugin:{alias}", entry.kind.as_deref().map_or("no kind".to_string(), |k| format!("kind = \"{k}\""))),
        );
    }
    let req = results::build_request(alias, Path::new(file), model_root, run, config, environment).unwrap_or_else(|e| fail("E553", format!("results plugin '{alias}': {e}")));
    let raw = results::invoke(entry, &req, model_root).unwrap_or_else(|e| fail("E553", results::explain(alias, &e)));
    let out = results::parse_envelope(alias, &raw).unwrap_or_else(|e| fail("E553", e));
    for d in &out.diagnostics {
        eprintln!("results plugin '{alias}': {d}");
    }
    ResultsData::parse_records(&out.records_json, file).unwrap_or_else(|e| fail("E553", format!("results plugin '{alias}': its records were rejected: {e}")))
}

pub fn cmd_ingest_results(model_root: &Path, format: Option<&str>, file: &str, run: Option<&str>, config: Option<&str>, opts: &RecordOpts, elems: &[syscribe_model::element::RawElement]) {
    syscribe_model::fsio::dry_banner();
    let (replace, strict, environment) = (opts.replace, opts.strict, opts.environment);
    let fmt_owned = match pick_format(format, file) {
        Some(f) => f,
        None => std::process::exit(1),
    };
    let fmt = fmt_owned.as_str();
    let plugin = fmt.strip_prefix("plugin:");
    let robot = fmt == "robot";
    let cucumber = fmt == "cucumber";
    let records_like = fmt == "records" || plugin.is_some() || robot || cucumber;
    if (replace || strict || environment.is_some() || opts.build_commit.is_some() || opts.operator.is_some() || !opts.artefacts.is_empty()) && !records_like {
        eprintln!("ingest-results: --replace, --strict, --environment, --build-commit, --operator and --artefact apply to --format records, robot, cucumber and plugin:<alias> only; nothing was written.");
        std::process::exit(1);
    }
    if opts.utc_offset.is_some() && !robot {
        eprintln!("ingest-results: --utc-offset applies to --format robot only (it is the zone Robot's zone-less timestamps were written in); nothing was written.");
        std::process::exit(1);
    }
    let offset = opts.utc_offset.unwrap_or("Z");
    if robot && !(offset == "Z" || (offset.len() == 6 && (offset.starts_with('+') || offset.starts_with('-')) && offset.as_bytes()[3] == b':' && offset[1..3].parse::<u32>().is_ok_and(|h| h < 24) && offset[4..].parse::<u32>().is_ok_and(|m| m < 60))) {
        eprintln!("ingest-results: --utc-offset '{offset}' must be Z or +HH:MM / -HH:MM; nothing was written.");
        std::process::exit(1);
    }
    let mut data = match plugin {
        Some(alias) => parse_with_plugin(model_root, alias, file, run, config, environment),
        None if robot => parse_robot(file, offset, elems),
        None if cucumber => parse_cucumber(file, elems),
        None => match parse_file(fmt, file) {
            Some(d) => d,
            None => std::process::exit(1),
        },
    };
    if records_like {
        // `--operator` and `--artefact` apply to every record; an operator a record names is kept.
        if let Some(op) = opts.operator {
            for r in &mut data.records {
                let p = r.provenance.get_or_insert_with(Default::default);
                if p.operator.is_none() {
                    p.operator = Some(op.to_string());
                }
            }
        }
        for path in &opts.artefacts {
            let art = syscribe_model::result_records::file_artefact(path).unwrap_or_else(|e| {
                eprintln!("Cannot use --artefact: {e}\nNothing was written.");
                std::process::exit(1)
            });
            for r in &mut data.records {
                let p = r.provenance.get_or_insert_with(Default::default);
                if !p.artefacts.iter().any(|a| a.uri == art.uri) {
                    p.artefacts.push(art.clone());
                }
            }
        }
        // `--build-commit` fills the commit for every record that has none; a record's own value is kept.
        if let Some(c) = opts.build_commit {
            for r in &mut data.records {
                let b = r.provenance.get_or_insert_with(Default::default).build.get_or_insert_with(Default::default);
                if b.commit.is_none() {
                    b.commit = Some(c.to_string());
                }
            }
        }
        // `--environment` names the rig for every record that does not name one itself; a record's own value is kept.
        if let Some(env) = environment {
            for r in &mut data.records {
                let p = r.provenance.get_or_insert_with(Default::default);
                if p.environment.is_none() {
                    p.environment = Some(env.to_string());
                }
            }
        }
        // An environment must be a TestEnvironment of the model; a typo is rejected, not stored.
        if let Err(e) = data.resolve_environments(elems) {
            eprintln!("Cannot parse records: {e}\nNothing was written.");
            std::process::exit(1);
        }
        // Attach each record to its TestCase by id, qualified name or extRef. An ambiguous key, or one naming
        // something that is not a TestCase, rejects the ingest; a key naming nothing is reported below.
        if let Err(e) = data.join_test_cases(elems) {
            eprintln!("Cannot parse records: {e}\nNothing was written.");
            std::process::exit(1);
        }
        if strict && !data.unmatched.is_empty() {
            eprintln!(
                "ingest-results --strict: {} record(s) name no TestCase of this model ({}); nothing was written.",
                data.unmatched.len(),
                unmatched_keys(&data)
            );
            std::process::exit(1);
        }
        data.replace_records = replace;
    }
    // `--config` names a Configuration (id or qualified name); it is stored by id (REQ-TRS-CFGRES-001).
    let config_key: Option<String> = match config {
        None => None,
        Some(c) => match elems.iter().find(|e| {
            matches!(e.frontmatter.element_type, Some(syscribe_model::element::ElementType::Configuration))
                && (e.frontmatter.id.as_deref() == Some(c) || e.qualified_name == c)
        }) {
            Some(e) => Some(e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())),
            None => {
                eprintln!("ingest-results: --config '{c}' is not a Configuration of this model; nothing was written.");
                std::process::exit(1);
            }
        },
    };
    // Load the history first so a corrupt one aborts before anything is written.
    let mut history = None;
    if let Some(r) = run {
        match syscribe_model::results::RunHistory::load(model_root) {
            Ok(mut h) => {
                // Refuse before the sidecar is written, so a protected history never leaves the two out of step.
                if let Err(e) = syscribe_model::fsio::check(syscribe_model::results::RunHistory::path(model_root)) {
                    eprintln!("Cannot write run history: {e}");
                    std::process::exit(1);
                }
                h.record_for(r, &data, config_key.as_deref());
                history = Some(h);
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
    let merged = match &config_key {
        Some(c) => data.merge_config_into_sidecar(model_root, c),
        None => data.merge_into_sidecar(model_root),
    };
    match merged {
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
            if records_like {
                let n = |f: &dyn Fn(&Verdict) -> bool| data.records.iter().filter(|r| f(&r.verdict)).count();
                println!(
                    "Ingested {} result record(s) from {} ({}): {} pass, {} fail, {} flaky, {} blocked, {} not-run, {} inconclusive, {} waived, {} ignored.",
                    data.count,
                    file,
                    if let Some(a) = plugin { format!("records via plugin {a}") } else if robot { "robot".to_string() } else if cucumber { "cucumber".to_string() } else { "records".to_string() },
                    n(&|v| matches!(v, Verdict::Pass)),
                    n(&|v| matches!(v, Verdict::Fail)),
                    n(&|v| matches!(v, Verdict::Flaky)),
                    n(&|v| matches!(v, Verdict::Blocked)),
                    n(&|v| matches!(v, Verdict::NotRun)),
                    n(&|v| matches!(v, Verdict::Inconclusive)),
                    n(&|v| matches!(v, Verdict::Waived)),
                    n(&|v| matches!(v, Verdict::Ignored)),
                );
                let superseded = data.count.saturating_sub(data.records.len() + data.unmatched.len());
                if (robot || cucumber) && data.count > data.records.len() + data.unmatched.len() {
                    println!("{} {}(s) were folded into {} record(s): those of one TestCase and scenario count as one, the worst verdict deciding.", data.count, if robot { "test" } else { "scenario" }, data.records.len() + data.unmatched.len());
                } else if superseded > 0 {
                    println!("{superseded} record(s) repeated a test, scenario and environment already in the file; the last of each was kept.");
                }
                if !data.unmatched.is_empty() {
                    println!(
                        "{} of {} record(s) name no TestCase of this model and count as evidence for nothing ({}); `results failures` lists them. Use --strict to reject such a file.",
                        data.unmatched.len(),
                        data.count,
                        unmatched_keys(&data)
                    );
                }
                println!(
                    "Merged into sidecar: {} ({}; kept any function-level and session-log verdicts)",
                    path.display(),
                    if replace { "replaced all records" } else { "superseded the records with the same test, scenario and environment, kept the others" }
                );
                return;
            }
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
            match &config_key {
                Some(c) => println!(
                    "Merged into sidecar: {} (configuration {c}: replaced its {} verdicts; kept its {} verdicts, the global ones and other configurations)",
                    path.display(),
                    own,
                    other
                ),
                None => println!(
                    "Merged into sidecar: {} (replaced the {} verdicts; kept any {} verdicts)",
                    path.display(),
                    own,
                    other
                ),
            }
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
