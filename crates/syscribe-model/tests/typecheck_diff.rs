//! REQ-TRS-STYPE-001: the static rules agree with the evaluator. For every operator and function over a spread of operand
//! types, the type checker rejects an expression exactly when the evaluator fails with a type or dimension error for some
//! truth value of the boolean operand (`and`, `or` and `if` short-circuit, so a latent error shows only for one of them),
//! with values chosen so that no domain error — division by zero, negative sqrt — can occur.

use syscribe_model::expr::{self, ExprError, Value};
use syscribe_model::typecheck::{infer, Ty};

fn operands() -> Vec<(&'static str, Value)> {
    let v = |s: &str| expr::parse(s).and_then(|e| e.eval(&|_: &str| None)).unwrap();
    vec![
        ("len", v("2 m")),
        ("len2", v("3 m")),
        ("plain", v("4")),
        ("flag", Value::Bool(true)),
        ("text", Value::Str("on".into())),
        ("vel", v("5 m / 1 s")),
        ("area", v("9 m * 1 m")),
        ("nobody", v("0")),
    ]
}

fn check(src: &str) {
    check_with(src, true)
}

/// Only the direction that must always hold: a static error is a real error (the checker may be lenient, e.g. about
/// the join of two different branch types of an `if`).
fn check_sound(src: &str) {
    check_with(src, false)
}

fn check_with(src: &str, complete: bool) {
    let ops = operands();
    let env = |n: &str| ops.iter().find(|o| o.0 == n).map(|o| Ty::of_value(&o.1)).unwrap_or(Ty::Unknown);
    let ast = expr::parse(src).unwrap_or_else(|e| panic!("{src}: {e}"));
    let stat = infer(&ast, &env);
    // the evaluator under both truth values of `flag`
    let runs: Vec<Result<Value, ExprError>> = [true, false]
        .iter()
        .map(|f| {
            let lookup = |n: &str| if n == "flag" { Some(Value::Bool(*f)) } else { ops.iter().find(|o| o.0 == n).map(|o| o.1.clone()) };
            ast.eval(&lookup)
        })
        .collect();
    let type_error = |r: &Result<Value, ExprError>| matches!(r, Err(ExprError::Type(_) | ExprError::Dimension(_)));
    match &stat {
        Ok(_) if !complete => {}
        Ok(t) => {
            for r in &runs {
                assert!(!type_error(r), "`{src}`: static Ok({t:?}) but runtime {r:?}");
                // the static result type matches the value's type
                if let Ok(v) = r {
                    match (t, v) {
                        (Ty::Bool, Value::Bool(_)) | (Ty::Str, Value::Str(_)) | (Ty::Unknown, _) | (Ty::Num(None), Value::Num(_)) => {}
                        (Ty::Num(Some(d)), Value::Num(q)) => assert_eq!(*d, q.dim, "`{src}` dimension"),
                        (t, v) => panic!("`{src}`: static type {t:?} but value {v:?}"),
                    }
                }
            }
        }
        Err(why) => assert!(runs.iter().any(type_error), "`{src}`: static Err({why}) but the evaluator accepts it for both truth values: {runs:?}"),
    }
}

/// Names the type checker knows nothing about; the evaluator is given a number, a length, a boolean and a string in turn.
fn check_unknown(src: &str) {
    let ops = operands();
    let ast = expr::parse(src).unwrap_or_else(|e| panic!("{src}: {e}"));
    let stat = infer(&ast, &|n: &str| ops.iter().find(|o| o.0 == n).map(|o| Ty::of_value(&o.1)).unwrap_or(Ty::Unknown));
    let unknowns: Vec<Value> = vec![Value::Bool(true), Value::Bool(false), Value::Str("a".into()), expr::parse("3").unwrap().eval(&|_: &str| None).unwrap(), expr::parse("3 m").unwrap().eval(&|_: &str| None).unwrap()];
    let any_ok = unknowns.iter().any(|u| {
        let lookup = |n: &str| if n == "unk" { Some(u.clone()) } else if n == "flag" { Some(Value::Bool(true)) } else { ops.iter().find(|o| o.0 == n).map(|o| o.1.clone()) };
        !matches!(ast.eval(&lookup), Err(ExprError::Type(_) | ExprError::Dimension(_)))
    });
    let all_ok = unknowns.iter().all(|u| {
        let lookup = |n: &str| if n == "unk" { Some(u.clone()) } else if n == "flag" { Some(Value::Bool(true)) } else { ops.iter().find(|o| o.0 == n).map(|o| o.1.clone()) };
        !matches!(ast.eval(&lookup), Err(ExprError::Type(_) | ExprError::Dimension(_)))
    });
    // a static error with an unknown operand must be real for at least one value of it: either the known part is wrong,
    // or (lazy `and`/`or`) the evaluator reaches it for some value
    if stat.is_err() {
        assert!(!all_ok, "`{src}`: static {stat:?} but the evaluator accepts every tried value of the unknown name (any accepted: {any_ok})");
    }
}

#[test]
fn unknown_operands_are_never_rejected_unless_the_known_part_is_wrong() {
    let names: Vec<&str> = operands().iter().map(|o| o.0).collect();
    for a in &names {
        for op in ["+", "-", "*", "/", "%", "<", "<=", ">", ">=", "==", "!=", "and", "or"] {
            check_unknown(&format!("unk {op} {a}"));
            check_unknown(&format!("{a} {op} unk"));
        }
        check_unknown(&format!("min(unk, {a})"));
        check_unknown(&format!("pow({a}, unk)"));
        check_unknown(&format!("if unk then {a} else {a}"));
    }
    check_unknown("-unk == true");
    check_unknown("not unk");
}

#[test]
fn pow_with_a_negative_literal_exponent() {
    check("pow(len, -1)");
    check("pow(len, -2)");
    check("pow(plain, -2)");
    check("pow(len, -1.5)");
    check("pow(len, -2000)");
    check("pow(flag, -2)");
}

#[test]
fn binary_operators_agree_with_the_evaluator() {
    let names: Vec<&str> = operands().iter().map(|o| o.0).collect();
    for a in &names {
        for b in &names {
            for op in ["+", "-", "*", "/", "%", "<", "<=", ">", ">=", "==", "!=", "and", "or"] {
                check(&format!("{a} {op} {b}"));
            }
        }
    }
}

#[test]
fn functions_and_unary_operators_agree_with_the_evaluator() {
    let names: Vec<&str> = operands().iter().map(|o| o.0).collect();
    for a in &names {
        check(&format!("not {a}"));
        check(&format!("-{a}"));
        for f in ["abs", "floor", "ceil", "round", "sqrt"] {
            check(&format!("{f}({a})"));
        }
        for e in ["2", "0", "1.5", "len", "flag", "plain"] {
            check(&format!("pow({a}, {e})"));
        }
        for b in &names {
            check(&format!("min({a}, {b})"));
            check(&format!("max({a}, {b})"));
        }
    }
}

#[test]
fn conditionals_agree_with_the_evaluator() {
    let names: Vec<&str> = operands().iter().map(|o| o.0).collect();
    for c in &names {
        for a in &names {
            check(&format!("if {c} then {a} else {a}"));
        }
    }
    check("if flag then len else len2");
    check("if flag then len else plain");
    for a in &names {
        for b in &names {
            check_sound(&format!("if flag then {a} else {b}"));
            check_sound(&format!("(if flag then {a} else {b}) == {a}"));
        }
    }
}
