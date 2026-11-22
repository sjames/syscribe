//! Robot Framework `output.xml` as a built-in results format (REQ-TRS-RESROBOT-001).
//!
//! Robot 6 (schema 4) and Robot 7 (schema 5) differ in their timestamps (`starttime`/`endtime` as
//! `20261015 01:43:41.976`, against `start`/`elapsed` as ISO 8601) and nothing else that matters here. A test is
//! joined to a TestCase by a tag, so what this module produces is one record per (TestCase, scenario) with the
//! worst verdict of the tests that map to it, and one record under a `robot:` key for every test that maps to none.

use std::collections::BTreeMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::element::RawElement;
use crate::result_join::{canonical, testcase_lookup};
use crate::result_records::{badness, Provenance, ResultRecord};
use crate::results::{one_line, Verdict};

/// One test of a Robot run, as the file says it.
#[derive(Debug, Clone, Default)]
pub struct RobotTest {
    /// The suites and the test, `Outer.Inner.Test name`.
    pub full_name: String,
    /// The test's own name.
    pub name: String,
    pub tags: Vec<String>,
    /// `PASS`, `FAIL`, `SKIP` or `NOT RUN`.
    pub status: String,
    /// The status element's text: a failure's message.
    pub message: String,
    /// Naive local timestamps as Robot wrote them, normalised to `YYYY-MM-DDTHH:MM:SS[.fff]`.
    pub start: Option<String>,
    pub end: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RobotReport {
    /// `Robot 7.4.2`, from the `generator` attribute.
    pub tool: Option<String>,
    pub tests: Vec<RobotTest>,
}

fn attr(e: &BytesStart, key: &str) -> Option<String> {
    e.attributes().flatten().find(|a| a.key.as_ref() == key.as_bytes()).map(|a| match a.normalized_value(quick_xml::XmlVersion::Implicit1_0) {
        Ok(v) => v.into_owned(),
        Err(_) => String::from_utf8_lossy(&a.value).into_owned(),
    })
}

/// Robot 6 writes `20261015 01:43:41.976`, Robot 7 `2026-10-15T01:43:41.976000`: both to the second form.
fn normalise_time(t: &str) -> Option<String> {
    let t = t.trim();
    if t.len() >= 19 && t.as_bytes()[4] == b'-' {
        return Some(t.replacen(' ', "T", 1));
    }
    let b = t.as_bytes();
    if t.len() >= 17 && b[8] == b' ' && b[..8].iter().all(u8::is_ascii_digit) {
        return Some(format!("{}-{}-{}T{}", &t[0..4], &t[4..6], &t[6..8], &t[9..]));
    }
    None
}

/// Days from 1970-01-01 of a civil date (proleptic Gregorian).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `naive` plus `secs` seconds, to the millisecond.
fn add_seconds(naive: &str, secs: f64) -> Option<String> {
    let num = |a: usize, n: usize| naive.get(a..a + n)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, s) = (num(0, 4)?, num(5, 2)?, num(8, 2)?, num(11, 2)?, num(14, 2)?, num(17, 2)?);
    let frac: f64 = naive.get(19..).filter(|f| f.starts_with('.')).and_then(|f| format!("0{f}").parse().ok()).unwrap_or(0.0);
    let total_ms = ((days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + s) as f64 + frac + secs) * 1000.0;
    let total_ms = total_ms.round() as i64;
    let (secs_total, ms) = (total_ms.div_euclid(1000), total_ms.rem_euclid(1000));
    let (cy, cm, cd) = civil_from_days(secs_total.div_euclid(86400));
    let rem = secs_total.rem_euclid(86400);
    Some(format!("{cy:04}-{cm:02}-{cd:02}T{:02}:{:02}:{:02}.{ms:03}", rem / 3600, rem % 3600 / 60, rem % 60))
}

/// `ts` (RFC 3339, with a `Z` or `±HH:MM` zone) plus `secs` seconds, in the same zone.
pub fn add_seconds_zoned(ts: &str, secs: f64) -> Option<String> {
    let zone_at = ts.rfind(['Z', 'z', '+']).or_else(|| ts.rfind('-').filter(|i| *i > 10))?;
    let (naive, zone) = ts.split_at(zone_at);
    Some(format!("{}{zone}", add_seconds(naive, secs)?))
}

enum Text {
    Tag,
    Status,
}

/// Parse a Robot Framework `output.xml`.
pub fn parse(text: &str) -> Result<RobotReport, String> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(false);
    let mut report = RobotReport::default();
    let mut stack: Vec<String> = Vec::new();
    let mut suites: Vec<String> = Vec::new();
    let mut cur: Option<(RobotTest, usize)> = None; // the open test and its depth in `stack`
    let mut collecting: Option<(Text, String)> = None;
    let mut status_attrs: Option<(String, Option<String>, Option<String>, Option<f64>)> = None;
    let mut buf = String::new();

    let finish_text = |cur: &mut Option<(RobotTest, usize)>, kind: Text, raw: &str, status: &mut Option<(String, Option<String>, Option<String>, Option<f64>)>| {
        let Some((t, _)) = cur.as_mut() else { return };
        match kind {
            Text::Tag => {
                let tag = one_line(raw);
                if !tag.is_empty() {
                    t.tags.push(tag);
                }
            }
            Text::Status => {
                if let Some((s, start, end, elapsed)) = status.take() {
                    t.status = s;
                    t.message = one_line(raw);
                    t.start = start.clone();
                    t.end = end.or_else(|| start.as_deref().zip(elapsed).and_then(|(s, e)| add_seconds(s, e)));
                }
            }
        }
    };

    loop {
        let ev = reader.read_event().map_err(|e| format!("robot: not well-formed XML at byte {}: {e}", reader.buffer_position()))?;
        let (start_el, empty) = match &ev {
            Event::Start(e) => (Some(e.clone()), false),
            Event::Empty(e) => (Some(e.clone()), true),
            _ => (None, false),
        };
        if let Some(e) = start_el {
            let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
            if stack.is_empty() {
                if name != "robot" {
                    return Err(format!("robot: not a Robot Framework result: the root element is <{name}>, expected <robot>"));
                }
                if attr(&e, "rpa").as_deref() == Some("true") {
                    return Err("robot: this is an RPA result (tasks, not tests); only test results are ingested".to_string());
                }
                report.tool = attr(&e, "generator").map(|g| g.split(" (").next().unwrap_or(&g).trim().to_string()).filter(|g| !g.is_empty());
            }
            match (name.as_str(), cur.as_ref().map(|(_, d)| stack.len() - d)) {
                ("suite", None) => suites.push(attr(&e, "name").unwrap_or_default()),
                ("test", None) if stack.last().map(String::as_str) == Some("suite") => {
                    let leaf = attr(&e, "name").unwrap_or_default();
                    let full = suites.iter().cloned().chain([leaf.clone()]).collect::<Vec<_>>().join(".");
                    cur = Some((RobotTest { full_name: full, name: leaf, ..Default::default() }, stack.len()));
                }
                // a test's own children: RF7 `<tag>` directly, older `<tags><tag>`
                ("tag", Some(1)) => collecting = Some((Text::Tag, String::new())),
                ("tag", Some(2)) if stack.last().map(String::as_str) == Some("tags") => collecting = Some((Text::Tag, String::new())),
                ("status", Some(1)) => {
                    let start = attr(&e, "start").or_else(|| attr(&e, "starttime")).and_then(|t| normalise_time(&t));
                    let end = attr(&e, "endtime").and_then(|t| normalise_time(&t));
                    let elapsed = attr(&e, "elapsed").and_then(|v| v.parse::<f64>().ok());
                    status_attrs = Some((attr(&e, "status").unwrap_or_default(), start, end, elapsed));
                    collecting = Some((Text::Status, String::new()));
                }
                _ => {}
            }
            if empty {
                // `<status .../>` and `<tag/>` have no text: finish them now
                if let Some((kind, raw)) = collecting.take() {
                    finish_text(&mut cur, kind, &raw, &mut status_attrs);
                }
            } else {
                stack.push(name);
            }
            continue;
        }
        match ev {
            Event::Text(t) => {
                if let Some((_, raw)) = collecting.as_mut() {
                    buf.clear();
                    buf.push_str(&t.decode().map_err(|e| format!("robot: unreadable text: {e}"))?);
                    raw.push_str(&buf);
                }
            }
            Event::CData(t) => {
                if let Some((_, raw)) = collecting.as_mut() {
                    raw.push_str(&String::from_utf8_lossy(&t));
                }
            }
            Event::GeneralRef(r) => {
                if let Some((_, raw)) = collecting.as_mut() {
                    let name = r.decode().map_err(|e| format!("robot: unreadable entity: {e}"))?;
                    if let Ok(Some(c)) = r.resolve_char_ref() {
                        raw.push(c);
                    } else if let Some(s) = quick_xml::escape::resolve_predefined_entity(&name) {
                        raw.push_str(s);
                    }
                }
            }
            Event::End(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                if matches!(name.as_str(), "tag" | "status") {
                    if let Some((kind, raw)) = collecting.take() {
                        finish_text(&mut cur, kind, &raw, &mut status_attrs);
                    }
                }
                stack.pop();
                if name == "test" {
                    if let Some((t, depth)) = cur.take_if(|(_, d)| *d == stack.len()) {
                        let _ = depth;
                        if t.status.is_empty() {
                            return Err(format!("robot: the test '{}' has no <status>", t.full_name));
                        }
                        report.tests.push(t);
                    }
                } else if name == "suite" {
                    suites.pop();
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if report.tests.is_empty() {
        return Err("robot: no tests found in the result file".to_string());
    }
    Ok(report)
}

/// Robot's verdict for a test, in ours. A test that never ran because a parent suite's setup failed is `blocked`
/// (Robot reports it as FAIL), one whose own setup or body failed is `fail`.
fn verdict_of(t: &RobotTest) -> Result<Verdict, String> {
    match t.status.as_str() {
        "PASS" => Ok(Verdict::Pass),
        "FAIL" if t.message.starts_with("Parent suite setup failed") => Ok(Verdict::Blocked),
        "FAIL" => Ok(Verdict::Fail),
        "SKIP" | "NOT RUN" => Ok(Verdict::NotRun),
        other => Err(format!("robot: the test '{}' has the unknown status '{other}'", t.full_name)),
    }
}

/// Apply a UTC offset (`Z` or `+HH:MM`) to a naive timestamp.
fn stamp(naive: &str, offset: &str) -> String {
    format!("{naive}{offset}")
}

/// Turn the tests into records. A tag that is a TestCase id, qualified name or `extRef` joins the test to it, as does
/// `extref:<value>` for an `extRef`; other tags (requirements, `smoke`) are ignored. A test that joins nothing is
/// returned under a `robot:` key, so the join reports it as unmatched instead of dropping it.
pub fn to_records(report: &RobotReport, elems: &[RawElement], utc_offset: &str) -> Result<Vec<ResultRecord>, String> {
    let lookup = testcase_lookup(elems);
    // (test case, scenario) -> the folded record
    let mut folded: BTreeMap<(String, Option<String>), ResultRecord> = BTreeMap::new();
    let mut unmatched: Vec<ResultRecord> = Vec::new();
    for t in &report.tests {
        let verdict = verdict_of(t)?;
        let mut targets: Vec<&RawElement> = Vec::new();
        for tag in &t.tags {
            let key = if tag.len() > 7 && tag[..7].eq_ignore_ascii_case("extref:") { &tag[7..] } else { tag.as_str() };
            if let Some(tc) = lookup(key) {
                if !targets.iter().any(|x| std::ptr::eq(*x, tc)) {
                    targets.push(tc);
                }
            }
        }
        let step = {
            let mut s = serde_json::json!({"test": t.full_name, "status": t.status});
            if !t.message.is_empty() {
                s["message"] = serde_json::Value::String(t.message.clone());
            }
            s
        };
        let started = t.start.as_deref().map(|s| stamp(s, utc_offset));
        let finished = t.end.as_deref().map(|s| stamp(s, utc_offset));
        if targets.is_empty() {
            unmatched.push(ResultRecord {
                test_case: format!("robot:{}", t.full_name),
                scenario: None,
                verdict,
                rationale: None,
                steps: vec![step],
                measurements: vec![],
                provenance: Some(Provenance { tool: report.tool.clone(), started_at: started, finished_at: finished, ..Default::default() }),
                tc_hash: None,
                req_hashes: Default::default(),
                condition: Default::default(),
            });
            continue;
        }
        for tc in targets {
            let id = canonical(tc);
            let titles = crate::results::gherkin_scenario_titles(&tc.doc);
            let scenario = titles.iter().find(|s| one_line(s) == one_line(&t.name)).cloned();
            let slot = folded.entry((id.clone(), scenario.clone())).or_insert_with(|| ResultRecord {
                test_case: id.clone(),
                scenario,
                verdict: Verdict::Pass,
                rationale: None,
                steps: vec![],
                measurements: vec![],
                provenance: Some(Provenance { tool: report.tool.clone(), ..Default::default() }),
                tc_hash: None,
                req_hashes: Default::default(),
                condition: Default::default(),
            });
            if badness(verdict) > badness(slot.verdict) {
                slot.verdict = verdict;
            }
            slot.steps.push(step.clone());
            let p = slot.provenance.as_mut().expect("set above");
            // the span of the tests folded here: earliest start, latest end (RFC 3339 with one offset sorts as text)
            if let Some(s) = &started {
                if p.started_at.as_ref().map_or(true, |o| s < o) {
                    p.started_at = Some(s.clone());
                }
            }
            if let Some(f) = &finished {
                if p.finished_at.as_ref().map_or(true, |o| f > o) {
                    p.finished_at = Some(f.clone());
                }
            }
        }
    }
    // the first failing test's name and message lead the steps, so the detail a reader needs is first
    let mut out: Vec<ResultRecord> = folded.into_values().collect();
    for r in &mut out {
        r.steps.sort_by_key(|s| match s["status"].as_str() {
            Some("FAIL") => 0,
            Some("SKIP") | Some("NOT RUN") => 1,
            _ => 2,
        });
    }
    out.extend(unmatched);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robot_6_and_7_timestamps_normalise() {
        assert_eq!(normalise_time("20261015 01:43:41.976").as_deref(), Some("2026-10-15T01:43:41.976"));
        assert_eq!(normalise_time("2026-10-15T01:43:42.716847").as_deref(), Some("2026-10-15T01:43:42.716847"));
        assert_eq!(normalise_time("yesterday"), None);
    }

    #[test]
    fn elapsed_seconds_are_added_across_midnight_and_month_ends() {
        assert_eq!(add_seconds("2026-10-15T01:43:42.716847", 0.000677).as_deref(), Some("2026-10-15T01:43:42.718"));
        assert_eq!(add_seconds("2026-12-31T23:59:59.500", 1.0).as_deref(), Some("2027-01-01T00:00:00.500"));
        assert_eq!(add_seconds("2024-02-28T23:59:59", 2.0).as_deref(), Some("2024-02-29T00:00:01.000"));
        assert_eq!(add_seconds("2026-10-15T00:00:00", 3600.0 * 30.0).as_deref(), Some("2026-10-16T06:00:00.000"));
    }

    #[test]
    fn a_non_robot_root_and_rpa_are_errors() {
        assert!(parse("<testsuite/>").unwrap_err().contains("<testsuite>"));
        assert!(parse("<robot rpa=\"true\"><suite name=\"s\"/></robot>").unwrap_err().contains("RPA"));
        assert!(parse("<robot rpa=\"false\"><suite name=\"s\"/></robot>").unwrap_err().contains("no tests"));
        assert!(parse("<robot><suite name=\"s\"><test name=\"t\"><tag>x</test></suite></robot>").is_err());
    }
}
