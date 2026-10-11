//! Static types and dimensions of executable expressions (REQ-TRS-STYPE-001): the rules of the evaluator in `expr`, applied
//! to the types of variables instead of their values, so an expression the evaluator would reject whatever the values is
//! found at validation time. A name that is not a variable has the unknown type, which is compatible with everything:
//! an expression is rejected only when the error is certain.

use crate::expr::{Expr, Value};
use crate::ir::{Action, IrExpr, Machine, Trigger};
use crate::units::Dim;

pub type Diag = (&'static str, String);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ty {
    Bool,
    Str,
    /// A number; the dimension when it is known.
    Num(Option<Dim>),
    Unknown,
}

impl Ty {
    pub fn of_value(v: &Value) -> Ty {
        match v {
            Value::Num(q) => Ty::Num(Some(q.dim)),
            Value::Bool(_) => Ty::Bool,
            Value::Str(_) => Ty::Str,
        }
    }

    fn describe(&self) -> String {
        match self {
            Ty::Bool => "a boolean".into(),
            Ty::Str => "a string".into(),
            Ty::Num(Some(d)) => format!("a number ({})", d.human()),
            Ty::Num(None) => "a number".into(),
            Ty::Unknown => "unknown".into(),
        }
    }
}

const NONE: Dim = Dim([0; 7]);

fn dim_combine(a: &Dim, b: &Dim, sign: i8) -> Option<Dim> {
    let mut o = [0i8; 7];
    for i in 0..7 {
        o[i] = a.0[i].checked_add(b.0[i].checked_mul(sign)?)?;
    }
    Some(Dim(o))
}

fn want_num(t: Ty, what: &str) -> Result<Option<Dim>, String> {
    match t {
        Ty::Num(d) => Ok(d),
        Ty::Unknown => Ok(None),
        other => Err(format!("{what} needs a number, got {}", other.describe())),
    }
}

fn want_bool(t: Ty, what: &str) -> Result<(), String> {
    match t {
        Ty::Bool | Ty::Unknown => Ok(()),
        other => Err(format!("{what} needs a boolean, got {}", other.describe())),
    }
}

fn same_dim(a: Option<Dim>, b: Option<Dim>, what: &str) -> Result<Option<Dim>, String> {
    match (a, b) {
        (Some(x), Some(y)) if x != y => Err(format!("{what} needs equal dimensions, got {} and {}", x.human(), y.human())),
        (Some(x), _) | (_, Some(x)) => Ok(Some(x)),
        _ => Ok(None),
    }
}

/// The type of `e`, or the reason the evaluator would reject it whatever the values.
pub fn infer(e: &Expr, env: &dyn Fn(&str) -> Ty) -> Result<Ty, String> {
    Ok(match e {
        Expr::Lit(v) => Ty::of_value(v),
        Expr::Name(n) => env(n),
        Expr::Unary("not", a) => {
            want_bool(infer(a, env)?, "`not`")?;
            Ty::Bool
        }
        Expr::Unary(_, a) => Ty::Num(want_num(infer(a, env)?, "unary '-'")?),
        Expr::Bin(op @ ("and" | "or"), a, b) => {
            want_bool(infer(a, env)?, &format!("`{op}`"))?;
            want_bool(infer(b, env)?, &format!("`{op}`"))?;
            Ty::Bool
        }
        Expr::Bin(op @ ("==" | "!=" | "<" | "<=" | ">" | ">="), a, b) => {
            let (ta, tb) = (infer(a, env)?, infer(b, env)?);
            let ordering = !matches!(*op, "==" | "!=");
            match (ta, tb) {
                (Ty::Num(x), Ty::Num(y)) => {
                    if let (Some(x), Some(y)) = (x, y) {
                        if x != y {
                            return Err(format!("cannot compare {} with {}", x.human(), y.human()));
                        }
                    }
                }
                (Ty::Unknown, other) | (other, Ty::Unknown) => {
                    if ordering {
                        want_num(other, &format!("'{op}'"))?;
                    }
                }
                (x, y) if x == y && !ordering => {}
                (x, y) if ordering => return Err(format!("'{op}' needs two numbers, got {} and {}", x.describe(), y.describe())),
                (x, y) => return Err(format!("cannot compare {} with {}", x.describe(), y.describe())),
            }
            Ty::Bool
        }
        Expr::Bin(op @ ("+" | "-" | "%"), a, b) => {
            let (x, y) = (want_num(infer(a, env)?, &format!("'{op}'"))?, want_num(infer(b, env)?, &format!("'{op}'"))?);
            Ty::Num(same_dim(x, y, &format!("'{op}'"))?)
        }
        Expr::Bin(op, a, b) => {
            // `*` and `/`
            let (x, y) = (want_num(infer(a, env)?, &format!("'{op}'"))?, want_num(infer(b, env)?, &format!("'{op}'"))?);
            Ty::Num(match (x, y) {
                (Some(x), Some(y)) => dim_combine(&x, &y, if *op == "*" { 1 } else { -1 }),
                _ => None,
            })
        }
        Expr::If(c, a, b) => {
            want_bool(infer(c, env)?, "`if`")?;
            let (ta, tb) = (infer(a, env)?, infer(b, env)?);
            match (ta, tb) {
                (x, y) if x == y => x,
                (Ty::Num(_), Ty::Num(_)) => Ty::Num(None),
                _ => Ty::Unknown,
            }
        }
        Expr::Call(f, args) => {
            let ts: Vec<Ty> = args.iter().map(|a| infer(a, env)).collect::<Result<_, _>>()?;
            match f.as_str() {
                "abs" | "floor" | "ceil" | "round" => Ty::Num(want_num(ts[0], f)?),
                "sqrt" => match want_num(ts[0], f)? {
                    Some(d) if d.0.iter().any(|x| x % 2 != 0) => return Err(format!("sqrt needs even dimension exponents, got {}", d.human())),
                    Some(d) => Ty::Num(Some(Dim(d.0.map(|x| x / 2)))),
                    None => Ty::Num(None),
                },
                "pow" => {
                    let base = want_num(ts[0], f)?;
                    let exp = want_num(ts[1], f)?;
                    if exp.is_some_and(|d| d != NONE) {
                        return Err("pow needs a dimensionless integer exponent (|n| <= 1024)".into());
                    }
                    // a negative literal is parsed as a unary minus over a literal
                    let lit = match &args[1] {
                        Expr::Lit(Value::Num(q)) => Some(*q),
                        Expr::Unary("-", inner) => match &**inner {
                            Expr::Lit(Value::Num(q)) => Some(crate::expr::Quantity { v: -q.v, dim: q.dim }),
                            _ => None,
                        },
                        _ => None,
                    };
                    match &lit {
                        Some(q) => {
                            if q.v.fract() != 0.0 || q.v.abs() > 1024.0 {
                                return Err("pow needs a dimensionless integer exponent (|n| <= 1024)".into());
                            }
                            let n = q.v as i32;
                            match base {
                                Some(d) => {
                                    let mut o = [0i8; 7];
                                    for (o, x) in o.iter_mut().zip(d.0) {
                                        match i8::try_from(i32::from(x) * n) {
                                            Ok(v) => *o = v,
                                            Err(_) => return Ok(Ty::Num(None)),
                                        }
                                    }
                                    Ty::Num(Some(Dim(o)))
                                }
                                None => Ty::Num(None),
                            }
                        }
                        None => Ty::Num(None),
                    }
                }
                _ => {
                    // min / max
                    let mut d: Option<Dim> = None;
                    for t in &ts {
                        d = same_dim(d, want_num(*t, f)?, f)?;
                    }
                    Ty::Num(d)
                }
            }
        }
    })
}

/// Check an expression of the IR against the types of the machine's variables.
fn infer_ir(m: &Machine, e: &IrExpr) -> Result<Ty, String> {
    let env = |n: &str| -> Ty {
        match e.refs.iter().find(|r| r.name == n) {
            Some(r) if r.kind == "var" => r.slot.and_then(|s| m.variables.get(s)).map(|v| Ty::of_value(&v.value)).unwrap_or(Ty::Unknown),
            _ => Ty::Unknown,
        }
    };
    infer(&e.ast, &env).map_err(|why| format!("`{}`: {why}", e.src))
}

fn check_bool(m: &Machine, e: &IrExpr, ctx: &str, what: &str, out: &mut Vec<Diag>) {
    match infer_ir(m, e) {
        Ok(Ty::Bool | Ty::Unknown) => {}
        Ok(other) => out.push(("E935", format!("{ctx}: {what} `{}` is {}, not a boolean", e.src, other.describe()))),
        Err(why) => out.push(("E935", format!("{ctx}: {what} {why}"))),
    }
}

fn check_actions(m: &Machine, acts: &[Action], ctx: &str, out: &mut Vec<Diag>) {
    for a in acts {
        match a {
            Action::Assign { var, expr } => match infer_ir(m, expr) {
                Err(why) => out.push(("E935", format!("{ctx}: assignment {why}"))),
                Ok(rhs) => {
                    let v = &m.variables[*var];
                    let lhs = Ty::of_value(&v.value);
                    let bad = match (lhs, rhs) {
                        (Ty::Num(Some(a)), Ty::Num(Some(b))) if a != b => Some(format!("{} to {}", b.human(), a.human())),
                        (Ty::Num(_), Ty::Bool | Ty::Str) | (Ty::Bool, Ty::Num(_) | Ty::Str) | (Ty::Str, Ty::Num(_) | Ty::Bool) => Some(format!("{} to {}", rhs.describe(), lhs.describe())),
                        _ => None,
                    };
                    if let Some(b) = bad {
                        out.push(("E935", format!("{ctx}: `{} := {}` assigns {b}", v.name, expr.src)));
                    }
                }
            },
            Action::Send { payload, .. } => {
                for (k, e) in payload {
                    if let Err(why) = infer_ir(m, e) {
                        out.push(("E935", format!("{ctx}: payload '{k}' {why}")));
                    }
                }
            }
            Action::Behavior { .. } => {}
        }
    }
}

/// `E935` findings of a lowered machine.
pub fn machine_diags(m: &Machine) -> Vec<Diag> {
    let mut out = Vec::new();
    check_actions(m, &m.initial_actions, "initialEffect", &mut out);
    for s in &m.states {
        check_actions(m, &s.entry, &format!("state '{}' entry", s.path), &mut out);
        check_actions(m, &s.exit, &format!("state '{}' exit", s.path), &mut out);
    }
    for t in &m.transitions {
        let ctx = format!("transition '{}' → '{}'", m.states[t.source].path, t.target.map(|x| m.states[x].path.as_str()).unwrap_or("(internal)"));
        if let Some(g) = &t.guard {
            check_bool(m, g, &ctx, "guard", &mut out);
        }
        if let Trigger::Change { cond } = &t.trigger {
            check_bool(m, cond, &ctx, "`when`", &mut out);
        }
        check_actions(m, &t.actions, &ctx, &mut out);
    }
    out
}
