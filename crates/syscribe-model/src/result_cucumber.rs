//! Cucumber JSON as a built-in results format (REQ-TRS-RESCUKE-001).
//!
//! The classic `json` formatter of Cucumber-JVM, Cucumber.js and Cucumber-Ruby: an array of features, each with
//! `elements` that are scenarios (and, in JVM and Ruby, a `background` element before each scenario). The three
//! differ in places that matter, and this module reads all of them:
//!
//! * a background is a separate element (JVM, Ruby) or merged into the scenario's own steps (JS);
//! * hooks are `before`/`after` arrays on an element (JVM, Ruby) or steps with the keyword `Before`/`After` (JS);
//! * the rows of a Scenario Outline have distinct ids (JVM, Ruby) or the same id and distinct lines (JS);
//! * a retried scenario appears once per attempt (JVM) or only as its last attempt (JS).
//!
//! The newer NDJSON message protocol is a different format and is refused with a message that says so.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::element::RawElement;
use crate::result_join::{canonical, testcase_lookup};
use crate::result_records::{badness, Provenance, ResultRecord};
use crate::results::{gherkin_scenario_titles, one_line, Verdict};

/// One scenario, or one row of a Scenario Outline, after its attempts and its background are folded in.
#[derive(Debug, Clone)]
pub struct CukeScenario {
    pub feature: String,
    pub title: String,
    /// `Some(n)` for the n-th row of a Scenario Outline.
    pub row: Option<usize>,
    pub tags: Vec<String>,
    pub verdict: Verdict,
    /// The first failing step (or hook) and its message, or the first step that did not pass.
    pub detail: Option<(String, Option<String>)>,
    pub started: Option<String>,
    pub finished: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CukeReport {
    pub scenarios: Vec<CukeScenario>,
}

/// One attempt of one element, before attempts are folded.
struct Attempt {
    verdict: Verdict,
    detail: Option<(String, Option<String>)>,
    started: Option<String>,
    duration_ns: u64,
}

/// (status, label, message, duration) of every step and hook of an element, in execution order.
fn items(e: &Value) -> Result<Vec<(String, String, Option<String>, u64)>, String> {
    let mut out = Vec::new();
    let mut take = |arr: Option<&Value>, hook: &str| -> Result<(), String> {
        for it in arr.and_then(Value::as_array).into_iter().flatten() {
            let r = it.get("result").ok_or_else(|| "a step or hook has no result".to_string())?;
            let status = r.get("status").and_then(Value::as_str).ok_or_else(|| "a step or hook result has no status".to_string())?.to_string();
            let label = match it.get("name").and_then(Value::as_str) {
                Some(n) => format!("{}{}", it.get("keyword").and_then(Value::as_str).unwrap_or(""), n).trim().to_string(),
                // a hook has no name: Cucumber.js writes it as a step with the keyword Before/After, JVM and Ruby in an array
                None => {
                    let kw = it.get("keyword").and_then(Value::as_str).map(|k| k.trim().to_lowercase()).filter(|k| !k.is_empty());
                    format!("{} hook {}", kw.as_deref().unwrap_or(hook), it.pointer("/match/location").and_then(Value::as_str).unwrap_or("")).trim().to_string()
                }
            };
            let msg = r.get("error_message").and_then(Value::as_str).map(|m| one_line(m.lines().find(|l| !l.trim().is_empty()).unwrap_or(m)));
            out.push((status, label, msg.filter(|m| !m.is_empty()), r.get("duration").and_then(Value::as_u64).unwrap_or(0)));
        }
        Ok(())
    };
    take(e.get("before"), "before")?;
    take(e.get("steps"), "step")?;
    take(e.get("after"), "after")?;
    Ok(out)
}

/// Fold the items of a scenario (with its background's) into a verdict and the detail a reader needs.
/// `own_steps` is how many steps the scenario itself has, as against its background and hooks: a scenario with none
/// tested nothing, however well its background went.
fn judge(all: &[(String, String, Option<String>, u64)], own_steps: usize) -> Result<(Verdict, Option<(String, Option<String>)>), String> {
    let mut failing: Option<&(String, String, Option<String>, u64)> = None;
    let mut unfinished: Option<&(String, String, Option<String>, u64)> = None;
    for it in all {
        match it.0.as_str() {
            "passed" => {}
            "failed" | "ambiguous" => {
                failing.get_or_insert(it);
            }
            "skipped" | "pending" | "undefined" | "unknown" => {
                unfinished.get_or_insert(it);
            }
            other => return Err(format!("cucumber: a step has the unknown status '{other}'")),
        }
    }
    Ok(if let Some(f) = failing {
        (Verdict::Fail, Some((f.1.clone(), f.2.clone())))
    } else if let Some(u) = unfinished {
        (Verdict::NotRun, Some((format!("{} ({})", u.1, u.0), u.2.clone())))
    } else if all.is_empty() || own_steps == 0 {
        (Verdict::NotRun, Some(("the scenario has no steps".to_string(), None)))
    } else {
        (Verdict::Pass, None)
    })
}

/// Parse a Cucumber JSON formatter report.
pub fn parse(text: &str) -> Result<CukeReport, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let value: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            // The message protocol is one JSON object per line; a single JSON document it is not.
            let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
            if let Ok(Value::Object(o)) = serde_json::from_str::<Value>(first) {
                if ["meta", "source", "gherkinDocument", "pickle", "testCase", "testRunStarted", "testCaseStarted", "testStepFinished"].iter().any(|k| o.contains_key(*k)) {
                    return Err("cucumber: this is the newer NDJSON message protocol, not the `json` formatter; write the report with `--format json:<file>`".to_string());
                }
            }
            return Err(format!("cucumber: not valid JSON: {e}"));
        }
    };
    let features = match &value {
        Value::Array(a) => a,
        Value::Object(o) if o.contains_key("meta") || o.contains_key("gherkinDocument") => {
            return Err("cucumber: this is a message of the newer message protocol, not the `json` formatter's array of features".to_string())
        }
        other => return Err(format!("cucumber: expected the json formatter's array of features, found {}", kind(other))),
    };
    if features.is_empty() {
        return Err("cucumber: no features found (empty array)".to_string());
    }
    // key (feature, element id, line) -> attempts, in first-appearance order
    let mut order: Vec<(String, String, u64)> = Vec::new();
    let mut attempts: BTreeMap<(String, String, u64), (Vec<Attempt>, String, String, bool, Vec<String>)> = BTreeMap::new();
    for f in features {
        let fobj = f.as_object().ok_or_else(|| format!("cucumber: a feature must be an object, found {}", kind(f)))?;
        let uri = fobj.get("uri").and_then(Value::as_str).or_else(|| fobj.get("id").and_then(Value::as_str)).unwrap_or("").to_string();
        let fname = fobj.get("name").and_then(Value::as_str).unwrap_or(&uri).to_string();
        let ftags: Vec<String> = fobj.get("tags").and_then(Value::as_array).into_iter().flatten().filter_map(|t| t.get("name").and_then(Value::as_str)).map(String::from).collect();
        let mut background: Vec<(String, String, Option<String>, u64)> = Vec::new();
        for e in fobj.get("elements").and_then(Value::as_array).into_iter().flatten() {
            match e.get("type").and_then(Value::as_str).unwrap_or("scenario") {
                // the background that applies to the scenarios after it (JVM and Ruby write one before each)
                "background" => background = items(e)?,
                "scenario" => {
                    let mut all = background.clone();
                    all.extend(items(e)?);
                    let own_steps = e.get("steps").and_then(Value::as_array).map_or(0, Vec::len);
                    let (verdict, detail) = judge(&all, own_steps)?;
                    let title = e.get("name").and_then(Value::as_str).unwrap_or("").to_string();
                    let outline = e.get("keyword").and_then(Value::as_str).is_some_and(|k| k.contains("Outline") || k.contains("Template"));
                    let line = e.get("line").and_then(Value::as_u64).unwrap_or(0);
                    let id = e.get("id").and_then(Value::as_str).map(String::from).unwrap_or_else(|| title.clone());
                    let mut tags = ftags.clone();
                    for t in e.get("tags").and_then(Value::as_array).into_iter().flatten().filter_map(|t| t.get("name").and_then(Value::as_str)) {
                        if !tags.iter().any(|x| x == t) {
                            tags.push(t.to_string());
                        }
                    }
                    let started = e.get("start_timestamp").and_then(Value::as_str).filter(|t| crate::result_records::is_rfc3339(t)).map(String::from);
                    let duration_ns = all.iter().map(|i| i.3).sum();
                    let key = (uri.clone(), id, line);
                    if !attempts.contains_key(&key) {
                        order.push(key.clone());
                    }
                    attempts.entry(key).or_insert_with(|| (Vec::new(), fname.clone(), title, outline, tags)).0.push(Attempt { verdict, detail, started, duration_ns });
                }
                _ => {}
            }
        }
    }
    if order.is_empty() {
        return Err("cucumber: no scenarios found".to_string());
    }
    // rows of an outline are numbered in report order within (feature, title)
    let mut rows: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut report = CukeReport::default();
    for key in order {
        let (att, fname, title, outline, tags) = attempts.remove(&key).expect("recorded above");
        let last = att.last().expect("at least one attempt");
        // the last attempt decides; a pass after an earlier failure is flaky, not a pass
        let verdict = if last.verdict == Verdict::Pass && att.iter().any(|a| a.verdict != Verdict::Pass) { Verdict::Flaky } else { last.verdict };
        let detail = if verdict == Verdict::Flaky { att.iter().find(|a| a.verdict != Verdict::Pass).and_then(|a| a.detail.clone()) } else { last.detail.clone() };
        let started = att.iter().filter_map(|a| a.started.clone()).min();
        let finished = last.started.as_deref().and_then(|s| crate::result_robot::add_seconds_zoned(s, last.duration_ns as f64 / 1e9));
        let row = outline.then(|| {
            let n = rows.entry((key.0.clone(), title.clone())).or_insert(0);
            *n += 1;
            *n
        });
        report.scenarios.push(CukeScenario { feature: fname, title, row, tags, verdict, detail, started, finished });
    }
    Ok(report)
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

fn step_entry(s: &CukeScenario) -> Value {
    let name = match s.row {
        Some(n) => format!("{} [row {n}]", s.title),
        None => s.title.clone(),
    };
    let mut v = serde_json::json!({"scenario": name, "status": status_word(s.verdict)});
    if let Some((step, msg)) = &s.detail {
        v["step"] = Value::String(step.clone());
        if let Some(m) = msg {
            v["message"] = Value::String(m.clone());
        }
    }
    v
}

fn status_word(v: Verdict) -> &'static str {
    match v {
        Verdict::Pass => "passed",
        Verdict::Fail => "failed",
        Verdict::Flaky => "flaky",
        _ => "not run",
    }
}

fn new_record(test_case: String, scenario: Option<String>) -> ResultRecord {
    ResultRecord {
        test_case,
        scenario,
        verdict: Verdict::Pass,
        rationale: None,
        steps: vec![],
        measurements: vec![],
        provenance: Some(Provenance::default()),
        tc_hash: None,
        req_hashes: Default::default(),
        condition: Default::default(),
    }
}

/// Fold one scenario into a record: the worst verdict, the scenario entries (worst first) and the span of the times.
fn fold(slot: &mut ResultRecord, s: &CukeScenario) {
    if badness(s.verdict) > badness(slot.verdict) {
        slot.verdict = s.verdict;
    }
    slot.steps.push(step_entry(s));
    let p = slot.provenance.as_mut().expect("set by new_record");
    if let Some(t) = &s.started {
        if p.started_at.as_ref().map_or(true, |o| t < o) {
            p.started_at = Some(t.clone());
        }
    }
    if let Some(t) = &s.finished {
        if p.finished_at.as_ref().map_or(true, |o| t > o) {
            p.finished_at = Some(t.clone());
        }
    }
}

/// Turn the scenarios into records. A tag that is a TestCase id or qualified name, or `@extref:<value>`, joins a
/// scenario to it; the scenario is recorded against the TestCase's `Scenario:` of the same title, else as the
/// TestCase's own verdict. The rows of an outline are recorded as `<title> [row N]` and the outline's own title
/// carries the worst of them. A scenario that joins nothing comes back under a `cucumber:` key, so the join reports it.
pub fn to_records(report: &CukeReport, elems: &[RawElement]) -> Vec<ResultRecord> {
    let lookup = testcase_lookup(elems);
    let mut folded: BTreeMap<(String, Option<String>), ResultRecord> = BTreeMap::new();
    let mut unmatched: Vec<ResultRecord> = Vec::new();
    for s in &report.scenarios {
        let mut targets: Vec<&RawElement> = Vec::new();
        for tag in &s.tags {
            let t = tag.strip_prefix('@').unwrap_or(tag);
            let key = if t.len() > 7 && t[..7].eq_ignore_ascii_case("extref:") { &t[7..] } else { t };
            if let Some(tc) = lookup(key) {
                if !targets.iter().any(|x| std::ptr::eq(*x, tc)) {
                    targets.push(tc);
                }
            }
        }
        if targets.is_empty() {
            let name = match s.row {
                Some(n) => format!("cucumber:{}.{} [row {n}]", s.feature, s.title),
                None => format!("cucumber:{}.{}", s.feature, s.title),
            };
            let mut r = new_record(name, None);
            r.verdict = s.verdict;
            r.steps.push(step_entry(s));
            let p = r.provenance.as_mut().expect("set by new_record");
            p.started_at = s.started.clone();
            p.finished_at = s.finished.clone();
            unmatched.push(r);
            continue;
        }
        for tc in targets {
            let id = canonical(tc);
            let own = gherkin_scenario_titles(&tc.doc).into_iter().find(|t| one_line(t) == one_line(&s.title));
            if let Some(n) = s.row {
                // the row itself, under a title the TestCase does not list
                fold(folded.entry((id.clone(), Some(format!("{} [row {n}]", s.title)))).or_insert_with(|| new_record(id.clone(), Some(format!("{} [row {n}]", s.title)))), s);
            }
            // the scenario (or the outline's own title, carrying the worst of its rows), else the TestCase's verdict
            fold(folded.entry((id.clone(), own.clone())).or_insert_with(|| new_record(id.clone(), own)), s);
        }
    }
    let mut out: Vec<ResultRecord> = folded.into_values().collect();
    for r in &mut out {
        r.steps.sort_by_key(|s| match s["status"].as_str() {
            Some("failed") => 0,
            Some("flaky") => 1,
            Some("not run") => 2,
            _ => 3,
        });
    }
    out.extend(unmatched);
    out
}
