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
    ]
}

fn check(src: &str) {
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

#[test]
fn binary_operators_agree_with_the_evaluator() {
    let names: Vec<&str> = operands().iter().map(|o| o.0).collect();
    for a in &names {
        for b in &names {
            for op in ["+", "-", "*", "/", "<", "<=", ">", ">=", "==", "!=", "and", "or"] {
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
}
