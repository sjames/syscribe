//! TC-TRS-EXPR-001 / GH #273: the typed expression language.

use std::collections::HashMap;

use syscribe_model::expr::{format_value, parse, parse_value, ExprError, Value};

fn env(pairs: &[(&str, &str)]) -> HashMap<String, Value> {
    pairs.iter().map(|(k, v)| (k.to_string(), parse_value(v).unwrap_or_else(|| panic!("value {v}")))).collect()
}

fn eval_in(src: &str, vars: &[(&str, &str)]) -> Result<Value, ExprError> {
    let e = env(vars);
    parse(src)?.eval(&|n: &str| e.get(n).cloned())
}

fn ok(src: &str) -> Value {
    eval_in(src, &[]).unwrap_or_else(|e| panic!("{src}: {e}"))
}

fn num(src: &str) -> f64 {
    match ok(src) {
        Value::Num(q) => q.v,
        other => panic!("{src}: not a number: {other:?}"),
    }
}

fn truth(src: &str) -> bool {
    match ok(src) {
        Value::Bool(b) => b,
        other => panic!("{src}: not a boolean: {other:?}"),
    }
}

#[test]
fn arithmetic_follows_precedence() {
    assert_eq!(num("1 + 2 * 3"), 7.0);
    assert_eq!(num("(1 + 2) * 3"), 9.0);
    assert_eq!(num("10 / 4"), 2.5);
    assert_eq!(num("7 % 4"), 3.0);
    assert_eq!(num("-(2 + 3)"), -5.0);
    assert_eq!(num("2 - 3 - 4"), -5.0, "left associative");
    assert_eq!(num("1.5e2 + 1"), 151.0);
}

#[test]
fn boolean_logic_and_comparison() {
    assert!(truth("true and not false"));
    assert!(truth("1 < 2 and 3 >= 3"));
    assert!(truth("1 > 2 or 2 == 2"));
    assert!(truth("1 != 2 && !(2 < 1)"));
    assert!(!truth("true and false or false"));
    assert!(truth("false and (1 / 0 > 0) or true"), "and short-circuits");
    assert!(truth("true or (1 / 0 > 0)"), "or short-circuits");
}

#[test]
fn conditional_and_functions() {
    assert_eq!(num("if 2 > 1 then 10 else 20"), 10.0);
    assert_eq!(num("max(1, 3, 2)"), 3.0);
    assert_eq!(num("min(4, 2)"), 2.0);
    assert_eq!(num("abs(-2.5)"), 2.5);
    assert_eq!(num("sqrt(16)"), 4.0);
    assert_eq!(num("floor(2.7) + ceil(2.1) + round(2.5)"), 8.0);
    assert_eq!(num("pow(2, 10)"), 1024.0);
    let e = eval_in("if x > 0 then x else -x", &[("x", "-3")]).unwrap();
    assert_eq!(e, parse_value("3").unwrap());
}

#[test]
fn units_scale_and_dimension() {
    assert!((num("50 ms + 1 s") - 1.05).abs() < 1e-12, "mixed prefixes add in SI");
    assert!((num("3.3V") - 3.3).abs() < 1e-12, "unit directly after the number");
    assert!((num("1 kWh") - 3.6e6).abs() < 1e-6);
    assert!((num("90 deg") - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    assert!((num("2 min") - 120.0).abs() < 1e-12);
    assert!((num("100 km / 2 h") - 100_000.0 / 7200.0).abs() < 1e-9);
    assert!(truth("1 s > 500 ms"));
    assert!(truth("1000 ms == 1 s"));
    let p = ok("3 V * 2 A");
    assert_eq!(format_value(&p), "6 W");
    assert_eq!(format_value(&ok("sqrt(9 m * 1 m)")), "3 m");
    assert_eq!(format_value(&ok("pow(2 m, 2)")), "4 m^2");
}

#[test]
fn incompatible_dimensions_and_types_are_errors() {
    assert!(matches!(parse("1 s + 1 V").unwrap().eval(&|_: &str| None), Err(ExprError::Dimension(_))));
    assert!(matches!(parse("1 s < 5").unwrap().eval(&|_: &str| None), Err(ExprError::Dimension(_))));
    assert!(matches!(parse("true + 1").unwrap().eval(&|_: &str| None), Err(ExprError::Type(_))));
    assert!(matches!(parse("'a' < 'b'").unwrap().eval(&|_: &str| None), Err(ExprError::Type(_))));
    assert!(matches!(parse("1 == true").unwrap().eval(&|_: &str| None), Err(ExprError::Type(_))));
    assert!(matches!(parse("not 3").unwrap().eval(&|_: &str| None), Err(ExprError::Type(_))));
    assert!(matches!(parse("sqrt(2 m)").unwrap().eval(&|_: &str| None), Err(ExprError::Dimension(_))));
}

#[test]
fn domain_errors_are_not_infinities() {
    for src in ["1 / 0", "1 % 0", "sqrt(-1)", "pow(10, 400)", "pow(0, -1)"] {
        assert!(matches!(parse(src).unwrap().eval(&|_: &str| None), Err(ExprError::Domain(_))), "{src}");
    }
}

#[test]
fn names_dotted_qualified_and_unbound() {
    assert!(eval_in("gps.fix >= 1", &[("gps.fix", "2")]).unwrap() == Value::Bool(true));
    assert!(eval_in("Features::Display.size > 150 mm", &[("Features::Display.size", "8 in")]).unwrap() == Value::Bool(true), "8 in = 203.2 mm");
    assert_eq!(eval_in("x + 1", &[]).unwrap_err(), ExprError::Unbound("x".to_string()));
    assert_eq!(eval_in("a.b.c", &[]).unwrap_err(), ExprError::Unbound("a.b.c".to_string()));
}

#[test]
fn enumeration_literals_and_strings() {
    assert!(eval_in("armStatus == ArmStatus::disarmed", &[("armStatus", "ArmStatus::disarmed")]).unwrap() == Value::Bool(true));
    assert!(eval_in("armStatus == ArmStatus::armed", &[("armStatus", "ArmStatus::disarmed")]).unwrap() == Value::Bool(false));
    assert!(eval_in("mode == 'auto'", &[("mode", "'auto'")]).unwrap() == Value::Bool(true));
    assert!(eval_in("mode == Mode::auto", &[("mode", "'auto'")]).unwrap() == Value::Bool(true), "qualified literal vs plain string by last segment");
    assert!(eval_in("mode != \"manual\"", &[("mode", "'auto'")]).unwrap() == Value::Bool(true));
}

#[test]
fn parse_errors_name_the_problem() {
    for src in ["1 +", "(1", "1 2", "foo(", "1 = 2", "", "3 $ 4", "if 1 then 2", "max()"] {
        assert!(matches!(parse(src), Err(ExprError::Parse(_))), "{src:?} should be a parse error: {:?}", parse(src).map(|_| ()));
    }
}

#[test]
fn free_names_are_listed_in_order_without_duplicates() {
    let e = parse("a.b + c * a.b > Mode::x or d").unwrap();
    assert_eq!(e.names(), vec!["a.b".to_string(), "c".to_string(), "Mode::x".to_string(), "d".to_string()], "every identifier, in first-use order; the caller decides what an unbound qualified one means");
}

#[test]
fn parse_value_reads_bindings() {
    assert_eq!(format_value(&parse_value("50ms").unwrap()), "0.05 s");
    assert_eq!(format_value(&parse_value("-3.5 V").unwrap()), "-3.5 V");
    assert_eq!(parse_value("true"), Some(Value::Bool(true)));
    assert_eq!(parse_value("'x'"), Some(Value::Str("x".to_string())));
    assert_eq!(parse_value("ArmStatus::armed"), Some(Value::Str("ArmStatus::armed".to_string())));
    assert_eq!(format_value(&parse_value("12").unwrap()), "12");
    assert!(parse_value("1 +").is_none());
    assert!(parse_value("").is_none());
}

#[test]
fn numbers_print_without_noise() {
    assert_eq!(format_value(&ok("0.1 + 0.2")), "0.3");
    assert_eq!(format_value(&ok("1 / 3")), "0.333333333333333");
    assert_eq!(format_value(&ok("1e15 * 10")), "1e16");
    assert_eq!(format_value(&ok("0")), "0");
}

#[test]
fn review_regressions() {
    // overflowing literals are parse errors, never infinities
    assert!(matches!(parse("1e999"), Err(ExprError::Parse(_))));
    assert_eq!(parse_value("1e999"), None);
    // float noise around zero compares equal
    assert!(truth("0.1 + 0.2 - 0.3 == 0"));
    assert!(!truth("0.1 + 0.2 - 0.3 > 0"));
    assert!(truth("1 pF > 0 F"), "a picofarad is not zero");
    // 15 significant digits at both ends of the range
    assert_eq!(format_value(&ok("1 ns * 1.23456789")), "1.23456789e-9 s");
    assert_eq!(format_value(&ok("pow(2, 40)")), "1099511627776");
    assert_eq!(format_value(&ok("1.5e-3")), "0.0015");
    assert_eq!(format_value(&ok("-2.5e20")), "-2.5e20");
    // units: Ohm sign, both micro signs, prefixed bar
    assert!((num("1 kΩ") - 1000.0).abs() < 1e-9);
    assert!((num("5 μs") - 5e-6).abs() < 1e-18);
    assert!((num("5 µs") - 5e-6).abs() < 1e-18);
    assert!((num("2 mbar") - 200.0).abs() < 1e-9);
    // `if` is an operand
    assert_eq!(num("1 + if 2 > 1 then 10 else 20"), 11.0);
    assert!(truth("true and if false then false else true"));
    // underflow is flagged like overflow
    assert!(matches!(parse("pow(1e-200, 2)").unwrap().eval(&|_: &str| None), Err(ExprError::Domain(_))));
}
