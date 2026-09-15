//! Ingested test-run results (issue #4).
//!
//! `syscribe ingest-results` parses an external test report (cargo libtest JSON
//! or JUnit XML), reduces it to a per-test verdict keyed by the test's leaf
//! name, and persists it to a sidecar at `<model_root>/.syscribe/results.json`.
//! The validator then loads that sidecar and emits `W010` for any `active` /
//! `verified` TestCase whose `testFunctions[].function` last failed or was
//! absent from the run — so "verified" can mean "covered by a test that
//! actually passed".

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::matchers::function_leaf;

/// Verdict for a single test function from the ingested run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Pass,
    Fail,
    Ignored,
}

/// Reduced, persisted view of a test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsData {
    pub schema_version: String,
    /// Source format (`cargo-json` | `junit`).
    pub format: String,
    /// Original report path, for provenance.
    pub source: String,
    /// Unix epoch seconds at ingest time.
    pub ingested_at_unix: u64,
    /// Number of test records ingested.
    pub count: usize,
    /// Verdict per test, keyed by the test's leaf name (see [`function_leaf`]).
    /// On a leaf collision, `Fail` wins over `Pass`/`Ignored`.
    pub by_leaf: HashMap<String, Verdict>,
    /// Verdict per (`TestCase` id, Gherkin scenario title), keyed by the
    /// composed string `"{testCase}::{scenario}"` -- populated only by
    /// `session-log` ingestion (issue #113), the manual/exploratory-
    /// verification counterpart to `by_leaf`'s automated-test function names.
    /// `#[serde(default)]` so a sidecar written before this field existed
    /// still deserializes (empty map).
    #[serde(default)]
    pub by_scenario: HashMap<String, Verdict>,
}

/// One `session-log` input record (issue #113): a single Gherkin scenario
/// exercised manually/exploratorily against a live system. `steps` is opaque
/// (whatever shape the session actually used — commands, expected values) and
/// is never itself validated beyond "not empty": its role is to make the
/// pass/fail claim auditable by a human, not to be machine-checked here.
#[derive(Debug, Deserialize)]
struct SessionLogRecord {
    #[serde(rename = "testCase")]
    test_case: String,
    scenario: String,
    #[serde(default)]
    steps: Vec<serde_json::Value>,
    result: String,
    #[allow(dead_code)]
    #[serde(default)]
    timestamp: Option<String>,
}

/// Lookup outcome for one `testFunctions[].function`.
#[derive(Debug, PartialEq, Eq)]
pub enum FnVerdict {
    Pass,
    Fail,
    Ignored,
    /// The function was not present in the ingested run.
    Missing,
}

impl ResultsData {
    /// Path to the sidecar under a model root.
    pub fn sidecar_path(model_root: &Path) -> PathBuf {
        model_root.join(".syscribe").join("results.json")
    }

    /// Load the sidecar if it exists; `None` when absent or unreadable/invalid.
    pub fn load_sidecar(model_root: &Path) -> Option<Self> {
        let path = Self::sidecar_path(model_root);
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Persist this data to the sidecar, creating `.syscribe/` as needed.
    pub fn write_sidecar(&self, model_root: &Path) -> std::io::Result<PathBuf> {
        let dir = model_root.join(".syscribe");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("results.json");
        let json = serde_json::to_string_pretty(self).expect("ResultsData serialises");
        std::fs::write(&path, json)?;
        Ok(path)
    }

    /// Verdict for a `testFunctions[].function` reference.
    pub fn verdict_for(&self, function_ref: &str) -> FnVerdict {
        match self.by_leaf.get(function_leaf(function_ref)) {
            Some(Verdict::Pass) => FnVerdict::Pass,
            Some(Verdict::Fail) => FnVerdict::Fail,
            Some(Verdict::Ignored) => FnVerdict::Ignored,
            None => FnVerdict::Missing,
        }
    }

    /// Verdict for one (`TestCase`, Gherkin scenario) pair, as recorded by a
    /// `session-log` ingestion (issue #113).
    pub fn scenario_verdict(&self, tc_id: &str, scenario: &str) -> FnVerdict {
        match self.by_scenario.get(&format!("{tc_id}::{scenario}")) {
            Some(Verdict::Pass) => FnVerdict::Pass,
            Some(Verdict::Fail) => FnVerdict::Fail,
            Some(Verdict::Ignored) => FnVerdict::Ignored,
            None => FnVerdict::Missing,
        }
    }

    fn record(map: &mut HashMap<String, Verdict>, name: &str, verdict: Verdict) {
        let leaf = function_leaf(name).to_string();
        map.entry(leaf)
            .and_modify(|existing| {
                // Fail is sticky; otherwise the latest non-ignored wins.
                if *existing != Verdict::Fail && verdict == Verdict::Fail {
                    *existing = Verdict::Fail;
                } else if *existing == Verdict::Ignored {
                    *existing = verdict;
                }
            })
            .or_insert(verdict);
    }

    /// Parse cargo/libtest JSON output (`cargo test -- -Z unstable-options
    /// --format json`, or `--format json` on nightly). Non-JSON and non-`test`
    /// lines are ignored.
    pub fn parse_cargo_json(text: &str, source: &str) -> Self {
        let mut by_leaf = HashMap::new();
        let mut count = 0;
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with('{') {
                continue;
            }
            let v: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v.get("type").and_then(|t| t.as_str()) != Some("test") {
                continue;
            }
            let event = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
            let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let verdict = match event {
                "ok" => Verdict::Pass,
                "failed" => Verdict::Fail,
                "ignored" => Verdict::Ignored,
                _ => continue, // "started" etc.
            };
            Self::record(&mut by_leaf, name, verdict);
            count += 1;
        }
        Self::finish(by_leaf, count, "cargo-json", source)
    }

    /// Parse JUnit XML (`<testcase classname=".." name="..">` with optional
    /// `<failure>` / `<error>` / `<skipped>` children).
    pub fn parse_junit(text: &str, source: &str) -> Self {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut reader = Reader::from_str(text);
        reader.config_mut().trim_text(true);
        let mut by_leaf = HashMap::new();
        let mut count = 0;

        // Track the current open testcase: (leaf-name, verdict-so-far).
        let mut current: Option<(String, Verdict)> = None;
        let mut buf = Vec::new();

        let extract_name = |e: &quick_xml::events::BytesStart| -> Option<String> {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == b"name" {
                    return Some(String::from_utf8_lossy(&attr.value).into_owned());
                }
            }
            None
        };

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let tag = e.local_name();
                    match tag.as_ref() {
                        b"testcase" => {
                            // Flush a previous (Empty-element edge cases aside).
                            if let Some((name, verdict)) = current.take() {
                                Self::record(&mut by_leaf, &name, verdict);
                                count += 1;
                            }
                            let name = extract_name(e).unwrap_or_default();
                            // A self-closing <testcase .../> (Event::Empty) with no
                            // failure/error child is a pass; it is flushed when the
                            // next testcase opens or at EOF.
                            current = Some((name, Verdict::Pass));
                        }
                        b"failure" | b"error" => {
                            if let Some((_, v)) = current.as_mut() {
                                *v = Verdict::Fail;
                            }
                        }
                        b"skipped" => {
                            if let Some((_, v)) = current.as_mut() {
                                if *v != Verdict::Fail {
                                    *v = Verdict::Ignored;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"testcase" {
                        if let Some((name, verdict)) = current.take() {
                            if !name.is_empty() {
                                Self::record(&mut by_leaf, &name, verdict);
                                count += 1;
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        if let Some((name, verdict)) = current.take() {
            if !name.is_empty() {
                Self::record(&mut by_leaf, &name, verdict);
                count += 1;
            }
        }
        Self::finish(by_leaf, count, "junit", source)
    }

    /// Parse a `session-log` report (issue #113): a JSON array of per-scenario
    /// records for manual/exploratory verification --
    ///
    /// ```json
    /// [{"testCase": "TC-X-001", "scenario": "...", "steps": [...],
    ///   "result": "pass", "timestamp": "..."}]
    /// ```
    ///
    /// giving manual verification the same machine-checkable status
    /// automated tests get via `cargo-json`/`junit`, instead of leaving it as
    /// unverifiable prose. Unlike those two tolerant, line-skipping parsers,
    /// this one is **strict**: a record with an empty/missing `testCase`,
    /// `scenario`, or `steps`, or an unrecognized `result`, is a hard parse
    /// error naming the offending record — a hand/agent-authored session log
    /// is exactly the report where a subtly malformed record should not be
    /// silently swallowed into an empty (falsely "nothing ingested, but
    /// exit 0") result set.
    pub fn parse_session_log(text: &str, source: &str) -> Result<Self, String> {
        let records: Vec<SessionLogRecord> = serde_json::from_str(text)
            .map_err(|e| format!("session-log: not a valid JSON array of records: {e}"))?;
        if records.is_empty() {
            return Err("session-log: no records found (empty array)".to_string());
        }
        let mut by_scenario = HashMap::new();
        for (i, r) in records.iter().enumerate() {
            if r.test_case.trim().is_empty() {
                return Err(format!("session-log: record {i} has an empty/missing testCase"));
            }
            if r.scenario.trim().is_empty() {
                return Err(format!(
                    "session-log: record {i} ('{}') has an empty/missing scenario",
                    r.test_case
                ));
            }
            if r.steps.is_empty() {
                return Err(format!(
                    "session-log: record {i} ('{}' / '{}') has empty or missing steps",
                    r.test_case, r.scenario
                ));
            }
            let verdict = match r.result.as_str() {
                "pass" => Verdict::Pass,
                "fail" => Verdict::Fail,
                "unknown" => Verdict::Ignored,
                other => {
                    return Err(format!(
                        "session-log: record {i} ('{}' / '{}') has unrecognized result '{other}' (expected pass|fail|unknown)",
                        r.test_case, r.scenario
                    ))
                }
            };
            by_scenario.insert(format!("{}::{}", r.test_case, r.scenario), verdict);
        }
        let count = records.len();
        let mut data = Self::finish(HashMap::new(), count, "session-log", source);
        data.by_scenario = by_scenario;
        Ok(data)
    }

    fn finish(by_leaf: HashMap<String, Verdict>, count: usize, format: &str, source: &str) -> Self {
        let ingested_at_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        ResultsData {
            schema_version: "1.0".to_string(),
            format: format.to_string(),
            source: source.to_string(),
            ingested_at_unix,
            count,
            by_leaf,
            by_scenario: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_json_pass_fail() {
        let text = r#"
{ "type": "suite", "event": "started", "test_count": 2 }
{ "type": "test", "event": "started", "name": "mutex::tests::acquire_ok" }
{ "type": "test", "name": "mutex::tests::acquire_ok", "event": "ok" }
{ "type": "test", "name": "mutex::tests::release_fails", "event": "failed" }
Running unittests src/lib.rs
"#;
        let r = ResultsData::parse_cargo_json(text, "cargo.json");
        assert_eq!(r.verdict_for("mutex::tests::acquire_ok"), FnVerdict::Pass);
        assert_eq!(r.verdict_for("crate::release_fails"), FnVerdict::Fail);
        assert_eq!(r.verdict_for("mutex::tests::never_ran"), FnVerdict::Missing);
    }

    #[test]
    fn junit_pass_fail_skip() {
        let text = r#"<?xml version="1.0"?>
<testsuite name="s" tests="3">
  <testcase classname="MutexTest" name="AcquireWhenFree"/>
  <testcase classname="MutexTest" name="ReleaseWhenHeld"><failure message="boom"/></testcase>
  <testcase classname="MutexTest" name="Skipped"><skipped/></testcase>
</testsuite>"#;
        let r = ResultsData::parse_junit(text, "junit.xml");
        assert_eq!(r.verdict_for("MutexTest.AcquireWhenFree"), FnVerdict::Pass);
        assert_eq!(r.verdict_for("MutexTest.ReleaseWhenHeld"), FnVerdict::Fail);
        assert_eq!(r.verdict_for("Skipped"), FnVerdict::Ignored);
    }

    #[test]
    fn session_log_pass_fail_unknown() {
        let text = r#"[
            {"testCase": "TC-X-001", "scenario": "A passes", "steps": [{"cmd": "curl"}], "result": "pass", "timestamp": "2026-09-12T08:00:00Z"},
            {"testCase": "TC-X-001", "scenario": "B fails", "steps": [{"cmd": "curl"}], "result": "fail"},
            {"testCase": "TC-X-002", "scenario": "C unknown", "steps": [{"cmd": "curl"}], "result": "unknown"}
        ]"#;
        let r = ResultsData::parse_session_log(text, "session.json").expect("valid session-log");
        assert_eq!(r.scenario_verdict("TC-X-001", "A passes"), FnVerdict::Pass);
        assert_eq!(r.scenario_verdict("TC-X-001", "B fails"), FnVerdict::Fail);
        assert_eq!(r.scenario_verdict("TC-X-002", "C unknown"), FnVerdict::Ignored);
        assert_eq!(r.scenario_verdict("TC-X-001", "Never ran"), FnVerdict::Missing);
        assert_eq!(r.count, 3);
        assert_eq!(r.format, "session-log");
    }

    #[test]
    fn session_log_rejects_empty_steps() {
        let text = r#"[{"testCase": "TC-X-001", "scenario": "No steps", "steps": [], "result": "pass"}]"#;
        let err = ResultsData::parse_session_log(text, "session.json").unwrap_err();
        assert!(err.contains("steps"), "{err}");
        assert!(err.contains("TC-X-001"), "{err}");
    }

    #[test]
    fn session_log_rejects_missing_steps_field() {
        let text = r#"[{"testCase": "TC-X-001", "scenario": "No steps field", "result": "pass"}]"#;
        let err = ResultsData::parse_session_log(text, "session.json").unwrap_err();
        assert!(err.contains("steps"), "{err}");
    }

    #[test]
    fn session_log_rejects_unrecognized_result() {
        let text = r#"[{"testCase": "TC-X-001", "scenario": "S", "steps": [{"cmd": "x"}], "result": "maybe"}]"#;
        let err = ResultsData::parse_session_log(text, "session.json").unwrap_err();
        assert!(err.contains("maybe"), "{err}");
    }

    #[test]
    fn session_log_rejects_empty_array() {
        let err = ResultsData::parse_session_log("[]", "session.json").unwrap_err();
        assert!(err.contains("no records"), "{err}");
    }

    #[test]
    fn session_log_rejects_malformed_json() {
        let err = ResultsData::parse_session_log("not json", "session.json").unwrap_err();
        assert!(err.contains("session-log"), "{err}");
    }
}
