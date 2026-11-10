//! The run history (GH #258, `REQ-TRS-RUNHIST-001`): `ingest-results --run <id>` retains each ingest under a run
//! identity so runs can be listed and compared (`results runs`, `results diff`).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::result_records::ResultRecord;
use crate::results::{ResultsData, Section, Verdict};

/// Add `from` to `into`, keeping the worse verdict where a key is in both.
fn merge_worst(into: &mut std::collections::BTreeMap<String, Verdict>, from: &HashMap<String, Verdict>) {
    for (k, v) in from {
        into.entry(k.clone()).and_modify(|old| *old = crate::result_records::worst(*old, *v)).or_insert(*v);
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
    /// How many records the run's file under `runs/` holds (see [`RunRecords`]); `0` when it keeps none.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub record_count: usize,
    /// The run's records were dropped by `[results] keep_run_records` ([`RunHistory::plan`]); its verdict maps remain,
    /// so `results diff` still compares it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub records_dropped: bool,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
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
    /// Records an ingest staged for the run files, written by [`RunHistory::plan`]'s commit.
    #[serde(skip)]
    pending: Vec<Pending>,
}

/// An ingest's records for one run, staged until the plan is made (GH #471).
#[derive(Debug, Clone)]
struct Pending {
    run: String,
    config: Option<String>,
    records: Vec<ResultRecord>,
    replace: bool,
}

/// The records of one retained run: `<runs dir>/<run>.json`, kept apart from the index so an ingest rewrites only its
/// own run, the history file stays small, and dropping a run's records is deleting a file. Records of a records-like
/// ingest (`records`, `robot`, `cucumber`, a results plugin) are kept slimmed ([`slim_record`]), joined to the model's
/// TestCases and stamped with the hashes of that ingest, globally or per Configuration (`--config`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunRecords {
    pub run: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<ResultRecord>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_config: BTreeMap<String, Vec<ResultRecord>>,
}

impl RunRecords {
    pub fn total(&self) -> usize {
        self.records.len() + self.by_config.values().map(Vec::len).sum::<usize>()
    }

    /// Retain an ingest's records the way the sidecar does: an ingest supersedes the records with its own keys (test
    /// case, scenario, environment) and keeps the others, or replaces them all under `--replace`.
    fn apply(&mut self, p: &Pending) {
        let into = match &p.config {
            None => &mut self.records,
            Some(c) => self.by_config.entry(c.clone()).or_default(),
        };
        if p.replace {
            into.clear();
        } else {
            let keys: HashSet<_> = p.records.iter().map(|r| r.key()).collect();
            into.retain(|r| !keys.contains(&r.key()));
        }
        into.extend(p.records.iter().cloned());
        if let Some(c) = &p.config {
            if self.by_config.get(c).is_some_and(Vec::is_empty) {
                self.by_config.remove(c);
            }
        }
    }
}

/// A retained run read for `trade-study --run`: its index entry and its records file.
#[derive(Debug, Clone)]
pub struct LoadedRun {
    pub index: RunRecord,
    pub records: RunRecords,
    /// Where the run's records file is (or should be).
    pub file: PathBuf,
    /// The index says the run holds records but its file is not there.
    pub file_missing: bool,
}

/// A record as the history keeps it: what ranking and the stale check use — the key, the verdict, the measurements,
/// the hashes it was stamped with and when it finished — without the opaque `steps`, the artefact references and the
/// operator, tool and build. The sidecar keeps the full record; a run file keeps this, so it stays small.
pub fn slim_record(r: &ResultRecord) -> ResultRecord {
    use crate::result_records::Provenance;
    let provenance = r
        .provenance
        .as_ref()
        .map(|p| Provenance { environment: p.environment.clone(), started_at: p.started_at.clone(), finished_at: p.finished_at.clone(), ..Default::default() })
        .filter(|p| p.environment.is_some() || p.started_at.is_some() || p.finished_at.is_some());
    ResultRecord {
        test_case: r.test_case.clone(),
        scenario: r.scenario.clone(),
        verdict: r.verdict,
        rationale: r.rationale.clone(),
        steps: Vec::new(),
        measurements: r.measurements.clone(),
        provenance,
        tc_hash: r.tc_hash.clone(),
        req_hashes: r.req_hashes.clone(),
    }
}

/// How many runs keep their records when `.syscribe.toml` says nothing.
pub const DEFAULT_KEEP_RUN_RECORDS: usize = 20;

/// The `[results]` table of `<model_root>/.syscribe.toml` (GH #471): `keep_run_records`, how many of the most recently
/// ingested runs keep their records (`0`: none), and `runs_dir`, where the run files live. A malformed entry is ignored,
/// listed in `problems`, and the default applies.
#[derive(Debug, Clone)]
pub struct ResultsSettings {
    pub keep_run_records: usize,
    /// Where the run files live: `<model>/.syscribe/runs` unless `runs_dir` says otherwise.
    pub runs_dir: PathBuf,
    /// `runs_dir` was set (and valid), so it may be outside `.syscribe/` and the agent policy must know it.
    pub runs_dir_custom: bool,
    pub problems: Vec<String>,
}

/// `p` with `.` and `..` resolved by name, without touching the file system (the directory may not exist yet).
fn normalize(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    let mut root = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Prefix(_) | Component::RootDir => root.push(c.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.last().is_some_and(|l| l != "..") {
                    parts.pop();
                } else if root.as_os_str().is_empty() {
                    parts.push("..".into());
                }
            }
            Component::Normal(n) => parts.push(n.to_os_string()),
        }
    }
    parts.into_iter().fold(root, |acc, n| acc.join(n))
}

impl ResultsSettings {
    pub fn load(model_root: &Path) -> Self {
        let mut s = ResultsSettings { keep_run_records: DEFAULT_KEEP_RUN_RECORDS, runs_dir: model_root.join(".syscribe").join("runs"), runs_dir_custom: false, problems: Vec::new() };
        let table = std::fs::read_to_string(model_root.join(".syscribe.toml"))
            .ok()
            .and_then(|t| toml::from_str::<toml::Value>(&t).ok())
            .and_then(|d| d.get("results").and_then(|t| t.as_table().cloned()));
        for (k, v) in table.iter().flatten() {
            match k.as_str() {
                "keep_run_records" => match v.as_integer() {
                    Some(n) if n >= 0 => s.keep_run_records = n as usize,
                    _ => s.problems.push(format!("[results] keep_run_records must be a whole number, 0 or more (using {DEFAULT_KEEP_RUN_RECORDS})")),
                },
                "runs_dir" => match v.as_str().map(str::trim).filter(|d| !d.is_empty()) {
                    Some(d) => {
                        // Relative to the model root, like `[scripts] path`; absolute is taken as given. Resolved against the
                        // canonical model root so the agent policy can compare it with git and file-system paths.
                        let model = std::fs::canonicalize(model_root).unwrap_or_else(|_| model_root.to_path_buf());
                        let dir = normalize(&if Path::new(d).is_absolute() { PathBuf::from(d) } else { model.join(d) });
                        // The run files are `<run>.json`; in the model root or `.syscribe/` itself a run called `results`
                        // would replace the sidecar.
                        if dir == normalize(&model) || dir == normalize(&model.join(".syscribe")) {
                            s.problems.push(format!("[results] runs_dir `{d}` is the model root or .syscribe itself, where a run named `results` would replace the sidecar (using .syscribe/runs)"));
                        } else {
                            s.runs_dir = dir;
                            s.runs_dir_custom = true;
                        }
                    }
                    None => s.problems.push("[results] runs_dir must be a non-empty path (using .syscribe/runs)".to_string()),
                },
                other => s.problems.push(format!("[results] unknown key `{other}` (known: keep_run_records, runs_dir)")),
            }
        }
        s
    }
}

/// A run id as a file name: letters, digits, `_`, `-` and `.` (not first) stay, every other byte is `%XX`, so it is
/// reversible and unique; a very long id keeps a prefix and a hash of the whole.
pub fn run_file_stem(run: &str) -> String {
    let mut s = String::new();
    for (i, b) in run.bytes().enumerate() {
        if b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-') || (b == b'.' && i != 0) {
            s.push(b as char);
        } else {
            s.push_str(&format!("%{b:02X}"));
        }
    }
    if s.len() > 150 {
        s = format!("{}~{}", &s[..100], &blake3::hash(run.as_bytes()).to_hex()[..16]);
    }
    s
}

/// What an ingest changes on disk, decided in memory: the run files to write, the index, and the files of runs whose
/// records the cap dropped. [`check`](SavePlan::check) asks the write layer's guard about every path first, so a
/// refused path leaves nothing written; [`commit`](SavePlan::commit) writes the run files, then the index, then
/// removes the dropped files, so a crash leaves orphan files the index ignores, never an index that points at nothing.
#[derive(Debug)]
pub struct SavePlan {
    index_path: PathBuf,
    index: String,
    writes: Vec<(PathBuf, String)>,
    removals: Vec<PathBuf>,
    /// Runs whose records the cap dropped.
    pub dropped: Vec<String>,
}

impl SavePlan {
    /// Would the write layer refuse any of the paths? (The agent policy's `protected_paths`.)
    pub fn check(&self) -> std::io::Result<()> {
        for p in std::iter::once(&self.index_path).chain(self.writes.iter().map(|(p, _)| p)).chain(self.removals.iter()) {
            crate::fsio::check(p)?;
        }
        Ok(())
    }

    pub fn commit(self) -> std::io::Result<PathBuf> {
        for (p, text) in &self.writes {
            crate::fsio::create_dir_all(p.parent().expect("has parent"))?;
            crate::fsio::write_atomic(p, text)?;
        }
        crate::fsio::create_dir_all(self.index_path.parent().expect("has parent"))?;
        crate::fsio::write_atomic(&self.index_path, &self.index)?;
        for p in &self.removals {
            match crate::fsio::remove_file(p) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                // The index already says the records are gone; a file left behind is an orphan it ignores.
                Err(e) => eprintln!("warning: could not remove {}: {e}", p.display()),
            }
        }
        Ok(self.index_path)
    }
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

    /// The directory of the run files: `[results] runs_dir`, default `<model>/.syscribe/runs`.
    pub fn runs_dir(model_root: &Path) -> PathBuf {
        ResultsSettings::load(model_root).runs_dir
    }

    fn file_in(dir: &Path, run: &str) -> PathBuf {
        dir.join(format!("{}.json", run_file_stem(run)))
    }

    /// The file holding `run`'s records.
    pub fn run_file(model_root: &Path, run: &str) -> PathBuf {
        Self::file_in(&Self::runs_dir(model_root), run)
    }

    /// `run`'s records file: `None` when there is none, an error when it exists but cannot be read, never mistaken for
    /// "no records" and never overwritten.
    pub fn load_records(model_root: &Path, run: &str) -> Result<Option<RunRecords>, String> {
        Self::read_records(&Self::run_file(model_root, run))
    }

    fn read_records(p: &Path) -> Result<Option<RunRecords>, String> {
        match std::fs::read_to_string(p) {
            Ok(t) => serde_json::from_str(&t).map(Some).map_err(|e| format!("{} is not valid: {e} (fix or remove it)", p.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("cannot read {}: {e}", p.display())),
        }
    }

    /// A retained run with its records, or `None` for an unknown id.
    pub fn load_run(&self, model_root: &Path, run: &str) -> Result<Option<LoadedRun>, String> {
        let Some(index) = self.get(run).cloned() else { return Ok(None) };
        let file = Self::load_records(model_root, run)?;
        let file_missing = file.is_none() && index.record_count > 0;
        Ok(Some(LoadedRun { index, records: file.unwrap_or_default(), file: Self::run_file(model_root, run), file_missing }))
    }

    /// Decide what an ingest writes (GH #471): merge the staged records into their run files, count them in the index,
    /// then keep the records of the `keep` most recently ingested runs that hold any and drop the rest (their verdict
    /// maps stay). Newest: the runs this ingest touched, then by ingest time, a tie going to the later position. Nothing is written until
    /// [`SavePlan::commit`]; an unreadable run file is an error, never overwritten.
    pub fn plan(&mut self, model_root: &Path, keep: usize) -> Result<SavePlan, String> {
        let dir = Self::runs_dir(model_root);
        let mut files: BTreeMap<String, RunRecords> = BTreeMap::new();
        for p in std::mem::take(&mut self.pending) {
            if !files.contains_key(&p.run) {
                let f = Self::read_records(&Self::file_in(&dir, &p.run))?.unwrap_or_else(|| RunRecords { run: p.run.clone(), ..Default::default() });
                files.insert(p.run.clone(), f);
            }
            files.get_mut(&p.run).expect("inserted").apply(&p);
        }
        for (run, f) in &files {
            if let Some(r) = self.runs.iter_mut().find(|r| &r.run == run) {
                r.record_count = f.total();
                r.records_dropped = false;
            }
        }
        let (mut dropped, mut removals) = (Vec::new(), Vec::new());
        // Newest first. A run this ingest touched is newest whatever the clock says (an old run re-ingested within the
        // same second as a newer one must not lose the records just written); then by ingest time, then position.
        let mut order: Vec<usize> = (0..self.runs.len()).filter(|i| self.runs[*i].record_count > 0).collect();
        let rank = |i: usize| (files.contains_key(&self.runs[i].run), self.runs[i].ingested_at_unix, i);
        order.sort_by(|a, b| rank(*b).cmp(&rank(*a)));
        for i in order.into_iter().skip(keep) {
            let r = &mut self.runs[i];
            r.record_count = 0;
            r.records_dropped = true;
            files.remove(&r.run);
            removals.push(Self::file_in(&dir, &r.run));
            dropped.push(r.run.clone());
        }
        let writes = files.iter().map(|(run, f)| (Self::file_in(&dir, run), serde_json::to_string_pretty(f).expect("run records serialise") + "\n")).collect();
        Ok(SavePlan { index_path: Self::path(model_root), index: serde_json::to_string_pretty(self).expect("history serialises"), writes, removals, dropped })
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
        // The records themselves go to the run's file: staged here, written by the plan.
        if matches!(data.section(), Section::Records) && (!data.records.is_empty() || data.replace_records) {
            self.pending.push(Pending { run: run.to_string(), config: config.map(str::to_string), records: data.records.iter().map(slim_record).collect(), replace: data.replace_records });
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

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: &str = r#"[{"testCase":"TC-X-001","scenario":"S","verdict":"pass","measurements":[{"name":"t","value":"1 ms"}]}]"#;

    fn data(json: &str, at: u64) -> ResultsData {
        let mut d = ResultsData::parse_records(json, "src").unwrap();
        d.ingested_at_unix = at;
        d
    }

    fn tmp() -> PathBuf {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let d = std::env::temp_dir().join(format!("syscribe-runhist-unit-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// Ingest `data` for `run`, as `ingest-results --run` does: stage, plan, check, commit.
    fn ingest(h: &mut RunHistory, root: &Path, run: &str, d: &ResultsData, keep: usize) -> Vec<String> {
        h.record_for(run, d, Some("C"));
        let plan = h.plan(root, keep).unwrap();
        plan.check().unwrap();
        let dropped = plan.dropped.clone();
        plan.commit().unwrap();
        dropped
    }

    #[test]
    fn run_ids_become_safe_reversible_distinct_file_names() {
        assert_eq!(run_file_stem("R1"), "R1");
        assert_eq!(run_file_stem("SW-0.9.0_rc3"), "SW-0.9.0_rc3");
        assert_eq!(run_file_stem("rc/1 2"), "rc%2F1%202");
        assert_eq!(run_file_stem(".hidden"), "%2Ehidden", "a leading dot would hide the file");
        assert_ne!(run_file_stem("a%2Fb"), run_file_stem("a/b"), "a literal % is encoded too, so no two ids collide");
        let long = "x".repeat(300);
        let s = run_file_stem(&long);
        assert!(s.len() < 130 && s.starts_with(&"x".repeat(100)), "{s}");
        assert_ne!(s, run_file_stem(&format!("{long}y")), "a long id keeps a hash of the whole");
    }

    #[test]
    fn the_cap_keeps_the_newest_runs_and_a_touched_run_is_always_newest() {
        let root = tmp();
        let mut h = RunHistory::default();
        for (i, id) in ["R1", "R2", "R3"].iter().enumerate() {
            let dropped = ingest(&mut h, &root, id, &data(ONE, 100 + i as u64), 2);
            assert_eq!(dropped, if *id == "R3" { vec!["R1".to_string()] } else { vec![] });
        }
        assert!(!RunHistory::run_file(&root, "R1").exists() && RunHistory::run_file(&root, "R2").exists() && RunHistory::run_file(&root, "R3").exists());
        assert_eq!(h.get("R1").map(|r| (r.record_count, r.records_dropped)), Some((0, true)));
        // R1 is re-ingested with an older clock than R3's: it is the run just touched, so it is the newest.
        let dropped = ingest(&mut h, &root, "R1", &data(ONE, 50), 2);
        assert_eq!(dropped, vec!["R2".to_string()]);
        assert!(h.get("R1").is_some_and(|r| r.record_count == 1 && !r.records_dropped), "a re-ingest restores the run's flag");
        // Keeping none drops everything, the run just ingested included.
        let dropped = ingest(&mut h, &root, "R4", &data(ONE, 200), 0);
        assert_eq!(dropped.len(), 3, "{dropped:?}");
        assert!(std::fs::read_dir(RunHistory::runs_dir(&root)).unwrap().next().is_none(), "no run file is left");
    }

    #[test]
    fn records_supersede_by_key_and_replace_drops_the_rest() {
        let root = tmp();
        let mut h = RunHistory::default();
        ingest(&mut h, &root, "R1", &data(r#"[{"testCase":"TC-A-001","verdict":"pass"},{"testCase":"TC-B-001","verdict":"pass"}]"#, 1), 20);
        assert_eq!(RunHistory::load_records(&root, "R1").unwrap().unwrap().total(), 2);
        // A later ingest of TC-A supersedes its record and keeps TC-B's.
        ingest(&mut h, &root, "R1", &data(r#"[{"testCase":"TC-A-001","verdict":"fail"}]"#, 2), 20);
        let f = RunHistory::load_records(&root, "R1").unwrap().unwrap();
        let by: Vec<(&str, Verdict)> = f.by_config["C"].iter().map(|r| (r.test_case.as_str(), r.verdict)).collect();
        assert_eq!(by.len(), 2);
        assert!(by.contains(&("TC-A-001", Verdict::Fail)) && by.contains(&("TC-B-001", Verdict::Pass)), "{by:?}");
        // --replace drops every record the ingest does not contain.
        let mut replacing = data(r#"[{"testCase":"TC-C-001","verdict":"pass"}]"#, 3);
        replacing.replace_records = true;
        ingest(&mut h, &root, "R1", &replacing, 20);
        let f = RunHistory::load_records(&root, "R1").unwrap().unwrap();
        assert_eq!(f.by_config["C"].iter().map(|r| r.test_case.as_str()).collect::<Vec<_>>(), vec!["TC-C-001"]);
        assert_eq!(h.get("R1").map(|r| r.record_count), Some(1));
    }

    #[test]
    fn an_unreadable_run_file_stops_the_plan_and_is_left_alone() {
        let root = tmp();
        let mut h = RunHistory::default();
        ingest(&mut h, &root, "R1", &data(ONE, 1), 20);
        let f = RunHistory::run_file(&root, "R1");
        std::fs::write(&f, "{ not json").unwrap();
        h.record_for("R1", &data(ONE, 2), Some("C"));
        let e = h.plan(&root, 20).unwrap_err();
        assert!(e.contains("is not valid") && e.contains("fix or remove"), "{e}");
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "{ not json");
        // `load_run` reports the same for a reader.
        assert!(RunHistory::default().load_run(&root, "R1").unwrap().is_none(), "an unknown id is None, not an error");
    }

    fn model_with(toml: &str) -> PathBuf {
        let root = tmp();
        std::fs::write(root.join(".syscribe.toml"), toml).unwrap();
        std::fs::canonicalize(root).unwrap()
    }

    #[test]
    fn runs_dir_defaults_to_syscribe_runs_and_a_relative_one_is_resolved_against_the_model_root() {
        let root = model_with("");
        let s = ResultsSettings::load(&root);
        assert_eq!((s.runs_dir.clone(), s.runs_dir_custom, s.keep_run_records, s.problems.len()), (root.join(".syscribe").join("runs"), false, 20, 0));
        for (toml, want) in [
            ("[results]\nruns_dir = \"evidence/runs\"\n", "evidence/runs"),
            ("[results]\nruns_dir = \"../evidence/runs\"\n", "../evidence/runs"),
            ("[results]\nruns_dir = \"./a/../b\"\n", "b"),
        ] {
            let root = model_with(toml);
            let s = ResultsSettings::load(&root);
            assert!(s.runs_dir_custom && s.problems.is_empty(), "{toml}: {:?}", s.problems);
            assert_eq!(s.runs_dir, normalize(&root.join(want)), "{toml}");
        }
        // The run files follow it.
        let root = model_with("[results]\nruns_dir = \"evidence/runs\"\n");
        assert_eq!(RunHistory::run_file(&root, "R1"), root.join("evidence/runs/R1.json"));
    }

    #[test]
    fn an_absolute_runs_dir_is_taken_as_given() {
        let other = tmp();
        let root = model_with(&format!("[results]\nruns_dir = {:?}\n", other.display().to_string()));
        let s = ResultsSettings::load(&root);
        assert!(s.runs_dir_custom && s.runs_dir == other, "{:?} vs {:?}", s.runs_dir, other);
    }

    #[test]
    fn a_runs_dir_that_would_let_a_run_replace_the_sidecar_is_refused_and_the_default_applies() {
        for toml in ["[results]\nruns_dir = \".syscribe\"\n", "[results]\nruns_dir = \".\"\n", "[results]\nruns_dir = \"\"\n", "[results]\nruns_dir = 3\n", "[results]\nruns_dir = \"sub/..\"\n"] {
            let root = model_with(toml);
            let s = ResultsSettings::load(&root);
            assert!(!s.runs_dir_custom && s.runs_dir == root.join(".syscribe").join("runs") && s.problems.len() == 1, "{toml}: {:?}", s.problems);
            assert!(s.problems[0].contains("runs_dir"), "{:?}", s.problems);
        }
        // An unknown key names the known ones.
        let s = ResultsSettings::load(&model_with("[results]\nruns_folder = \"x\"\n"));
        assert!(s.problems[0].contains("keep_run_records, runs_dir"), "{:?}", s.problems);
    }

    #[test]
    fn normalize_resolves_dots_without_the_file_system() {
        assert_eq!(normalize(Path::new("/a/b/../c/./d")), PathBuf::from("/a/c/d"));
        assert_eq!(normalize(Path::new("a/../../b")), PathBuf::from("../b"));
        assert_eq!(normalize(Path::new("../../x")), PathBuf::from("../../x"));
        assert_eq!(normalize(Path::new("/../x")), PathBuf::from("/x"), "the root has no parent");
    }

    #[test]
    fn slim_records_keep_only_what_ranking_needs() {
        let full: ResultRecord = serde_json::from_str(
            r#"{"testCase":"TC-A-001","scenario":"S","verdict":"waived","rationale":"why","steps":[{"x":1}],"measurements":[{"name":"t","value":"1 ms","limit":{"op":"<","value":"2 ms"}}],
                "provenance":{"operator":"o","tool":"t","build":{"commit":"c"},"environment":"TE-1","startedAt":"2026-11-01T10:00:00Z","finishedAt":"2026-11-01T10:05:00Z","artefacts":[{"uri":"u","hash":"sha256:0000000000000000000000000000000000000000000000000000000000000000"}]}}"#,
        )
        .unwrap();
        let s = slim_record(&full);
        assert!(s.steps.is_empty() && s.rationale.as_deref() == Some("why") && s.measurements == full.measurements);
        let p = s.provenance.unwrap();
        assert_eq!((p.environment.as_deref(), p.finished_at.as_deref(), p.operator, p.tool, p.build, p.artefacts.len()), (Some("TE-1"), Some("2026-11-01T10:05:00Z"), None, None, None, 0));
        // No timing, environment or anything else: no provenance at all.
        let bare: ResultRecord = serde_json::from_str(r#"{"testCase":"TC-A-001","verdict":"pass","provenance":{"operator":"o"}}"#).unwrap();
        assert!(slim_record(&bare).provenance.is_none());
    }
}
