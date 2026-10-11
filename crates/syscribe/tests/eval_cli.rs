//! TC-TRS-EXPR-001 / GH #273: `syscribe eval`.

use std::process::Command;

fn eval(args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("eval").args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).trim().to_string(), String::from_utf8_lossy(&o.stderr).trim().to_string(), o.status.code().unwrap_or(-1))
}

#[test]
fn prints_the_value_in_si_units() {
    assert_eq!(eval(&["1 + 2 * 3"]), ("7".into(), "".into(), 0));
    assert_eq!(eval(&["50 ms + 1 s"]).0, "1.05 s");
    assert_eq!(eval(&["3 V * 2 A"]).0, "6 W");
    assert_eq!(eval(&["1 s > 500 ms"]).0, "true");
}

#[test]
fn set_binds_names_and_unit_converts() {
    assert_eq!(eval(&["a + b", "--set", "a=2", "--set", "b=3"]).0, "5");
    assert_eq!(eval(&["t * 2", "--set", "t=0.5s", "--unit", "ms"]).0, "1000 ms");
    assert_eq!(eval(&["mode == Mode::auto", "--set", "mode='auto'"]).0, "true");
}

#[test]
fn json_describes_the_value() {
    let (o, _, c) = eval(&["2 * 3 V", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert_eq!(v["expression"], "2 * 3 V");
    assert_eq!(v["type"], "number");
    assert_eq!(v["value"], 6.0);
    assert_eq!(v["unit"], "V");
    let (o, _, _) = eval(&["1 < 2", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert_eq!((v["type"].as_str(), v["value"].as_bool()), (Some("boolean"), Some(true)));
}

#[test]
fn errors_exit_one_with_a_message() {
    for (args, needle) in [
        (vec!["x + 1"], "x"),
        (vec!["1 s + 1 V"], "dimension"),
        (vec!["1 +"], "parse"),
        (vec!["1 / 0"], "zero"),
        (vec!["1 s", "--unit", "V"], "dimension"),
        (vec!["1", "--set", "bad"], "--set"),
    ] {
        let (o, e, c) = eval(&args);
        assert_eq!(c, 1, "{args:?}: {o} {e}");
        assert!(e.to_lowercase().contains(needle), "{args:?}: {e}");
        assert_eq!(o, "", "{args:?}: nothing on stdout");
    }
}

#[test]
fn a_missing_expression_is_a_usage_error() {
    let (_, e, c) = eval(&[]);
    assert_eq!(c, 1);
    assert!(e.to_lowercase().contains("usage"), "{e}");
}

#[test]
fn expressions_starting_with_dash_m_are_not_taken_for_the_model_flag() {
    assert_eq!(eval(&["-max(1, 2)"]).0, "-2");
    assert_eq!(eval(&["-mass * 2", "--set", "mass=3"]).0, "-6");
    assert_eq!(eval(&["-min(3 s, 1 s)"]).0, "-1 s");
    // an explicit model flag before the command still works
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).args(["-m", "does-not-matter", "eval", "1 + 1"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "2");
}
