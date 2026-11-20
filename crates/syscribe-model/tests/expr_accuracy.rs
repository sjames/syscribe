//! Accuracy of the expression language (GH #499, extends `TC-TRS-EXPR-001`).
//!
//! Simulation, trade studies, constraints and recorded measurements all read this parser, so a wrong value is
//! worse than an error. These tests do not restate the implementation: temperatures are checked against the
//! conversion formulas and against a second route through the arithmetic, and plain arithmetic is checked against a
//! separate evaluator over thousands of generated expressions.

use syscribe_model::expr::{parse, parse_value, Expr, ExprError, Quantity, TempKind, Value};

fn eval(src: &str) -> Result<Value, ExprError> {
    parse(src)?.eval(&|_: &str| None)
}

fn quantity(src: &str) -> Quantity {
    match eval(src).unwrap_or_else(|e| panic!("{src}: {e}")) {
        Value::Num(q) => q,
        other => panic!("{src}: not a number: {other:?}"),
    }
}

fn truth(src: &str) -> bool {
    match eval(src).unwrap_or_else(|e| panic!("{src}: {e}")) {
        Value::Bool(b) => b,
        other => panic!("{src}: not a boolean: {other:?}"),
    }
}

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * a.abs().max(b.abs()).max(1.0)
}

const ZERO_C: f64 = 273.15;

// ── temperatures ─────────────────────────────────────────────────────────────

#[test]
fn a_celsius_reading_is_its_kelvin_value_whatever_its_sign() {
    for n in [-273.15, -273.0, -200.5, -100.0, -40.0, -20.5, -20.0, -1.0, -0.1, -0.0, 0.0, 0.1, 1.0, 23.4, 37.0, 100.0, 1000.0] {
        // `{n:?}` keeps `-0.0` and a fractional part; `-0.0 degC` is the freezing point, like `0 degC`.
        let q = quantity(&format!("{n:?} degC"));
        assert!(close(q.v, n + ZERO_C, 1e-12), "{n} degC = {} K, expected {}", q.v, n + ZERO_C);
        assert_eq!(q.temp, TempKind::Point, "{n} degC is an absolute temperature");
        assert_eq!(q.dim.0, [0, 0, 0, 0, 1, 0, 0], "{n} degC has the dimension of temperature");
    }
}

#[test]
fn a_fahrenheit_reading_is_its_kelvin_value_whatever_its_sign() {
    for n in [-459.67, -400.0, -100.0, -40.0, -0.5, 0.0, 32.0, 98.6, 212.0, 500.0] {
        let q = quantity(&format!("{n:?} degF"));
        let expected = (n - 32.0) * 5.0 / 9.0 + ZERO_C;
        assert!(close(q.v, expected, 1e-12), "{n} degF = {} K, expected {expected}", q.v);
        assert_eq!(q.temp, TempKind::Point);
    }
}

/// A second route to the same number: `0 degC` moved by a temperature difference, which never goes through the
/// literal parser's sign handling.
#[test]
fn a_negative_reading_equals_the_freezing_point_moved_down_by_a_difference() {
    for n in [0.5, 1.0, 20.0, 20.5, 40.0, 100.0, 273.0] {
        let literal = quantity(&format!("-{n:?} degC"));
        let routed = quantity(&format!("0 degC - {n:?} delta_degC"));
        assert!(close(literal.v, routed.v, 1e-12), "-{n} degC: {} vs {}", literal.v, routed.v);
        assert_eq!((literal.temp, routed.temp), (TempKind::Point, TempKind::Point));
    }
}

#[test]
fn the_two_scales_agree_where_they_meet() {
    assert!(truth("-40 degC == -40 degF"), "the one point where Celsius and Fahrenheit coincide");
    assert!(truth("0 degC == 32 degF"));
    assert!(truth("-17.77777777777778 degC == 0 degF"));
    assert!(truth("-273.15 degC == 0 K"));
    // 0 K from the Fahrenheit scale carries ~1e-13 K of rounding (the offsets are ~255), more than the absolute
    // floor of `==`, so it is compared with a tolerance rather than for equality.
    assert!(quantity("-459.67 degF").v.abs() < 1e-9);
    assert!(truth("-20 degC == 253.15 K"), "the example of GH #499");
    assert!(truth("-20.5 degC == 252.65 K"));
    assert!(!truth("-20 degC == -20 degF"));
}

#[test]
fn readings_below_zero_order_correctly() {
    assert!(truth("-20 degC < -10 degC"));
    assert!(truth("-10 degC > -20 degC"));
    assert!(truth("-1 degC < 0 degC"));
    assert!(truth("0 degC > -0.5 degC"));
    assert!(truth("-40 degF < -40 degC + 1 delta_degC"));
    assert!(!truth("-20 degC >= -10 degC"));
    assert!(truth("-20 degC <= -20 degC"));
    assert!(truth("min(-5 degC, -20 degC) == -20 degC"));
    assert!(truth("max(-5 degC, -20 degC) == -5 degC"));
}

#[test]
fn differences_between_readings_are_differences_not_readings() {
    let d = quantity("(-20 degC) - (-30 degC)");
    assert!(close(d.v, 10.0, 1e-9) && d.temp == TempKind::Delta, "{d:?}");
    let d = quantity("(10 degC) - (-10 degC)");
    assert!(close(d.v, 20.0, 1e-9) && d.temp == TempKind::Delta, "{d:?}");
    let p = quantity("(-20 degC) + 5 delta_degC");
    assert!(close(p.v, ZERO_C - 15.0, 1e-9) && p.temp == TempKind::Point, "{p:?}");
    let p = quantity("-20 degC - 5 delta_degC");
    assert!(close(p.v, ZERO_C - 25.0, 1e-9) && p.temp == TempKind::Point, "{p:?}");
    assert!(truth("(-20 degC) - (-30 degC) == 10 delta_degC"));
}

#[test]
fn a_temperature_below_absolute_zero_is_an_error_not_a_negative_kelvin() {
    for src in ["-273.16 degC", "-300 degC", "-1000 degC", "-459.68 degF", "-500 degF"] {
        match eval(src) {
            Err(ExprError::Parse(m)) => assert!(m.contains("below absolute zero"), "{src}: {m}"),
            other => panic!("{src}: expected a below-absolute-zero error, got {other:?}"),
        }
    }
    // The boundary itself is a temperature.
    assert!(quantity("-273.15 degC").v.abs() < 1e-9);
    assert!(quantity("-459.67 degF").v.abs() < 1e-9);
}

#[test]
fn negating_a_temperature_expression_stays_an_error() {
    for src in ["-(20 degC)", "-(-20 degC)", "--20 degC", "- -20 degC", "-(0 degC)"] {
        assert!(matches!(eval(src), Err(ExprError::Dimension(_))), "{src}: {:?}", eval(src));
    }
    // A name carries no unit, so a unit after it is not a literal.
    assert!(matches!(parse("-x degC"), Err(ExprError::Parse(_))));
    // A reading is not a plain number and a plain number is not a reading.
    assert!(matches!(eval("-20 degC + 5"), Err(ExprError::Dimension(_))));
    assert!(matches!(eval("5 - -20 degC"), Err(ExprError::Dimension(_))));
    assert!(matches!(eval("-20 degC + -30 degC"), Err(ExprError::Dimension(_))), "two absolute temperatures do not add");
    assert!(matches!(eval("-20 degC * 2"), Err(ExprError::Dimension(_))));
}

#[test]
fn the_sign_may_be_separated_from_the_number() {
    assert!(close(quantity("- 20 degC").v, ZERO_C - 20.0, 1e-12));
    assert!(close(quantity("-  20.5degC").v, ZERO_C - 20.5, 1e-12));
    assert!(close(quantity("-2e1 degC").v, ZERO_C - 20.0, 1e-12), "an exponent belongs to the number");
}

#[test]
fn recorded_values_read_negative_readings() {
    for (text, kelvin) in [("-20 degC", ZERO_C - 20.0), ("-20.5 degC", ZERO_C - 20.5), ("  -40 degF ", (-40.0 - 32.0) * 5.0 / 9.0 + ZERO_C), ("-273.15 degC", 0.0)] {
        match parse_value(text) {
            Some(Value::Num(q)) => assert!(close(q.v, kelvin, 1e-9) && q.temp == TempKind::Point, "{text}: {q:?}"),
            other => panic!("{text}: {other:?}"),
        }
    }
    assert_eq!(parse_value("-300 degC"), None, "below absolute zero is not a quantity");
    assert_eq!(parse_value("-x degC"), None);
}

// ── nothing else changes ─────────────────────────────────────────────────────

/// A compiled machine's hash is taken over its guard text and tree, and the evidence recorded against it names
/// that hash. A negative literal that is not an offset-scale temperature must therefore parse to exactly the tree
/// it always did.
#[test]
fn other_negative_literals_keep_their_tree() {
    for src in ["-5", "-5 ms", "-5 K", "-5 delta_degC", "-5 delta_degF", "-1.5e2", "-0", "-(20 degC)", "-x"] {
        assert!(matches!(parse(src), Ok(Expr::Unary("-", _))), "{src} must stay a negation: {:?}", parse(src));
    }
    assert!(matches!(parse("-20 degC"), Ok(Expr::Lit(Value::Num(_)))), "-20 degC is one literal");
    assert!(matches!(parse("-5 degF"), Ok(Expr::Lit(Value::Num(_)))));
    assert!(matches!(parse("5 - 3"), Ok(Expr::Bin("-", _, _))), "a binary minus is still a binary minus");
    assert!(matches!(parse("5 -3"), Ok(Expr::Bin("-", _, _))));
    assert!(matches!(parse("a-20 degC"), Ok(Expr::Bin("-", _, _))));
}

#[test]
fn a_minus_before_a_unit_literal_means_what_it_always_meant() {
    assert!(close(quantity("-5 ms").v, -0.005, 1e-12));
    assert!(close(quantity("-3 V * -2 A").v, 6.0, 1e-12));
    assert!(close(quantity("-1 kWh").v, -3.6e6, 1e-12));
    assert!(close(quantity("-90 deg").v, -std::f64::consts::FRAC_PI_2, 1e-12));
    assert!(close(quantity("-1 km + 1 m").v, -999.0, 1e-12));
    assert!(close(quantity("-2 s / -4 s").v, 0.5, 1e-12));
    assert!(close(quantity("-20 K").v, -20.0, 1e-12));
    assert!(close(quantity("-20 delta_degC").v, -20.0, 1e-12));
    assert!(truth("-3 ms < 0 ms"));
}

// ── arithmetic against a separate evaluator ──────────────────────────────────

#[test]
fn precedence_associativity_and_signs() {
    let cases: &[(&str, f64)] = &[
        ("-2 * 3", -6.0),
        ("2 * -3", -6.0),
        ("-2 * -3", 6.0),
        ("2 - -3", 5.0),
        ("2 - - 3", 5.0),
        ("--2", 2.0),
        ("---2", -2.0),
        ("-(-(3))", 3.0),
        ("-2 - -2", 0.0),
        ("2 - 3 * 4", -10.0),
        ("(2 - 3) * 4", -4.0),
        ("2 - 3 - 4", -5.0),
        ("2 - (3 - 4)", 3.0),
        ("8 / 2 / 2", 2.0),
        ("8 / (2 / 2)", 8.0),
        ("2 * 3 % 4", 2.0),
        ("-7 % 4", -3.0),
        ("7 % -4", 3.0),
        ("10 % 3 * 2", 2.0),
        ("1 - 1 + 1", 1.0),
        ("-.5", -0.5),
        ("-1.5e2", -150.0),
        ("-1.5e-2", -0.015),
        ("1e3 * 1e-3", 1.0),
        ("2 * 3 + 4 * 5", 26.0),
        ("2 * (3 + 4) * 5", 70.0),
        ("-2 * -3 - -4", 10.0),
        ("1 + 2 - 3 + 4 - 5", -1.0),
        ("abs(-3) - abs(3)", 0.0),
        ("-abs(-3)", -3.0),
        ("max(-1, -2, -3)", -1.0),
        ("min(-1, -2, -3)", -3.0),
        ("floor(-2.5)", -3.0),
        ("ceil(-2.5)", -2.0),
        ("round(-2.5)", -3.0),
        ("pow(-2, 3)", -8.0),
        ("pow(-2, 2)", 4.0),
        ("if -1 < 0 then -1 else 1", -1.0),
        ("if not (-1 < 0) then -1 else 1", 1.0),
        ("1 + if 2 > 1 then 10 else 20", 11.0),
    ];
    for (src, want) in cases {
        let got = quantity(src).v;
        assert!(got == *want || close(got, *want, 1e-12), "{src} = {got}, expected {want}");
    }
}

#[test]
fn comparisons_and_logic_with_signs() {
    for (src, want) in [
        ("-1 < 0", true),
        ("-1 > 0", false),
        ("-2 < -1", true),
        ("-1 == -1", true),
        ("-1 != 1", true),
        ("-0 == 0", true),
        ("-1 <= -1 and -1 >= -1", true),
        ("not -1 < 0", false),
        ("-1 < 0 or 1 / 0 > 0", true),
        ("0.1 + 0.2 == 0.3", true),
        ("-0.1 - 0.2 == -0.3", true),
        ("1000 ms == 1 s", true),
        ("-1000 ms == -1 s", true),
        ("-1 s < -500 ms", true),
    ] {
        assert_eq!(truth(src), want, "{src}");
    }
}

/// A generated expression: its text, and its value computed by a separate recursive evaluator.
#[derive(Clone)]
enum G {
    Lit(u32),
    Neg(Box<G>),
    Bin(char, Box<G>, Box<G>),
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn gen(r: &mut Rng, depth: u32) -> G {
    if depth == 0 || r.below(5) == 0 {
        return G::Lit(r.below(10) as u32);
    }
    match r.below(6) {
        0 => G::Neg(Box::new(gen(r, depth - 1))),
        k => G::Bin(['+', '-', '*', '/', '%'][k as usize - 1], Box::new(gen(r, depth - 1)), Box::new(gen(r, depth - 1))),
    }
}

/// `None` where the language reports a domain error (division or remainder by zero).
fn value(g: &G) -> Option<f64> {
    match g {
        G::Lit(n) => Some(*n as f64),
        G::Neg(a) => value(a).map(|v| -v),
        G::Bin(op, a, b) => {
            let (x, y) = (value(a)?, value(b)?);
            match op {
                '+' => Some(x + y),
                '-' => Some(x - y),
                '*' => Some(x * y),
                '/' => (y != 0.0).then(|| x / y),
                _ => (y != 0.0).then(|| x % y),
            }
        }
    }
}

fn prec(g: &G) -> u8 {
    match g {
        G::Bin('+' | '-', ..) => 1,
        G::Bin(..) => 2,
        G::Neg(_) => 3,
        G::Lit(_) => 4,
    }
}

/// Every operand parenthesised: precedence plays no part.
fn full(g: &G) -> String {
    match g {
        G::Lit(n) => n.to_string(),
        G::Neg(a) => format!("-({})", full(a)),
        G::Bin(op, a, b) => format!("({} {op} {})", full(a), full(b)),
    }
}

/// Parentheses only where the grammar needs them, so the parser's precedence and associativity decide the value.
fn minimal(g: &G) -> String {
    let wrap = |c: &G, need: bool| if need { format!("({})", minimal(c)) } else { minimal(c) };
    match g {
        G::Lit(n) => n.to_string(),
        G::Neg(a) => format!("-{}", wrap(a, prec(a) < 3)),
        G::Bin(op, a, b) => {
            let p = prec(g);
            let left_assoc_needs = |c: &G| prec(c) < p;
            let right_needs = |c: &G| prec(c) < p || (prec(c) == p && matches!(op, '-' | '/' | '%'));
            // `+` and `*` associate, but `a + (b - c)` is the same value either way only in exact arithmetic;
            // keep the tree's grouping so the floating-point result is identical.
            let right_needs = |c: &G| right_needs(c) || (prec(c) == p);
            format!("{} {op} {}", wrap(a, left_assoc_needs(a)), wrap(b, right_needs(b)))
        }
    }
}

#[test]
fn thousands_of_generated_expressions_agree_with_a_separate_evaluator() {
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let (mut checked, mut errors) = (0, 0);
    for _ in 0..6000 {
        let depth = 1 + rng.below(5) as u32;
        let g = gen(&mut rng, depth);
        for text in [full(&g), minimal(&g)] {
            match (value(&g), eval(&text)) {
                (Some(want), Ok(Value::Num(q))) => {
                    assert!(q.v == want || close(q.v, want, 1e-12), "{text} = {}, expected {want}", q.v);
                    checked += 1;
                }
                (None, Err(ExprError::Domain(_))) => errors += 1,
                (want, got) => panic!("{text}: expected {want:?}, got {got:?}"),
            }
        }
    }
    assert!(checked > 5000, "the generator must produce mostly valid expressions ({checked})");
    assert!(errors > 50, "and some division by zero, to check those are errors ({errors})");
}

/// Every sign of every offset-scale conversion, against the formula, over a sweep of readings.
#[test]
fn a_sweep_of_readings_in_both_scales_matches_the_conversion_formulas() {
    let mut n = -273.0_f64;
    while n <= 1500.0 {
        let c = quantity(&format!("{n:?} degC")).v;
        assert!(close(c, n + ZERO_C, 1e-12), "{n} degC = {c}");
        let f = n + 100.0; // a Fahrenheit reading, always above -459.67 here
        let k = quantity(&format!("{f:?} degF")).v;
        assert!(close(k, (f - 32.0) * 5.0 / 9.0 + ZERO_C, 1e-12), "{f} degF = {k}");
        // The reading and its difference route agree.
        let routed = quantity(&format!("0 degC + {n:?} delta_degC")).v;
        assert!(close(c, routed, 1e-12), "{n}: {c} vs {routed}");
        n += 37.3;
    }
}
