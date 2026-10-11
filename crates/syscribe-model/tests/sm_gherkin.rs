//! Gherkin extraction robustness for executable scenarios (REQ-TRS-SMSCEN-001, review fixes).

use syscribe_model::sm_scen::{extract_scenarios, parse_step, vocabulary_findings, Cmd, Kw};

fn fenced(body: &str) -> String {
    format!("```gherkin\nFeature: f\n\n{body}```\n")
}

#[test]
fn a_multibyte_line_does_not_panic() {
    let doc = fenced("  Scenario: s\n    温度 is high\n    Given n is 5\n");
    let sc = extract_scenarios(&doc);
    assert_eq!(sc[0].steps.len(), 1);
    let _ = vocabulary_findings(&doc);
}

#[test]
fn background_steps_prefix_every_scenario() {
    let sc = extract_scenarios(&fenced("  Background:\n    Given n is 5\n\n  Scenario: a\n    Then n == 5\n\n  Scenario: b\n    Then n > 1\n"));
    assert_eq!(sc.len(), 2);
    assert!(sc.iter().all(|s| s.steps.len() == 2 && s.steps[0].text == "n is 5"));
}

#[test]
fn each_examples_table_has_its_own_header() {
    let sc = extract_scenarios(&fenced("  Scenario Outline: o\n    Then n == <v>\n\n    Examples:\n      | v |\n      | 0 |\n\n    Examples:\n      | v |\n      | 1 |\n"));
    let t: Vec<_> = sc.iter().map(|s| (s.title.clone(), s.steps[0].text.clone())).collect();
    assert_eq!(t, [("o [row 1]".to_string(), "n == 0".to_string()), ("o [row 2]".to_string(), "n == 1".to_string())]);
}

#[test]
fn an_escaped_pipe_is_part_of_the_cell() {
    let sc = extract_scenarios(&fenced("  Scenario Outline: o\n    Then s == '<v>'\n\n    Examples:\n      | v |\n      | a\\|b |\n"));
    assert_eq!(sc[0].steps[0].text, "s == 'a|b'");
}

#[test]
fn doc_strings_are_not_steps() {
    let sc = extract_scenarios(&fenced("  Scenario: s\n    Given n is 5\n    \"\"\"\n    Then n == 99\n    \"\"\"\n    Then n == 5\n"));
    assert_eq!(sc[0].steps.len(), 2);
}

#[test]
fn quoted_vocabulary_is_not_vocabulary() {
    assert_eq!(parse_step(Kw::Then, "s != \"x is in state y\""), Ok(Cmd::Expect("s != \"x is in state y\"".into())));
}

#[test]
fn empty_scenarios_and_outlines_without_rows_are_findings() {
    let f = vocabulary_findings(&fenced("  Scenario: empty\n\n  Scenario Outline: none\n    Then n == <v>\n"));
    assert_eq!(f.len(), 2, "{f:?}");
}
