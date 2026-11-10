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
    /// Could not run, for instance the rig was unavailable (`records`, REQ-TRS-RESREC-001): not a pass.
    Blocked,
    /// Not executed (`records`): not a pass.
    #[serde(rename = "not-run")]
    NotRun,
    /// Ran but could not decide (`records`): not a pass.
    Inconclusive,
    /// Deliberately set aside with a recorded rationale (`records`): not a pass, but not a finding either.
    Waived,
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
    /// `records` and the verdicts derived from them (`records`, REQ-TRS-RESREC-001).
    Records,
}

/// Reduced, persisted view of a test run.
///
/// After a merge the top-level `format`/`source`/`ingested_at_unix`/`count`
/// describe the **latest** ingest (unchanged meaning for a single-format
/// sidecar); `leaf_meta`/`scenario_meta` record which ingest each section came
/// from. Both are optional, so a sidecar written before they existed still reads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Failure/skip/flaky evidence of the function-level section (GH #259), keyed like `by_leaf`
    /// (leaf and class-qualified key). Only non-passing testcases have an entry.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub details: HashMap<String, TestDetail>,
    /// Evidence recorded for one Configuration (`ingest-results --config`, GH #258, REQ-TRS-CFGRES-001),
    /// keyed by the Configuration's id. Each entry holds that configuration's own sections only (its
    /// `by_config` is empty); read it through [`ResultsData::for_config`].
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub by_config: std::collections::BTreeMap<String, ResultsData>,
    /// The normalised result records (`ingest-results --format records`, REQ-TRS-RESREC-001), latest per
    /// (test case, scenario, environment).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<crate::result_records::ResultRecord>,
    /// The verdicts derived from `records`, keyed like `by_scenario` (`"{testCase}::{scenario}"`, and
    /// `"{testCase}::*"` for a whole-TestCase record); the worst over the environments of a key. Kept apart from
    /// `by_scenario` so a session-log ingest, which replaces that map, cannot lose them and the reverse.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub by_record: HashMap<String, Verdict>,
    /// Records of the last records ingest whose `testCase` names no element of the model (REQ-TRS-RESJOIN-001):
    /// kept so they can be listed, never counted as evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched: Vec<crate::result_records::ResultRecord>,
    /// Provenance of `records` (the last records ingest).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub records_meta: Option<SectionMeta>,
    /// Set on a freshly parsed records ingest by `--replace`: drop the records it does not contain.
    #[serde(skip)]
    pub replace_records: bool,
}

/// Collapse whitespace and drop control characters (ESC, BEL, C1, …) of text taken from a test
/// report before it is stored or printed.
pub fn one_line(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() || c.is_whitespace()).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Longest retained JUnit `message` (characters).
const MAX_DETAIL_MESSAGE: usize = 500;

/// Retained evidence of one non-passing JUnit testcase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestDetail {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Testcase `time` in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<f64>,
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

/// Retained failure evidence of one failing `testFunctions` function.
#[derive(Debug, Clone, PartialEq)]
pub struct FailureNote {
    pub function: String,
    pub message: Option<String>,
    pub time: Option<f64>,
}

/// The failing functions of a TestCase with their retained JUnit message and time (GH #258):
/// only functions whose verdict is `fail` and for which a message or time was kept.
pub fn failure_notes(tc: &crate::element::RawElement, results: Option<&ResultsData>) -> Vec<FailureNote> {
    let Some(results) = results else { return Vec::new() };
    tc_function_refs(tc)
        .into_iter()
        .filter(|f| results.verdict_for(f) == FnVerdict::Fail)
        .filter_map(|f| {
            let d = results.detail_for(&f)?;
            Some(FailureNote { function: f, message: d.message.clone(), time: d.time })
        })
        .collect()
}

/// The exact `Scenario:`/`Scenario Outline:` titles declared in a TestCase's
/// body, in document order: the identity a `session-log` record names.
pub fn gherkin_scenario_titles(doc: &str) -> Vec<String> {
    doc.lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("Scenario Outline:")
                .or_else(|| t.strip_prefix("Scenario:"))
                .map(|title| title.trim().to_string())
        })
        .collect()
}

/// Pass only when everything passed; `waived` when everything passed or was waived and something was waived (a
/// waiver is never shown as a pass); otherwise unknown.
fn rolled_up(all_pass_or_waived: bool, waived: bool) -> crate::safety_case::Verdict {
    use crate::safety_case::Verdict as V;
    match (all_pass_or_waived, waived) {
        (true, false) => V::Pass,
        (true, true) => V::Waived,
        _ => V::Unknown,
    }
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
        let mut waived = false;
        for f in &funcs {
            match results.verdict_for(f) {
                FnVerdict::Fail => return V::Fail,
                FnVerdict::Pass => {}
                FnVerdict::Waived => waived = true,
                FnVerdict::Ignored | FnVerdict::Missing | FnVerdict::Flaky => all_pass = false,
            }
        }
        return rolled_up(all_pass, waived);
    }
    let Some(tc_id) = tc.frontmatter.id.as_deref() else { return V::Unknown };
    let scenarios = gherkin_scenario_titles(&tc.doc);
    if scenarios.is_empty() {
        // Nothing to score scenario by scenario: only a record of the whole TestCase can speak.
        return match results.testcase_record_verdict(tc_id) {
            Some(FnVerdict::Fail) => V::Fail,
            Some(FnVerdict::Pass) => V::Pass,
            Some(FnVerdict::Waived) => V::Waived,
            _ => V::Unknown,
        };
    }
    let mut all_pass = true;
    let mut waived = false;
    for s in &scenarios {
        match results.scenario_verdict(tc_id, s) {
            FnVerdict::Fail => return V::Fail,
            FnVerdict::Pass => {}
            FnVerdict::Waived => waived = true,
            FnVerdict::Ignored | FnVerdict::Missing | FnVerdict::Flaky => all_pass = false,
        }
    }
    rolled_up(all_pass, waived)
}

/// A TestCase's verdict when it must hold on **every** one of `envs` (REQ-TRS-RESENV-001): each environment is
/// judged on its own records, and the worst decides: a failure anywhere fails, an environment with no passing
/// result leaves it unknown, and a waiver is shown as a waiver. `unhealthy` environments contribute no result.
pub fn testcase_verdict_in_envs(
    tc: &crate::element::RawElement,
    results: &ResultsData,
    envs: &[String],
    unhealthy: &std::collections::HashSet<String>,
) -> crate::safety_case::Verdict {
    use crate::safety_case::Verdict as V;
    let rank = |v: V| match v {
        V::Fail => 3,
        V::Unknown => 2,
        V::Waived => 1,
        V::Pass => 0,
    };
    envs.iter()
        .map(|env| testcase_verdict(tc, Some(&results.view_for_env(env, unhealthy))))
        .reduce(|a, b| if rank(b) > rank(a) { b } else { a })
        .unwrap_or_else(|| testcase_verdict(tc, Some(results)))
}

/// The lookup outcome a stored verdict stands for. `blocked`, `not-run` and `inconclusive` are not a pass and
/// are treated as skipped (`Ignored`) by every reader that already handles that.
pub fn fn_verdict(v: Verdict) -> FnVerdict {
    match v {
        Verdict::Pass => FnVerdict::Pass,
        Verdict::Fail => FnVerdict::Fail,
        Verdict::Flaky => FnVerdict::Flaky,
        Verdict::Waived => FnVerdict::Waived,
        Verdict::Ignored | Verdict::Blocked | Verdict::NotRun | Verdict::Inconclusive => FnVerdict::Ignored,
    }
}

/// Lookup outcome for one `testFunctions[].function`.
#[derive(Debug, PartialEq, Eq)]
pub enum FnVerdict {
    Pass,
    Fail,
    Ignored,
    /// Set aside on purpose, with a rationale on record (`records`): not a pass, not a finding.
    Waived,
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
        match self.format.as_str() {
            "session-log" => Section::Scenario,
            "records" => Section::Records,
            _ => Section::Leaf,
        }
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
        if let Some(old) = existing {
            out.by_config = old.by_config.clone();
        }
        let own = Some(self.meta());
        match self.section() {
            Section::Leaf => {
                out.leaf_meta = own;
                if let Some(old) = existing {
                    out.by_scenario = old.by_scenario.clone();
                    out.scenario_meta = old.scenario_meta.clone().or_else(|| legacy_meta(old, Section::Scenario));
                    out.records = old.records.clone();
                    out.by_record = old.by_record.clone();
                    out.records_meta = old.records_meta.clone();
                    out.unmatched = old.unmatched.clone();
                }
            }
            Section::Scenario => {
                out.scenario_meta = own;
                if let Some(old) = existing {
                    out.by_leaf = old.by_leaf.clone();
                    out.details = old.details.clone();
                    out.leaf_meta = old.leaf_meta.clone().or_else(|| legacy_meta(old, Section::Leaf));
                    out.records = old.records.clone();
                    out.by_record = old.by_record.clone();
                    out.records_meta = old.records_meta.clone();
                    out.unmatched = old.unmatched.clone();
                }
            }
            Section::Records => {
                out.records_meta = own;
                if let Some(old) = existing {
                    out.by_leaf = old.by_leaf.clone();
                    out.details = old.details.clone();
                    out.leaf_meta = old.leaf_meta.clone().or_else(|| legacy_meta(old, Section::Leaf));
                    out.by_scenario = old.by_scenario.clone();
                    out.scenario_meta = old.scenario_meta.clone().or_else(|| legacy_meta(old, Section::Scenario));
                    if !self.replace_records {
                        // Latest per (test case, scenario, environment): this ingest supersedes the records with the
                        // same key and keeps the others, so one rig's ingest does not erase another's.
                        let own_keys: std::collections::HashSet<_> = self.records.iter().map(|r| r.key()).collect();
                        let mut kept: Vec<_> = old.records.iter().filter(|r| !own_keys.contains(&r.key())).cloned().collect();
                        kept.extend(self.records.iter().cloned());
                        out.records = kept;
                    }
                }
                out.by_record = derive_by_record(&out.records);
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

    /// Merge this ingest into the sidecar as the evidence of Configuration `config`
    /// (REQ-TRS-CFGRES-001): that configuration's own section of this ingest's kind is replaced, its
    /// other section, every other configuration and the global sections stay. With no sidecar yet the
    /// global sections start empty.
    pub fn merge_config_into_sidecar(&self, model_root: &Path, config: &str) -> std::io::Result<(PathBuf, ResultsData)> {
        let existing = Self::load_sidecar(model_root);
        let mut out = existing.clone().unwrap_or_else(|| ResultsData {
            // No global ingest has happened: say so rather than borrowing this ingest's format and source.
            format: "per-configuration".to_string(),
            source: String::new(),
            by_leaf: HashMap::new(),
            by_scenario: HashMap::new(),
            details: HashMap::new(),
            leaf_meta: None,
            scenario_meta: None,
            count: 0,
            by_config: Default::default(),
            records: Vec::new(),
            by_record: HashMap::new(),
            records_meta: None,
            unmatched: Vec::new(),
            ..self.clone()
        });
        let mut own = self.clone();
        own.by_config = Default::default();
        let merged = own.merged_over(out.by_config.get(config));
        out.by_config.insert(config.to_string(), merged);
        let path = out.write_sidecar(model_root)?;
        Ok((path, out))
    }

    /// The results as Configuration `config` sees them: the global sections overlaid with that
    /// configuration's own — per function, scenario and detail its own entry wins, otherwise the
    /// global one applies. No configuration inherits another's evidence.
    pub fn for_config(&self, config: &str) -> ResultsData {
        let mut out = self.clone();
        out.by_config = Default::default();
        if let Some(c) = self.by_config.get(config) {
            out.by_leaf.extend(c.by_leaf.iter().map(|(k, v)| (k.clone(), *v)));
            out.by_scenario.extend(c.by_scenario.iter().map(|(k, v)| (k.clone(), *v)));
            out.details.extend(c.details.iter().map(|(k, v)| (k.clone(), v.clone())));
            // Records: the configuration's own supersede the global ones with the same key.
            if !c.records.is_empty() {
                let own: std::collections::HashSet<_> = c.records.iter().map(|r| r.key()).collect();
                out.records.retain(|r| !own.contains(&r.key()));
                out.records.extend(c.records.iter().cloned());
            }
            out.by_record.extend(c.by_record.iter().map(|(k, v)| (k.clone(), *v)));
            // A configuration that reports a function has no use for the global failure detail of it,
            // and a bare leaf it reports supersedes the global class-qualified entries of that leaf
            // (`verdict_for` tries the qualified key first, which would otherwise mask the override).
            for k in c.by_leaf.keys() {
                if !c.details.contains_key(k) {
                    out.details.remove(k);
                }
                if !k.contains("::") {
                    let suffix = format!("::{k}");
                    let masked: Vec<String> = out.by_leaf.keys().filter(|g| g.ends_with(&suffix) && !c.by_leaf.contains_key(*g)).cloned().collect();
                    for g in masked {
                        out.by_leaf.remove(&g);
                        out.details.remove(&g);
                    }
                }
            }
        }
        out
    }

    /// Persist this data to the sidecar, creating `.syscribe/` as needed.
    pub fn write_sidecar(&self, model_root: &Path) -> std::io::Result<PathBuf> {
        let dir = model_root.join(".syscribe");
        crate::fsio::create_dir_all(&dir)?;
        let path = dir.join("results.json");
        let json = serde_json::to_string_pretty(self).expect("ResultsData serialises");
        crate::fsio::write(&path, json)?;
        Ok(path)
    }

    /// Worst-wins ordering of lookup outcomes: Fail > Flaky > Pass > Ignored > Missing.
    fn fn_rank(v: &FnVerdict) -> u8 {
        match v {
            FnVerdict::Fail => 4,
            FnVerdict::Flaky => 3,
            FnVerdict::Pass => 2,
            FnVerdict::Ignored | FnVerdict::Waived => 1,
            FnVerdict::Missing => 0,
        }
    }

    fn worse(a: FnVerdict, b: FnVerdict) -> FnVerdict {
        if Self::fn_rank(&b) > Self::fn_rank(&a) { b } else { a }
    }

    /// Verdict for a `testFunctions[].function` reference. With per-Configuration evidence present
    /// (`by_config`, GH #258) and no configuration chosen this is the worst outcome over the global
    /// sections and every configuration, so a failure on any variant is never hidden; a view taken
    /// with [`Self::for_config`] has no `by_config` and answers for that one configuration.
    pub fn verdict_for(&self, function_ref: &str) -> FnVerdict {
        let mut v = self.verdict_here(function_ref);
        for c in self.by_config.values() {
            v = Self::worse(v, c.verdict_here(function_ref));
        }
        v
    }

    /// [`Self::verdict_for`] over this layer's own sections only.
    fn verdict_here(&self, function_ref: &str) -> FnVerdict {
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
            Some(v) => fn_verdict(*v),
            None => FnVerdict::Missing,
        }
    }

    /// The retained detail (JUnit message / time) of a `testFunctions[].function` reference, looked
    /// up like [`Self::verdict_for`]: the class-qualified key first, then the leaf.
    pub fn detail_for(&self, function_ref: &str) -> Option<&TestDetail> {
        self.detail_here(function_ref).or_else(|| self.by_config.values().find_map(|c| c.detail_here(function_ref)))
    }

    fn detail_here(&self, function_ref: &str) -> Option<&TestDetail> {
        let mut qualified = function_ref.replace('#', "::");
        if !qualified.contains("::") {
            if let Some((class, name)) = qualified.rsplit_once('.') {
                qualified = format!("{class}::{name}");
            }
        }
        // A reference the run knows by its class-qualified key owns that key's detail (possibly none);
        // the leaf entry may belong to another class's test of the same name.
        if qualified.contains("::") && self.by_leaf.contains_key(&qualified) {
            return self.details.get(&qualified);
        }
        self.details.get(function_leaf(function_ref))
    }

    /// Verdict for one (`TestCase`, Gherkin scenario) pair, as recorded by a
    /// `session-log` ingestion (issue #113).
    pub fn scenario_verdict(&self, tc_id: &str, scenario: &str) -> FnVerdict {
        let mut v = self.scenario_here(tc_id, scenario);
        for c in self.by_config.values() {
            v = Self::worse(v, c.scenario_here(tc_id, scenario));
        }
        v
    }

    fn scenario_here(&self, tc_id: &str, scenario: &str) -> FnVerdict {
        // A session-log verdict, a record of this scenario and a record of the whole TestCase all speak about it:
        // the worst one decides.
        let key = format!("{tc_id}::{scenario}");
        let whole = format!("{tc_id}::*");
        [self.by_scenario.get(&key), self.by_record.get(&key), self.by_record.get(&whole)]
            .into_iter()
            .flatten()
            .copied()
            .reduce(crate::result_records::worst)
            .map_or(FnVerdict::Missing, fn_verdict)
    }

    /// The verdict a `records` ingest gave the whole TestCase (a record without a scenario), if any.
    pub fn testcase_record_verdict(&self, tc_id: &str) -> Option<FnVerdict> {
        let key = format!("{tc_id}::*");
        let mut v = self.by_record.get(&key).copied();
        for c in self.by_config.values() {
            if let Some(w) = c.by_record.get(&key) {
                v = Some(v.map_or(*w, |x| crate::result_records::worst(x, *w)));
            }
        }
        v.map(fn_verdict)
    }

    /// Worst-wins ordering used when two results share a key: Fail > Flaky > Pass > Ignored.
    fn rank(v: Verdict) -> u8 {
        match v {
            Verdict::Fail => 3,
            Verdict::Flaky => 2,
            Verdict::Pass | Verdict::Waived => 1,
            Verdict::Ignored | Verdict::Blocked | Verdict::NotRun | Verdict::Inconclusive => 0,
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
        let mut details: HashMap<String, TestDetail> = HashMap::new();
        // The open testcase's `time` and the first `message` of a failure/skip/retry child.
        let mut time: Option<f64> = None;
        let mut message: Option<String> = None;
        // The open testcase's classname and whether it carried a retry marker.
        let mut class: String = String::new();
        let mut retried = false;
        let mut buf = Vec::new();

        let extract = |e: &quick_xml::events::BytesStart, key: &[u8]| -> Option<String> {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == key {
                    return Some(match attr.normalized_value(quick_xml::XmlVersion::Implicit1_0) {
                        Ok(v) => v.into_owned(),
                        Err(_) => String::from_utf8_lossy(&attr.value).into_owned(),
                    });
                }
            }
            None
        };
        // Record a finished testcase: leaf key and, when a classname is known, the
        // qualified `classname::name` key (GH #259). A retry marker on an otherwise
        // passing testcase makes it flaky.
        let finish_case = |by_leaf: &mut HashMap<String, Verdict>,
                           details: &mut HashMap<String, TestDetail>,
                           name: &str,
                           class: &str,
                           verdict: Verdict,
                           retried: bool,
                           time: Option<f64>,
                           message: Option<String>| {
            let verdict = if retried && verdict == Verdict::Pass { Verdict::Flaky } else { verdict };
            Self::record(by_leaf, name, verdict);
            let qualified = (!class.is_empty()).then(|| format!("{class}::{}", function_leaf(name)));
            if let Some(q) = &qualified {
                Self::record_key(by_leaf, q, verdict);
            }
            if verdict != Verdict::Pass {
                // Cap runaway messages (assertion diffs, stack traces) at a char boundary, and drop
                // terminal control characters (a test message must not drive the terminal).
                let message = message.map(|m| one_line(&m)).filter(|m| !m.is_empty());
                let message = message.map(|m| {
                    if m.chars().count() > MAX_DETAIL_MESSAGE {
                        let cut: String = m.chars().take(MAX_DETAIL_MESSAGE).collect();
                        format!("{cut}…")
                    } else {
                        m
                    }
                });
                let d = TestDetail { message, time };
                for key in std::iter::once(function_leaf(name).to_string()).chain(qualified) {
                    // The case whose verdict won the leaf owns its detail (Fail beats Flaky beats Ignored);
                    // a winner with nothing to keep clears a weaker case's detail rather than inheriting it.
                    if by_leaf.get(&key) == Some(&verdict) {
                        if d.message.is_some() || d.time.is_some() {
                            details.insert(key, d.clone());
                        } else {
                            details.remove(&key);
                        }
                    }
                }
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
                                finish_case(&mut by_leaf, &mut details, &name, &class, verdict, retried, time.take(), message.take());
                                count += 1;
                            }
                            let name = extract(e, b"name").unwrap_or_default();
                            class = extract(e, b"classname").unwrap_or_default();
                            time = extract(e, b"time").and_then(|t| t.trim().parse::<f64>().ok());
                            message = None;
                            retried = false;
                            // A self-closing <testcase .../> (Event::Empty) with no
                            // failure/error child is a pass; it is flushed when the
                            // next testcase opens or at EOF.
                            current = Some((name, Verdict::Pass));
                        }
                        b"flakyFailure" | b"flakyError" | b"rerunFailure" | b"rerunError" => {
                            retried = true;
                            if message.is_none() {
                                message = extract(e, b"message");
                            }
                        }
                        b"failure" | b"error" => {
                            if let Some((_, v)) = current.as_mut() {
                                *v = Verdict::Fail;
                            }
                            // A failure's message outranks a retry/skip message already seen.
                            if let Some(m) = extract(e, b"message") {
                                message = Some(m);
                            }
                        }
                        b"skipped" => {
                            if let Some((_, v)) = current.as_mut() {
                                if *v != Verdict::Fail {
                                    *v = Verdict::Ignored;
                                }
                            }
                            if message.is_none() {
                                message = extract(e, b"message");
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"testcase" {
                        if let Some((name, verdict)) = current.take() {
                            if !name.is_empty() {
                                finish_case(&mut by_leaf, &mut details, &name, &class, verdict, retried, time.take(), message.take());
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
                finish_case(&mut by_leaf, &mut details, &name, &class, verdict, retried, time.take(), message.take());
                count += 1;
            }
        }
        // A later passing duplicate wins the leaf: drop the stale detail of the earlier non-pass.
        details.retain(|k, _| by_leaf.get(k) != Some(&Verdict::Pass));
        let mut data = Self::finish(by_leaf, count, "junit", source);
        data.details = details;
        data
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

    /// Parse a normalised-record file (`--format records`, REQ-TRS-RESREC-001). Strict like `session-log`: a
    /// malformed record is a hard error naming it, and nothing is stored.
    pub fn parse_records(text: &str, source: &str) -> Result<Self, String> {
        let parsed = crate::result_records::parse(text)?;
        // `count` is what the file held; `records` keeps the last of any repeated key.
        let count = parsed.len();
        let records = crate::result_records::keep_last_per_key(parsed);
        let mut data = Self::finish(HashMap::new(), count, "records", source);
        data.by_record = derive_by_record(&records);
        data.records = records;
        Ok(data)
    }

    /// Join the records to the TestCases of the model (REQ-TRS-RESJOIN-001): each record gets its TestCase's
    /// canonical id and the hashes that let a later edit show; records naming nothing move to `unmatched`. An
    /// ambiguous key, or one naming something that is not a TestCase, is an error.
    pub fn join_test_cases(&mut self, elems: &[crate::element::RawElement]) -> Result<(), String> {
        let joined = crate::result_join::join(std::mem::take(&mut self.records), elems)?;
        // the same test named by id in one record and by extRef in another is one key now
        self.records = crate::result_records::keep_last_per_key(joined.matched);
        self.unmatched = joined.unmatched;
        self.by_record = derive_by_record(&self.records);
        Ok(())
    }

    /// True when any record is stored, globally or for a Configuration.
    pub fn has_records(&self) -> bool {
        !self.records.is_empty() || self.by_config.values().any(|c| !c.records.is_empty())
    }

    /// These results as the TestEnvironment `env` alone shows them (REQ-TRS-RESENV-001): only the records made on
    /// that rig count, and none at all when the rig is `unhealthy` (retired or out of calibration). Function-level
    /// and session-log verdicts carry no environment and are untouched. Apply it to the Configuration's effective
    /// results, so the Configuration is chosen first and the environment filters inside it.
    pub fn view_for_env(&self, env: &str, unhealthy: &std::collections::HashSet<String>) -> ResultsData {
        let mut out = self.clone();
        let usable = !unhealthy.contains(env);
        let on_env = |r: &crate::result_records::ResultRecord| usable && r.provenance.as_ref().and_then(|p| p.environment.as_deref()) == Some(env);
        out.records.retain(|r| on_env(r));
        out.by_record = derive_by_record(&out.records);
        for c in out.by_config.values_mut() {
            c.records.retain(|r| on_env(r));
            c.by_record = derive_by_record(&c.records);
        }
        out
    }

    /// Every stored record about `tc_id`, in the global section and in each Configuration's.
    pub fn records_for<'a>(&'a self, tc_id: &'a str) -> impl Iterator<Item = &'a crate::result_records::ResultRecord> + 'a {
        self.records.iter().chain(self.by_config.values().flat_map(|c| c.records.iter())).filter(move |r| r.test_case == tc_id)
    }

    /// A records ingest built from records a built-in adapter produced (not parsed from a records file). `count` is
    /// what the report held (tests, for Robot), which may be more than the records it folds into.
    pub fn from_records(records: Vec<crate::result_records::ResultRecord>, count: usize, source: &str) -> Self {
        let mut data = Self::finish(HashMap::new(), count, "records", source);
        data.records = records;
        data
    }

    /// Replace each record's `environment` with the id of the `TestEnvironment` it names (an id or a qualified
    /// name); an environment the model does not have is an error, so a typo cannot hide a rig.
    pub fn resolve_environments(&mut self, elems: &[crate::element::RawElement]) -> Result<(), String> {
        use crate::element::ElementType;
        for r in &mut self.records {
            let Some(env) = r.provenance.as_mut().and_then(|p| p.environment.as_mut()) else { continue };
            let found = elems.iter().find(|e| {
                e.frontmatter.element_type == Some(ElementType::TestEnvironment)
                    && (e.frontmatter.id.as_deref() == Some(env.as_str()) || e.qualified_name == *env)
            });
            match found {
                Some(e) => *env = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone()),
                None => {
                    return Err(format!("records: record ('{}'): environment '{env}' is not a TestEnvironment of this model", r.test_case));
                }
            }
        }
        // The same rig named by id in one record and by qualified name in another is one key now.
        self.records = crate::result_records::keep_last_per_key(std::mem::take(&mut self.records));
        self.by_record = derive_by_record(&self.records);
        Ok(())
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
            details: HashMap::new(),
            by_config: Default::default(),
            ..Default::default()
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
        Section::Records => !old.records.is_empty(),
    };
    (non_empty && old.section() == section).then(|| old.meta())
}

/// The verdicts derived from `records`: one per `"{testCase}::{scenario}"` key (`"{testCase}::*"` for a
/// whole-TestCase record), the worst over the environments the key was recorded on.
pub fn derive_by_record(records: &[crate::result_records::ResultRecord]) -> HashMap<String, Verdict> {
    let mut out: HashMap<String, Verdict> = HashMap::new();
    for r in records {
        let key = format!("{}::{}", r.test_case, r.scenario.as_deref().unwrap_or("*"));
        out.entry(key).and_modify(|v| *v = crate::result_records::worst(*v, r.verdict)).or_insert(r.verdict);
    }
    out
}

/// Add `from` to `into`, keeping the worse verdict where a key is in both.
fn merge_worst(into: &mut std::collections::BTreeMap<String, Verdict>, from: &HashMap<String, Verdict>) {
    for (k, v) in from {
        into.entry(k.clone()).and_modify(|old| *old = crate::result_records::worst(*old, *v)).or_insert(*v);
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
    /// The per-Configuration sections of this run (`ingest-results --run … --config …`).
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub by_config: std::collections::BTreeMap<String, ConfigSections>,
}

/// One Configuration's sections of a retained run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSections {
    #[serde(default)]
    pub by_leaf: std::collections::BTreeMap<String, Verdict>,
    #[serde(default)]
    pub by_scenario: std::collections::BTreeMap<String, Verdict>,
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
        // Through the write layer, so `--dry-run` and the agent policy's `protected_paths` apply to the history as
        // they do to the sidecar. Written beside and renamed, so a crash never leaves a truncated (unreadable) one.
        crate::fsio::create_dir_all(p.parent().expect("has parent"))?;
        crate::fsio::write_atomic(&p, serde_json::to_string_pretty(self).expect("history serialises"))?;
        Ok(p)
    }

    /// Record an ingest under `run`: replaces that run's section of the same kind, keeps the other
    /// section and other runs. Class-qualified `by_leaf` duplicates are not stored.
    pub fn record(&mut self, run: &str, data: &ResultsData) {
        self.record_for(run, data, None);
    }

    /// [`Self::record`] for the evidence of one Configuration (`Some`) or the global sections (`None`).
    pub fn record_for(&mut self, run: &str, data: &ResultsData, config: Option<&str>) {
        let idx = match self.runs.iter().position(|r| r.run == run) {
            Some(i) => i,
            None => {
                self.runs.push(RunRecord { run: run.to_string(), ..Default::default() });
                self.runs.len() - 1
            }
        };
        let rec = &mut self.runs[idx];
        rec.ingested_at_unix = data.ingested_at_unix;
        let leaves = || data.by_leaf.iter().filter(|(k, _)| !k.contains("::")).map(|(k, v)| (k.clone(), *v)).collect();
        let scenarios = || data.by_scenario.iter().map(|(k, v)| (k.clone(), *v)).collect();
        match (config, data.section()) {
            (None, Section::Leaf) => rec.by_leaf = leaves(),
            (None, Section::Scenario) => rec.by_scenario = scenarios(),
            // records: the derived verdicts join the run's scenario map, worst of the two when a key is shared
            (None, Section::Records) => merge_worst(&mut rec.by_scenario, &data.by_record),
            (Some(c), Section::Records) => merge_worst(&mut rec.by_config.entry(c.to_string()).or_default().by_scenario, &data.by_record),
            (Some(c), Section::Leaf) => rec.by_config.entry(c.to_string()).or_default().by_leaf = leaves(),
            (Some(c), Section::Scenario) => rec.by_config.entry(c.to_string()).or_default().by_scenario = scenarios(),
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
    /// This retained run as a results sidecar value, for `--results-as-of` (GH #258). Class-qualified
    /// keys are not retained, so a qualified reference resolves through its leaf name.
    pub fn to_results(&self) -> ResultsData {
        ResultsData {
            schema_version: "1.0".to_string(),
            format: "history".to_string(),
            source: format!("run {}", self.run),
            ingested_at_unix: self.ingested_at_unix,
            count: self.by_leaf.len() + self.by_scenario.len(),
            by_leaf: self.by_leaf.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            by_scenario: self.by_scenario.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            leaf_meta: None,
            scenario_meta: None,
            details: HashMap::new(),
            records: Vec::new(),
            by_record: HashMap::new(),
            records_meta: None,
            replace_records: false,
            unmatched: Vec::new(),
            by_config: self
                .by_config
                .iter()
                .map(|(c, sec)| {
                    let mut r = ResultsData {
                        schema_version: "1.0".to_string(),
                        format: "history".to_string(),
                        source: format!("run {}", self.run),
                        ingested_at_unix: self.ingested_at_unix,
                        count: sec.by_leaf.len() + sec.by_scenario.len(),
                        by_leaf: sec.by_leaf.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                        by_scenario: sec.by_scenario.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                        leaf_meta: None,
                        scenario_meta: None,
                        details: HashMap::new(),
                        by_config: Default::default(),
                        records: Vec::new(),
                        by_record: HashMap::new(),
                        records_meta: None,
                        replace_records: false,
                        unmatched: Vec::new(),
                    };
                    r.count = r.by_leaf.len() + r.by_scenario.len();
                    (c.clone(), r)
                })
                .collect(),
        }
    }

    /// This run's results as Configuration `c` sees them: its own sections over the global ones.
    fn effective(&self, c: &str) -> (std::collections::BTreeMap<String, Verdict>, std::collections::BTreeMap<String, Verdict>) {
        let (mut leaf, mut scn) = (self.by_leaf.clone(), self.by_scenario.clone());
        if let Some(sec) = self.by_config.get(c) {
            leaf.extend(sec.by_leaf.iter().map(|(k, v)| (k.clone(), *v)));
            scn.extend(sec.by_scenario.iter().map(|(k, v)| (k.clone(), *v)));
        }
        (leaf, scn)
    }

    /// The verdicts of the sections both runs hold (a run that only ingested function results is
    /// not compared against scenarios the other run recorded), plus, for every Configuration either
    /// run holds evidence for, the *effective* results of both runs under `<test> @ <config>` — a
    /// run without its own section for that configuration contributes its global results, so a
    /// failure that appears only under one configuration is a regression, not an invisible change.
    fn comparable(&self, other: &RunRecord) -> std::collections::BTreeMap<String, Verdict> {
        type Sec = std::collections::BTreeMap<String, Verdict>;
        fn sections(m: &mut Sec, (a_leaf, a_scn): (&Sec, &Sec), (b_leaf, b_scn): (&Sec, &Sec), suffix: &str) {
            if !a_leaf.is_empty() && !b_leaf.is_empty() {
                m.extend(a_leaf.iter().map(|(k, v)| (format!("{k}{suffix}"), *v)));
            }
            if !a_scn.is_empty() && !b_scn.is_empty() {
                m.extend(a_scn.iter().map(|(k, v)| (format!("{k}{suffix}"), *v)));
            }
        }
        let mut m = Sec::new();
        sections(&mut m, (&self.by_leaf, &self.by_scenario), (&other.by_leaf, &other.by_scenario), "");
        let mut configs: Vec<&String> = self.by_config.keys().chain(other.by_config.keys()).collect();
        configs.sort();
        configs.dedup();
        for c in configs {
            let (mine, theirs) = (self.effective(c), other.effective(c));
            sections(&mut m, (&mine.0, &mine.1), (&theirs.0, &theirs.1), &format!(" @ {c}"));
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
