//! `matrix --by environment` — each requirement's verification per `TestEnvironment` (GH #437,
//! `REQ-TRS-ENVMATRIX-001`).
//!
//! A cell is the verdict of the active TestCases that verify the requirement, judged on the records made on that
//! environment alone, with the latest such record's build commit and date and a *stale* mark when the evidence is
//! older than the model. The data function ([`build`]) is shared with the MCP `coverage_matrix` tool.

use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;
use syscribe_model::result_join::{canonical, runs_on_ids, stale_reasons, unhealthy_environments};
use syscribe_model::result_records::ResultRecord;
use syscribe_model::results::{testcase_verdict, ResultsData};
use syscribe_model::safety_case::Verdict;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Pass,
    Fail,
    Waived,
    /// Declared, or run, but without a passing result on this environment.
    Gap,
    /// A pass whose evidence is older than the model.
    Stale,
    /// No verifying test declares or ran on this environment.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Rollup {
    Verified,
    Failing,
    Gap,
    Stale,
    Waived,
    /// No environment is declared, but evidence exists.
    Unscoped,
    /// No environment declared and no evidence.
    None,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    pub state: State,
    /// A verifying test declares this environment (`runsOn:`, its own or its plan's).
    pub declared: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stale_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub id: String,
    pub qualified_name: String,
    pub status: String,
    pub tests: Vec<String>,
    pub declared_environments: Vec<String>,
    pub cells: BTreeMap<String, Cell>,
    pub rollup: Rollup,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvCol {
    pub id: String,
    pub name: String,
    /// False for a retired or out-of-calibration environment, whose results do not count.
    pub usable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvMatrix {
    pub environments: Vec<EnvCol>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Default, Clone)]
pub struct Options<'a> {
    pub status: Option<&'a str>,
    pub tag: Option<&'a str>,
    pub gaps_only: bool,
}

// ── time ─────────────────────────────────────────────────────────────────────

/// Seconds since the epoch for an RFC 3339 timestamp (`2026-10-01T08:00:04Z`, `…+02:00`, fractions ignored).
pub fn rfc3339_epoch(t: &str) -> Option<i64> {
    let (date, rest) = t.split_once(['T', 't', ' '])?;
    let mut d = date.split('-');
    let (y, m, day): (i64, i64, i64) = (d.next()?.parse().ok()?, d.next()?.parse().ok()?, d.next()?.parse().ok()?);
    let split_at = rest.find(['Z', 'z', '+', '-']).unwrap_or(rest.len());
    let (clock, zone) = rest.split_at(split_at);
    let mut c = clock.split(':');
    let (hh, mm): (i64, i64) = (c.next()?.parse().ok()?, c.next()?.parse().ok()?);
    let ss: i64 = c.next().map(|s| s.split('.').next().unwrap_or("0").parse().ok()).unwrap_or(Some(0))?;
    let offset = match zone.chars().next() {
        Some(sign @ ('+' | '-')) => {
            let (oh, om) = zone[1..].split_once(':').unwrap_or((&zone[1..], "0"));
            let secs = oh.parse::<i64>().ok()? * 3600 + om.parse::<i64>().ok()? * 60;
            if sign == '+' { secs } else { -secs }
        }
        _ => 0,
    };
    // Days from civil (Howard Hinnant).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + hh * 3600 + mm * 60 + ss - offset)
}

// ── git (currency of a build) ────────────────────────────────────────────────

/// Answers "does this build contain the last change to that file?" with memoised `git` calls.
struct Git {
    root: Option<PathBuf>,
    last_change: RefCell<HashMap<String, Option<String>>>,
    contains: RefCell<HashMap<(String, String), Option<bool>>>,
}

impl Git {
    fn new(model_root: Option<&Path>) -> Git {
        let root = model_root.and_then(|m| std::fs::canonicalize(m).ok()).and_then(|a| syscribe_model::config::detect_git_root(&a));
        Git { root, last_change: Default::default(), contains: Default::default() }
    }

    fn run(&self, args: &[&str]) -> Option<(i32, String)> {
        let root = self.root.as_ref()?;
        let o = std::process::Command::new("git").arg("-C").arg(root).args(args).output().ok()?;
        Some((o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).trim().to_string()))
    }

    /// The last commit that changed `file`, and the file as git names it.
    fn last_change(&self, file: &str) -> Option<(String, String)> {
        let root = self.root.as_ref()?;
        let rel = crate::impactfiles::to_git_rel(root, Path::new(file))?;
        let cached = self.last_change.borrow().get(&rel).cloned();
        let commit = match cached {
            Some(c) => c,
            None => {
                let c = self.run(&["log", "-1", "--format=%H", "--", &rel]).filter(|(code, out)| *code == 0 && !out.is_empty()).map(|(_, out)| out);
                self.last_change.borrow_mut().insert(rel.clone(), c.clone());
                c
            }
        };
        commit.map(|c| (c, rel))
    }

    /// `Some(true)` when `build` contains `change`; `Some(false)` when it predates it; `None` when `build` is not a
    /// commit of this repository.
    fn build_contains(&self, build: &str, change: &str) -> Option<bool> {
        let key = (build.to_string(), change.to_string());
        if let Some(v) = self.contains.borrow().get(&key) {
            return *v;
        }
        let known = self.run(&["rev-parse", "--verify", "--quiet", &format!("{build}^{{commit}}")]).is_some_and(|(c, _)| c == 0);
        let v = if !known {
            None
        } else {
            match self.run(&["merge-base", "--is-ancestor", change, build]) {
                Some((0, _)) => Some(true),
                Some((1, _)) => Some(false),
                _ => None,
            }
        };
        self.contains.borrow_mut().insert(key, v);
        v
    }
}

// ── the matrix ───────────────────────────────────────────────────────────────

fn rank(v: Verdict) -> u8 {
    match v {
        Verdict::Fail => 3,
        Verdict::Unknown => 2,
        Verdict::Waived => 1,
        Verdict::Pass => 0,
    }
}

fn env_of(r: &ResultRecord) -> Option<&str> {
    r.provenance.as_ref().and_then(|p| p.environment.as_deref())
}

fn record_time(r: &ResultRecord) -> Option<i64> {
    let p = r.provenance.as_ref()?;
    p.finished_at.as_deref().or(p.started_at.as_deref()).and_then(rfc3339_epoch)
}

/// Build the matrix. `model_root` lets the currency of a build commit be judged against git history; without a
/// repository only the content check applies.
pub fn build(elements: &[RawElement], results: Option<&ResultsData>, model_root: Option<&Path>, opts: &Options) -> EnvMatrix {
    let resolver = Resolver::new(elements);
    let unhealthy = unhealthy_environments(elements);
    let git = Git::new(model_root);

    // Active verifying TestCases of each requirement, and the environments each test declares.
    let mut tests_of: HashMap<String, Vec<&RawElement>> = HashMap::new();
    let tests: Vec<&RawElement> =
        elements.iter().filter(|e| Resolver::is_native_testcase(e) && e.frontmatter.status.as_deref() == Some("active")).collect();
    for tc in &tests {
        for v in tc.frontmatter.verifies.iter().flatten() {
            if let Some(req) = resolver.resolve_ref(elements, v) {
                tests_of.entry(req.qualified_name.clone()).or_default().push(tc);
            }
        }
    }
    let mut tc_envs: HashMap<String, BTreeSet<String>> = HashMap::new();
    for tc in &tests {
        tc_envs.entry(tc.qualified_name.clone()).or_default().extend(runs_on_ids(tc, elements, &resolver));
    }
    for plan in elements.iter().filter(|e| Resolver::is_test_plan(e)) {
        let envs = runs_on_ids(plan, elements, &resolver);
        for m in plan.frontmatter.test_cases.iter().flatten() {
            if let Some(tc) = resolver.resolve_ref(elements, m) {
                tc_envs.entry(tc.qualified_name.clone()).or_default().extend(envs.iter().cloned());
            }
        }
    }

    // Columns: declared by a verifying test, or named by a recorded result.
    let mut env_ids: BTreeSet<String> = BTreeSet::new();
    for tcs in tests_of.values() {
        for tc in tcs {
            env_ids.extend(tc_envs.get(&tc.qualified_name).into_iter().flatten().cloned());
        }
    }
    if let Some(r) = results {
        for rec in r.records.iter().chain(r.by_config.values().flat_map(|c| c.records.iter())) {
            if let Some(e) = env_of(rec) {
                env_ids.insert(e.to_string());
            }
        }
    }
    let environments: Vec<EnvCol> = env_ids
        .iter()
        .map(|id| {
            let el = elements.iter().find(|e| e.frontmatter.element_type == Some(ElementType::TestEnvironment) && canonical(e) == *id);
            EnvCol { id: id.clone(), name: el.and_then(|e| e.frontmatter.name.clone()).unwrap_or_else(|| id.clone()), usable: !unhealthy.contains(id) }
        })
        .collect();
    let views: HashMap<&str, ResultsData> = match results {
        Some(r) => env_ids.iter().map(|e| (e.as_str(), r.view_for_env(e, &unhealthy))).collect(),
        None => HashMap::new(),
    };

    let mut reqs: Vec<&RawElement> = elements
        .iter()
        .filter(|e| Resolver::is_native_requirement(e))
        .filter(|e| opts.status.is_none_or(|s| e.frontmatter.status.as_deref() == Some(s)))
        .filter(|e| opts.tag.is_none_or(|t| e.frontmatter.tags.as_ref().is_some_and(|ts| ts.iter().any(|x| x == t))))
        .collect();
    reqs.sort_by_key(|e| canonical(e));

    let mut rows: Vec<Row> = Vec::new();
    for req in reqs {
        let verifying: &[&RawElement] = tests_of.get(&req.qualified_name).map(Vec::as_slice).unwrap_or(&[]);
        let declared: BTreeSet<String> = verifying.iter().flat_map(|tc| tc_envs.get(&tc.qualified_name).into_iter().flatten().cloned()).collect();
        let mut cells: BTreeMap<String, Cell> = BTreeMap::new();
        for env in &env_ids {
            cells.insert(env.clone(), cell_for(env, req, verifying, &tc_envs, results, views.get(env.as_str()), !unhealthy.contains(env), elements, &resolver, &git));
        }
        let rollup = rollup_of(&declared, &cells);
        rows.push(Row {
            id: canonical(req),
            qualified_name: req.qualified_name.clone(),
            status: req.frontmatter.status.clone().unwrap_or_default(),
            tests: verifying.iter().map(|t| canonical(t)).collect(),
            declared_environments: declared.into_iter().collect(),
            cells,
            rollup,
        });
    }
    if opts.gaps_only {
        rows.retain(|r| matches!(r.rollup, Rollup::Gap | Rollup::Failing | Rollup::Stale));
    }
    EnvMatrix { environments, rows }
}

#[allow(clippy::too_many_arguments)]
fn cell_for(
    env: &str,
    req: &RawElement,
    verifying: &[&RawElement],
    tc_envs: &HashMap<String, BTreeSet<String>>,
    results: Option<&ResultsData>,
    view: Option<&ResultsData>,
    usable: bool,
    elements: &[RawElement],
    resolver: &Resolver,
    git: &Git,
) -> Cell {
    let declares = |tc: &RawElement| tc_envs.get(&tc.qualified_name).is_some_and(|s| s.contains(env));
    // Records of a test on this environment, from the unfiltered results (the view drops an unusable environment).
    let records_of = |tc: &RawElement| -> Vec<&ResultRecord> {
        let id = canonical(tc);
        results.map(|r| r.records.iter().chain(r.by_config.values().flat_map(|c| c.records.iter())).filter(|rec| rec.test_case == id && env_of(rec) == Some(env)).collect()).unwrap_or_default()
    };
    let relevant: Vec<&RawElement> = verifying.iter().copied().filter(|tc| declares(tc) || !records_of(tc).is_empty()).collect();
    let declared = relevant.iter().any(|tc| declares(tc));
    if relevant.is_empty() {
        return Cell { state: State::None, declared: false, build_commit: None, build_version: None, date: None, stale_reasons: vec![], note: None };
    }

    let worst = relevant.iter().map(|tc| testcase_verdict(tc, view)).max_by_key(|v| rank(*v)).unwrap_or(Verdict::Unknown);
    let all: Vec<&ResultRecord> = relevant.iter().flat_map(|tc| records_of(tc)).collect();
    let latest = all.iter().copied().max_by_key(|r| record_time(r).unwrap_or(i64::MIN));
    let prov = latest.and_then(|r| r.provenance.as_ref());
    let build_commit = prov.and_then(|p| p.build.as_ref()).and_then(|b| b.commit.clone());

    let mut reasons: Vec<String> = Vec::new();
    let mut note: Option<String> = None;
    if !usable {
        note = Some("the environment is retired or out of calibration; results recorded there do not count".to_string());
    }
    // 1. Content: the TestCase, or a requirement it verifies, changed since these records were ingested.
    for tc in &relevant {
        let recs: Vec<&ResultRecord> = records_of(tc);
        for r in stale_reasons(tc, recs.iter().copied(), elements, resolver) {
            if !reasons.contains(&r) {
                reasons.push(r);
            }
        }
    }
    // 2. History: the build that produced the latest evidence does not contain the last change to the requirement
    //    or to a verifying test.
    if let Some(b) = &build_commit {
        let files = std::iter::once(&req.file_path).chain(relevant.iter().map(|t| &t.file_path));
        let mut unknown = false;
        for f in files {
            let Some((change, rel)) = git.last_change(f) else { continue };
            match git.build_contains(b, &change) {
                Some(false) => reasons.push(format!("build {} predates the change {} to {rel}", &b[..b.len().min(7)], &change[..7])),
                Some(true) => {}
                None => unknown = true,
            }
        }
        if unknown && git.root.is_some() && reasons.is_empty() && note.is_none() {
            note = Some(format!("build {b} is not a commit of this repository, so its currency cannot be judged"));
        }
    }
    let state = match worst {
        Verdict::Fail => State::Fail,
        Verdict::Waived => State::Waived,
        Verdict::Unknown => State::Gap,
        Verdict::Pass if !reasons.is_empty() => State::Stale,
        Verdict::Pass => State::Pass,
    };
    Cell {
        state,
        declared,
        build_commit,
        build_version: prov.and_then(|p| p.build.as_ref()).and_then(|b| b.version.clone()),
        date: prov.and_then(|p| p.finished_at.clone().or_else(|| p.started_at.clone())),
        stale_reasons: reasons,
        note,
    }
}

/// "Verified on every environment the verifying tests declare."
fn rollup_of(declared: &BTreeSet<String>, cells: &BTreeMap<String, Cell>) -> Rollup {
    if declared.is_empty() {
        return if cells.values().any(|c| c.state != State::None) { Rollup::Unscoped } else { Rollup::None };
    }
    let states: Vec<State> = declared.iter().map(|e| cells.get(e).map(|c| c.state).unwrap_or(State::Gap)).collect();
    let any = |s: State| states.contains(&s);
    if any(State::Fail) {
        Rollup::Failing
    } else if any(State::Gap) || any(State::None) {
        Rollup::Gap
    } else if any(State::Stale) {
        Rollup::Stale
    } else if any(State::Waived) {
        Rollup::Waived
    } else {
        Rollup::Verified
    }
}

// ── output ───────────────────────────────────────────────────────────────────

fn short(c: &str) -> &str {
    &c[..c.len().min(7)]
}

fn cell_text(c: &Cell) -> String {
    let when = c.date.as_deref().map(|d| d.split('T').next().unwrap_or(d)).unwrap_or("");
    let at = match (&c.build_commit, when) {
        (Some(b), "") => format!(" @{}", short(b)),
        (Some(b), w) => format!(" @{} {w}", short(b)),
        (None, "") => String::new(),
        (None, w) => format!(" {w}"),
    };
    match c.state {
        State::None => "—".to_string(),
        State::Gap => if c.note.is_some() { "gap*".to_string() } else { "gap".to_string() },
        State::Pass => format!("pass{at}"),
        State::Fail => format!("FAIL{at}"),
        State::Waived => format!("waived{at}"),
        State::Stale => format!("pass (stale){at}"),
    }
}

pub fn to_json(m: &EnvMatrix) -> serde_json::Value {
    let count = |r: Rollup| m.rows.iter().filter(|x| x.rollup == r).count();
    serde_json::json!({
        "schemaVersion": "1.0",
        "environments": m.environments,
        "rows": m.rows,
        "summary": {
            "requirements": m.rows.len(),
            "verified": count(Rollup::Verified),
            "gap": count(Rollup::Gap),
            "failing": count(Rollup::Failing),
            "stale": count(Rollup::Stale),
            "waived": count(Rollup::Waived),
            "unscoped": count(Rollup::Unscoped),
            "none": count(Rollup::None),
        },
    })
}

pub fn cmd_env_matrix(elements: &[RawElement], results: Option<&ResultsData>, model_root: &Path, opts: &Options, json: bool) {
    let m = build(elements, results, Some(model_root), opts);
    if json {
        println!("{}", serde_json::to_string_pretty(&to_json(&m)).unwrap());
        return;
    }
    if m.rows.is_empty() {
        println!("No requirements{}.", if opts.gaps_only { " with a verification gap" } else { "" });
        return;
    }
    println!("# Requirement × TestEnvironment\n");
    if m.environments.is_empty() {
        println!("No TestEnvironment is declared by a verifying test (`runsOn:`) or named by a recorded result.\n");
    }
    let mut head = String::from("| Requirement | Status |");
    let mut sep = String::from("|---|---|");
    for e in &m.environments {
        head.push_str(&format!(" {}{} |", e.name, if e.usable { "" } else { " (unusable)" }));
        sep.push_str("---|");
    }
    println!("{head} Roll-up |\n{sep}---|");
    for r in &m.rows {
        let mut line = format!("| {} | {} |", r.id, r.status);
        for e in &m.environments {
            line.push_str(&format!(" {} |", cell_text(&r.cells[&e.id])));
        }
        println!("{line} {} |", serde_json::to_value(r.rollup).unwrap().as_str().unwrap_or(""));
    }
    let j = to_json(&m);
    let s = &j["summary"];
    println!(
        "\n{} requirement(s): {} verified, {} gap, {} failing, {} stale, {} waived, {} unscoped, {} without evidence.",
        s["requirements"], s["verified"], s["gap"], s["failing"], s["stale"], s["waived"], s["unscoped"], s["none"]
    );
    println!("A cell is the worst verdict of the active verifying tests on that environment; `gap` is an environment a test declares (or ran on) without a passing result there; `stale` is a pass older than the model; `gap*` marks an unusable environment.");
    let notes: Vec<String> = m
        .rows
        .iter()
        .flat_map(|r| r.cells.iter().filter_map(move |(e, c)| c.note.as_ref().map(|n| format!("- {} on {e}: {n}", r.id))))
        .collect();
    let stale: Vec<String> = m
        .rows
        .iter()
        .flat_map(|r| r.cells.iter().filter(|(_, c)| !c.stale_reasons.is_empty()).map(move |(e, c)| format!("- {} on {e}: {}", r.id, c.stale_reasons.join("; "))))
        .collect();
    if !stale.is_empty() {
        println!("\nStale evidence:");
        stale.iter().for_each(|l| println!("{l}"));
    }
    if !notes.is_empty() {
        println!("\nNotes:");
        notes.iter().for_each(|l| println!("{l}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_parses_zones_and_fractions() {
        assert_eq!(rfc3339_epoch("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(rfc3339_epoch("2026-10-01T08:00:04Z"), Some(1_790_841_604));
        assert_eq!(rfc3339_epoch("2026-10-01T10:00:04+02:00"), rfc3339_epoch("2026-10-01T08:00:04Z"));
        assert_eq!(rfc3339_epoch("2026-10-01T08:00:04.250Z"), rfc3339_epoch("2026-10-01T08:00:04Z"));
        assert!(rfc3339_epoch("2026-10-01T08:00:05Z") > rfc3339_epoch("2026-10-01T08:00:04Z"));
        assert_eq!(rfc3339_epoch("not a time"), None);
    }

    fn cell(state: State) -> Cell {
        Cell { state, declared: true, build_commit: None, build_version: None, date: None, stale_reasons: vec![], note: None }
    }

    fn rollup(declared: &[&str], cells: &[(&str, State)]) -> Rollup {
        rollup_of(&declared.iter().map(|s| s.to_string()).collect(), &cells.iter().map(|(e, s)| (e.to_string(), cell(*s))).collect())
    }

    #[test]
    fn the_rollup_is_the_worst_declared_cell() {
        assert_eq!(rollup(&["A", "B"], &[("A", State::Pass), ("B", State::Pass)]), Rollup::Verified);
        assert_eq!(rollup(&["A", "B"], &[("A", State::Pass), ("B", State::Gap)]), Rollup::Gap);
        assert_eq!(rollup(&["A", "B"], &[("A", State::Fail), ("B", State::Gap)]), Rollup::Failing, "a failure outranks a gap");
        assert_eq!(rollup(&["A", "B"], &[("A", State::Pass), ("B", State::Stale)]), Rollup::Stale);
        assert_eq!(rollup(&["A"], &[("A", State::Waived)]), Rollup::Waived);
        assert_eq!(rollup(&["A"], &[]), Rollup::Gap, "a declared environment with no cell is a gap");
    }

    #[test]
    fn an_environment_nobody_declared_does_not_affect_the_rollup() {
        assert_eq!(rollup(&["A"], &[("A", State::Pass), ("Z", State::Fail)]), Rollup::Verified, "Z is recorded but undeclared");
        assert_eq!(rollup(&[], &[("A", State::Pass)]), Rollup::Unscoped);
        assert_eq!(rollup(&[], &[("A", State::None)]), Rollup::None);
    }

    #[test]
    fn cells_read_plainly() {
        let mut c = cell(State::Pass);
        c.build_commit = Some("abcdef1234".into());
        c.date = Some("2026-10-01T08:00:04Z".into());
        assert_eq!(cell_text(&c), "pass @abcdef1 2026-10-01");
        c.state = State::Stale;
        assert_eq!(cell_text(&c), "pass (stale) @abcdef1 2026-10-01");
        assert_eq!(cell_text(&cell(State::None)), "—");
        assert_eq!(cell_text(&cell(State::Gap)), "gap");
        assert_eq!(cell_text(&cell(State::Fail)), "FAIL");
    }
}
