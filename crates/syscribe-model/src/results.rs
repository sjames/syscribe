//! Ingested test-run results (issue #4).
//!
//! `syscribe ingest-results` parses an external test report (cargo libtest JSON
//! or JUnit XML), reduces it to a per-test verdict keyed by the test's leaf
//! name, and persists it to a sidecar at `<model_root>/.syscribe/results.json`.
//! The validator then loads that sidecar and emits `W010` for any `active` /
//! `verified` TestCase whose `testFunctions[].function` last failed or was
//! absent from the run — so "verified" can mean "covered by a test that
//! actually passed".
//!
//! The sidecar holds two independent sections — `by_leaf` (function-level
//! verdicts from `cargo-json`/`junit`) and `by_scenario` (scenario verdicts from
//! `session-log`, issue #113). Each ingest **merges** into the existing sidecar
//! and replaces only its own section (REQ-TRS-INGEST-001: session-log verdicts
//! are kept "alongside the existing per-function verdicts"); see
//! [`ResultsData::merged_over`].

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
    /// Passed only after a retry (JUnit `flakyFailure`/`rerunFailure`, GH #259): not a pass.
    Flaky,
}

/// Provenance of one sidecar section: which ingest last replaced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionMeta {
    /// Source format of that ingest (`cargo-json` | `junit` | `session-log`).
    pub format: String,
    /// Original report path.
    pub source: String,
    /// Unix epoch seconds at that ingest.
    pub ingested_at_unix: u64,
    /// Number of records that ingest contributed.
    pub count: usize,
}

/// Which sidecar section an ingest format owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// `by_leaf` — function-level verdicts (`cargo-json`, `junit`).
    Leaf,
    /// `by_scenario` — scenario verdicts (`session-log`).
    Scenario,
}

/// Reduced, persisted view of a test run.
///
/// After a merge the top-level `format`/`source`/`ingested_at_unix`/`count`
/// describe the **latest** ingest (unchanged meaning for a single-format
/// sidecar); `leaf_meta`/`scenario_meta` record which ingest each section came
/// from. Both are optional, so a sidecar written before they existed still reads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsData {
    pub schema_version: String,
    /// Source format of the latest ingest (`cargo-json` | `junit` | `session-log`).
    pub format: String,
    /// Original report path of the latest ingest, for provenance.
    pub source: String,
    /// Unix epoch seconds of the latest ingest.
    pub ingested_at_unix: u64,
    /// Number of test records the latest ingest contributed.
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
    /// Provenance of `by_leaf` (the last function-level ingest); absent in a
    /// sidecar written before merging existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leaf_meta: Option<SectionMeta>,
    /// Provenance of `by_scenario` (the last session-log ingest).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_meta: Option<SectionMeta>,
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

/// The `function` strings declared under a TestCase's `testFunctions:`.
pub fn tc_function_refs(tc: &crate::element::RawElement) -> Vec<String> {
    let func_key = serde_yaml::Value::String("function".into());
    tc.frontmatter
        .test_functions
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|tf| match tf {
            serde_yaml::Value::Mapping(map) => match map.get(&func_key) {
                Some(serde_yaml::Value::String(f)) => Some(f.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// The exact `Scenario:`/`Scenario Outline:` titles declared in a TestCase's
/// body, in document order: the identity a `session-log` record names.
fn gherkin_scenario_titles(doc: &str) -> Vec<String> {
    doc.lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("Scenario Outline:")
                .or_else(|| t.strip_prefix("Scenario:"))
                .map(|title| title.trim().to_string())
        })
        .collect()
}

/// Aggregate a TestCase's ingested verdict (issue #21): scored against its
/// `testFunctions:` when it has any, else against its Gherkin scenario titles
/// (`session-log`, issue #113). `Unknown` without results, without anything to
/// score, or when some function / scenario is missing or ignored and none failed.
/// The one definition: the CLI's `tc_verdict` and the derived GSN diagram read it.
pub fn testcase_verdict(tc: &crate::element::RawElement, results: Option<&ResultsData>) -> crate::safety_case::Verdict {
    use crate::safety_case::Verdict as V;
    let Some(results) = results else { return V::Unknown };
    let funcs = tc_function_refs(tc);
    if !funcs.is_empty() {
        let mut all_pass = true;
        for f in &funcs {
            match results.verdict_for(f) {
                FnVerdict::Fail => return V::Fail,
                FnVerdict::Pass => {}
                FnVerdict::Ignored | FnVerdict::Missing | FnVerdict::Flaky => all_pass = false,
            }
        }
        return if all_pass { V::Pass } else { V::Unknown };
    }
    let Some(tc_id) = tc.frontmatter.id.as_deref() else { return V::Unknown };
    let scenarios = gherkin_scenario_titles(&tc.doc);
    if scenarios.is_empty() {
        return V::Unknown;
    }
    let mut all_pass = true;
    for s in &scenarios {
        match results.scenario_verdict(tc_id, s) {
            FnVerdict::Fail => return V::Fail,
            FnVerdict::Pass => {}
            FnVerdict::Ignored | FnVerdict::Missing | FnVerdict::Flaky => all_pass = false,
        }
    }
    if all_pass {
        V::Pass
    } else {
        V::Unknown
    }
}

/// Lookup outcome for one `testFunctions[].function`.
#[derive(Debug, PartialEq, Eq)]
pub enum FnVerdict {
    Pass,
    Fail,
    Ignored,
    /// Passed only after a retry (GH #259).
    Flaky,
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

    /// The section this ingest's format owns: `session-log` → `by_scenario`,
    /// every other (function-level) format → `by_leaf`.
    pub fn section(&self) -> Section {
        if self.format == "session-log" { Section::Scenario } else { Section::Leaf }
    }

    /// This ingest's own provenance record.
    fn meta(&self) -> SectionMeta {
        SectionMeta {
            format: self.format.clone(),
            source: self.source.clone(),
            ingested_at_unix: self.ingested_at_unix,
            count: self.count,
        }
    }

    /// Merge this freshly parsed ingest over an `existing` sidecar
    /// (REQ-TRS-INGEST-001): this ingest's own section **replaces** the existing
    /// one (re-ingesting the same kind never accumulates), the other section is
    /// kept as is, and the top-level fields describe this (latest) ingest.
    /// `existing = None` (no or unreadable sidecar) is treated as empty.
    pub fn merged_over(&self, existing: Option<&ResultsData>) -> ResultsData {
        let mut out = self.clone();
        let own = Some(self.meta());
        match self.section() {
            Section::Leaf => {
                out.leaf_meta = own;
                if let Some(old) = existing {
                    out.by_scenario = old.by_scenario.clone();
                    out.scenario_meta = old.scenario_meta.clone().or_else(|| legacy_meta(old, Section::Scenario));
                }
            }
            Section::Scenario => {
                out.scenario_meta = own;
                if let Some(old) = existing {
                    out.by_leaf = old.by_leaf.clone();
                    out.leaf_meta = old.leaf_meta.clone().or_else(|| legacy_meta(old, Section::Leaf));
                }
            }
        }
        out
    }

    /// Merge this ingest into the sidecar under `model_root` (see
    /// [`Self::merged_over`]) and write it; returns the path and the merged data.
    /// A missing or unreadable existing sidecar is treated as empty. Callers parse
    /// the new input first, so a malformed input never reaches this point and
    /// leaves the existing sidecar untouched.
    pub fn merge_into_sidecar(&self, model_root: &Path) -> std::io::Result<(PathBuf, ResultsData)> {
        let merged = self.merged_over(Self::load_sidecar(model_root).as_ref());
        let path = merged.write_sidecar(model_root)?;
        Ok((path, merged))
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
        // A class-qualified reference (`C#N`, `C::N`) is looked up exactly first; JUnit
        // ingestion stores `classname::name` beside the leaf (GH #259). A leaf never
        // contains `:`, so qualified keys cannot collide with leaves.
        let mut qualified = function_ref.replace('#', "::");
        if !qualified.contains("::") {
            // `com.A.test_x` → `com.A::test_x` (the last dot separates class and test).
            if let Some((class, name)) = qualified.rsplit_once('.') {
                qualified = format!("{class}::{name}");
            }
        }
        let hit = if qualified.contains("::") { self.by_leaf.get(&qualified) } else { None };
        match hit.or_else(|| self.by_leaf.get(function_leaf(function_ref))) {
            Some(Verdict::Pass) => FnVerdict::Pass,
            Some(Verdict::Fail) => FnVerdict::Fail,
            Some(Verdict::Ignored) => FnVerdict::Ignored,
            Some(Verdict::Flaky) => FnVerdict::Flaky,
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
            Some(Verdict::Flaky) => FnVerdict::Flaky,
            None => FnVerdict::Missing,
        }
    }

    /// Worst-wins ordering used when two results share a key: Fail > Flaky > Pass > Ignored.
    fn rank(v: Verdict) -> u8 {
        match v {
            Verdict::Fail => 3,
            Verdict::Flaky => 2,
            Verdict::Pass => 1,
            Verdict::Ignored => 0,
        }
    }

    fn record(map: &mut HashMap<String, Verdict>, name: &str, verdict: Verdict) {
        Self::record_key(map, function_leaf(name), verdict);
    }

    fn record_key(map: &mut HashMap<String, Verdict>, key: &str, verdict: Verdict) {
        map.entry(key.to_string())
            .and_modify(|existing| {
                if Self::rank(verdict) > Self::rank(*existing) {
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
        // The open testcase's classname and whether it carried a retry marker.
        let mut class: String = String::new();
        let mut retried = false;
        let mut buf = Vec::new();

        let extract = |e: &quick_xml::events::BytesStart, key: &[u8]| -> Option<String> {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == key {
                    return Some(String::from_utf8_lossy(&attr.value).into_owned());
                }
            }
            None
        };
        // Record a finished testcase: leaf key and, when a classname is known, the
        // qualified `classname::name` key (GH #259). A retry marker on an otherwise
        // passing testcase makes it flaky.
        let finish_case = |by_leaf: &mut HashMap<String, Verdict>, name: &str, class: &str, verdict: Verdict, retried: bool| {
            let verdict = if retried && verdict == Verdict::Pass { Verdict::Flaky } else { verdict };
            Self::record(by_leaf, name, verdict);
            if !class.is_empty() {
                Self::record_key(by_leaf, &format!("{class}::{}", function_leaf(name)), verdict);
            }
        };

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let tag = e.local_name();
                    match tag.as_ref() {
                        b"testcase" => {
                            // Flush a previous (Empty-element edge cases aside).
                            if let Some((name, verdict)) = current.take() {
                                finish_case(&mut by_leaf, &name, &class, verdict, retried);
                                count += 1;
                            }
                            let name = extract(e, b"name").unwrap_or_default();
                            class = extract(e, b"classname").unwrap_or_default();
                            retried = false;
                            // A self-closing <testcase .../> (Event::Empty) with no
                            // failure/error child is a pass; it is flushed when the
                            // next testcase opens or at EOF.
                            current = Some((name, Verdict::Pass));
                        }
                        b"flakyFailure" | b"flakyError" | b"rerunFailure" | b"rerunError" => {
                            retried = true;
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
                                finish_case(&mut by_leaf, &name, &class, verdict, retried);
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
                finish_case(&mut by_leaf, &name, &class, verdict, retried);
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
            leaf_meta: None,
            scenario_meta: None,
        }
    }
}

/// Provenance for `section` of a sidecar written before per-section metadata
/// existed: its top-level fields, when that single-format ingest owned the
/// section and the section is non-empty.
fn legacy_meta(old: &ResultsData, section: Section) -> Option<SectionMeta> {
    let non_empty = match section {
        Section::Leaf => !old.by_leaf.is_empty(),
        Section::Scenario => !old.by_scenario.is_empty(),
    };
    (non_empty && old.section() == section).then(|| old.meta())
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

    // ── merge semantics (REQ-TRS-INGEST-001) ────────────────────────────────

    const CARGO_PASS: &str = r#"{"type":"test","event":"ok","name":"crate::tests::it_works"}"#;
    const CARGO_FAIL: &str = r#"{"type":"test","event":"failed","name":"crate::tests::other_test"}"#;
    const SESSION_FAIL: &str = r#"[{"testCase":"TC-SL-002","scenario":"A scenario","steps":["x"],"result":"fail"}]"#;
    const SESSION_PASS: &str = r#"[{"testCase":"TC-SL-009","scenario":"Other","steps":["y"],"result":"pass"}]"#;

    fn tmp_root(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "syscribe-results-merge-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn cargo_json_then_session_log_keeps_both_sections() {
        let root = tmp_root("cs");
        ResultsData::parse_cargo_json(CARGO_PASS, "cargo.json").merge_into_sidecar(&root).unwrap();
        ResultsData::parse_session_log(SESSION_FAIL, "s.json").unwrap().merge_into_sidecar(&root).unwrap();
        let d = ResultsData::load_sidecar(&root).unwrap();
        assert_eq!(d.verdict_for("crate::tests::it_works"), FnVerdict::Pass, "by_leaf kept");
        assert_eq!(d.scenario_verdict("TC-SL-002", "A scenario"), FnVerdict::Fail);
        assert_eq!(d.format, "session-log", "top level describes the latest ingest");
        assert_eq!(d.leaf_meta.as_ref().map(|m| m.format.as_str()), Some("cargo-json"));
        assert_eq!(d.scenario_meta.as_ref().map(|m| m.format.as_str()), Some("session-log"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn session_log_then_cargo_json_keeps_both_sections() {
        let root = tmp_root("sc");
        ResultsData::parse_session_log(SESSION_FAIL, "s.json").unwrap().merge_into_sidecar(&root).unwrap();
        ResultsData::parse_cargo_json(CARGO_PASS, "cargo.json").merge_into_sidecar(&root).unwrap();
        let d = ResultsData::load_sidecar(&root).unwrap();
        assert_eq!(d.scenario_verdict("TC-SL-002", "A scenario"), FnVerdict::Fail, "by_scenario kept");
        assert_eq!(d.verdict_for("it_works"), FnVerdict::Pass);
        assert_eq!(d.format, "cargo-json");
        assert_eq!(d.scenario_meta.as_ref().map(|m| m.source.as_str()), Some("s.json"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn re_ingesting_the_same_kind_replaces_its_section() {
        let root = tmp_root("re");
        ResultsData::parse_cargo_json(CARGO_PASS, "a.json").merge_into_sidecar(&root).unwrap();
        ResultsData::parse_session_log(SESSION_FAIL, "s1.json").unwrap().merge_into_sidecar(&root).unwrap();
        // A second function-level run replaces by_leaf (no accumulation) …
        ResultsData::parse_junit(
            r#"<testsuite><testcase name="other_test"><failure/></testcase></testsuite>"#,
            "b.xml",
        )
        .merge_into_sidecar(&root)
        .unwrap();
        // … and a second session log replaces by_scenario.
        ResultsData::parse_session_log(SESSION_PASS, "s2.json").unwrap().merge_into_sidecar(&root).unwrap();
        let d = ResultsData::load_sidecar(&root).unwrap();
        assert_eq!(d.verdict_for("it_works"), FnVerdict::Missing, "earlier cargo-json leaf replaced");
        assert_eq!(d.verdict_for("other_test"), FnVerdict::Fail);
        assert_eq!(d.scenario_verdict("TC-SL-002", "A scenario"), FnVerdict::Missing, "earlier session replaced");
        assert_eq!(d.scenario_verdict("TC-SL-009", "Other"), FnVerdict::Pass);
        assert_eq!(d.leaf_meta.as_ref().map(|m| m.format.as_str()), Some("junit"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_or_unreadable_sidecar_is_treated_as_empty() {
        let root = tmp_root("bad");
        std::fs::create_dir_all(root.join(".syscribe")).unwrap();
        std::fs::write(ResultsData::sidecar_path(&root), "not json").unwrap();
        let (_, merged) = ResultsData::parse_cargo_json(CARGO_FAIL, "c.json").merge_into_sidecar(&root).unwrap();
        assert_eq!(merged.verdict_for("other_test"), FnVerdict::Fail);
        assert!(merged.by_scenario.is_empty() && merged.scenario_meta.is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_legacy_sidecar_without_section_metadata_still_merges() {
        // Written before `leaf_meta`/`scenario_meta` existed (schema 1.0 as-is).
        let legacy: ResultsData = serde_json::from_str(
            r#"{"schema_version":"1.0","format":"cargo-json","source":"old.json",
                "ingested_at_unix":1,"count":1,"by_leaf":{"it_works":"pass"}}"#,
        )
        .unwrap();
        let merged = ResultsData::parse_session_log(SESSION_FAIL, "s.json").unwrap().merged_over(Some(&legacy));
        assert_eq!(merged.verdict_for("it_works"), FnVerdict::Pass);
        assert_eq!(merged.leaf_meta.as_ref().map(|m| m.source.as_str()), Some("old.json"), "provenance recovered");
        assert_eq!(merged.schema_version, "1.0");
    }
}

// ---------------------------------------------------------------------------
// Run history (GH #258, REQ-TRS-RUNHIST-001)
// ---------------------------------------------------------------------------

/// One retained run: the verdicts of each section as last ingested under this run id.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunRecord {
    pub run: String,
    pub ingested_at_unix: u64,
    #[serde(default)]
    pub by_leaf: std::collections::BTreeMap<String, Verdict>,
    #[serde(default)]
    pub by_scenario: std::collections::BTreeMap<String, Verdict>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
}

/// `.syscribe/results-history.json`: retained runs, oldest first.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunHistory {
    #[serde(default)]
    pub runs: Vec<RunRecord>,
}

/// A test's verdict change between two runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestChange {
    pub test: String,
    /// Verdict in the first run (`None` = not present).
    pub from: Option<Verdict>,
    pub to: Option<Verdict>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDiff {
    pub regressions: Vec<TestChange>,
    pub fixed: Vec<TestChange>,
    pub still_failing: Vec<TestChange>,
    pub other_changes: Vec<TestChange>,
}

impl RunDiff {
    /// A failing test that vanished or was skipped (`fail` -> absent/ignored) — not a regression,
    /// but a gate must not let it through.
    pub fn vanished_failures(&self) -> Vec<&TestChange> {
        self.other_changes
            .iter()
            .filter(|c| c.from == Some(Verdict::Fail) && matches!(c.to, None | Some(Verdict::Ignored)))
            .collect()
    }
}

impl RunHistory {
    pub fn path(model_root: &Path) -> PathBuf {
        model_root.join(".syscribe").join("results-history.json")
    }

    /// Load the history; empty when absent. An unreadable file is an error rather than empty, so
    /// a corrupt history is never silently overwritten.
    pub fn load(model_root: &Path) -> Result<Self, String> {
        let p = Self::path(model_root);
        match std::fs::read_to_string(&p) {
            Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{} is not valid: {e}", p.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("cannot read {}: {e}", p.display())),
        }
    }

    pub fn save(&self, model_root: &Path) -> std::io::Result<PathBuf> {
        let p = Self::path(model_root);
        std::fs::create_dir_all(p.parent().expect("has parent"))?;
        // Write beside and rename, so a crash never leaves a truncated (unreadable) history.
        let tmp = p.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self).expect("history serialises"))?;
        std::fs::rename(&tmp, &p)?;
        Ok(p)
    }

    /// Record an ingest under `run`: replaces that run's section of the same kind, keeps the other
    /// section and other runs. Class-qualified `by_leaf` duplicates are not stored.
    pub fn record(&mut self, run: &str, data: &ResultsData) {
        let idx = match self.runs.iter().position(|r| r.run == run) {
            Some(i) => i,
            None => {
                self.runs.push(RunRecord { run: run.to_string(), ..Default::default() });
                self.runs.len() - 1
            }
        };
        let rec = &mut self.runs[idx];
        rec.ingested_at_unix = data.ingested_at_unix;
        match data.section() {
            Section::Leaf => {
                rec.by_leaf = data.by_leaf.iter().filter(|(k, _)| !k.contains("::")).map(|(k, v)| (k.clone(), *v)).collect();
            }
            Section::Scenario => {
                rec.by_scenario = data.by_scenario.iter().map(|(k, v)| (k.clone(), *v)).collect();
            }
        }
        if !data.source.is_empty() && !rec.sources.contains(&data.source) {
            rec.sources.push(data.source.clone());
        }
    }

    pub fn get(&self, run: &str) -> Option<&RunRecord> {
        self.runs.iter().find(|r| r.run == run)
    }
}

impl RunRecord {
    /// The verdicts of the sections both runs hold (a run that only ingested function results is
    /// not compared against scenarios the other run recorded).
    fn comparable(&self, other: &RunRecord) -> std::collections::BTreeMap<String, Verdict> {
        let mut m = std::collections::BTreeMap::new();
        if !self.by_leaf.is_empty() && !other.by_leaf.is_empty() {
            m.extend(self.by_leaf.iter().map(|(k, v)| (k.clone(), *v)));
        }
        if !self.by_scenario.is_empty() && !other.by_scenario.is_empty() {
            m.extend(self.by_scenario.iter().map(|(k, v)| (k.clone(), *v)));
        }
        m
    }

    /// Compare this run (`from`) with `to`, over the sections both runs hold.
    pub fn diff(&self, to: &RunRecord) -> RunDiff {
        let (a, b) = (self.comparable(to), to.comparable(self));
        let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
        keys.sort();
        keys.dedup();
        let mut d = RunDiff::default();
        for k in keys {
            let (from, to_v) = (a.get(k).copied(), b.get(k).copied());
            if from == to_v {
                if from == Some(Verdict::Fail) {
                    d.still_failing.push(TestChange { test: k.clone(), from, to: to_v });
                }
                continue;
            }
            let c = TestChange { test: k.clone(), from, to: to_v };
            match (from, to_v) {
                (_, Some(Verdict::Fail)) => d.regressions.push(c),
                (Some(Verdict::Fail), Some(Verdict::Pass)) => d.fixed.push(c),
                _ => d.other_changes.push(c),
            }
        }
        d
    }
}
