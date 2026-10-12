//! The typed expression language of the execution engine (GH #273, REQ-TRS-EXPR-001).
//!
//! One deterministic, side-effect free language for guards, effects, constraints and calculations:
//! numbers carry a dimension (seven SI base exponents) and are held in SI base units, booleans,
//! and strings (which also hold enumeration literals). No clock, randomness or environment.

use std::fmt;

use crate::units::Dim;

/// A dimensioned number in SI base units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    pub v: f64,
    pub dim: Dim,
}

impl Quantity {
    pub fn scalar(v: f64) -> Self {
        Quantity { v, dim: NONE }
    }
}

const NONE: Dim = Dim([0; 7]);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(Quantity),
    Bool(bool),
    Str(String),
}

impl Value {
    pub fn kind(&self) -> &'static str {
        match self {
            Value::Num(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Str(_) => "string",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprError {
    Parse(String),
    Unbound(String),
    Dimension(String),
    Type(String),
    Domain(String),
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprError::Parse(m) => write!(f, "parse error: {m}"),
            ExprError::Unbound(n) => write!(f, "unbound name '{n}'"),
            ExprError::Dimension(m) => write!(f, "dimension error: {m}"),
            ExprError::Type(m) => write!(f, "type error: {m}"),
            ExprError::Domain(m) => write!(f, "domain error: {m}"),
        }
    }
}

impl std::error::Error for ExprError {}

// ── Units ────────────────────────────────────────────────────────────────────

const fn dim(l: i8, m: i8, t: i8, i: i8, th: i8, n: i8, j: i8) -> Dim {
    Dim([l, m, t, i, th, n, j])
}

/// Base symbols that take SI prefixes: (symbol, long name, scale to SI base, dimension).
const PREFIXABLE: &[(&str, &str, f64, Dim)] = &[
    ("m", "metre", 1.0, dim(1, 0, 0, 0, 0, 0, 0)),
    ("g", "gram", 1e-3, dim(0, 1, 0, 0, 0, 0, 0)),
    ("s", "second", 1.0, dim(0, 0, 1, 0, 0, 0, 0)),
    ("A", "ampere", 1.0, dim(0, 0, 0, 1, 0, 0, 0)),
    ("K", "kelvin", 1.0, dim(0, 0, 0, 0, 1, 0, 0)),
    ("mol", "mole", 1.0, dim(0, 0, 0, 0, 0, 1, 0)),
    ("cd", "candela", 1.0, dim(0, 0, 0, 0, 0, 0, 1)),
    ("N", "newton", 1.0, dim(1, 1, -2, 0, 0, 0, 0)),
    ("Pa", "pascal", 1.0, dim(-1, 1, -2, 0, 0, 0, 0)),
    ("J", "joule", 1.0, dim(2, 1, -2, 0, 0, 0, 0)),
    ("W", "watt", 1.0, dim(2, 1, -3, 0, 0, 0, 0)),
    ("V", "volt", 1.0, dim(2, 1, -3, -1, 0, 0, 0)),
    ("ohm", "ohm", 1.0, dim(2, 1, -3, -2, 0, 0, 0)),
    ("F", "farad", 1.0, dim(-2, -1, 4, 2, 0, 0, 0)),
    ("H", "henry", 1.0, dim(2, 1, -2, -2, 0, 0, 0)),
    ("C", "coulomb", 1.0, dim(0, 0, 1, 1, 0, 0, 0)),
    ("Hz", "hertz", 1.0, dim(0, 0, -1, 0, 0, 0, 0)),
    ("L", "litre", 1e-3, dim(3, 0, 0, 0, 0, 0, 0)),
    ("Wh", "wattHour", 3600.0, dim(2, 1, -2, 0, 0, 0, 0)),
    ("Ω", "Ohm", 1.0, dim(2, 1, -3, -2, 0, 0, 0)),
    ("bar", "bar", 1e5, dim(-1, 1, -2, 0, 0, 0, 0)),
];

/// Units that never take a prefix.
const FIXED: &[(&str, f64, Dim)] = &[
    ("min", 60.0, dim(0, 0, 1, 0, 0, 0, 0)),
    ("minute", 60.0, dim(0, 0, 1, 0, 0, 0, 0)),
    ("h", 3600.0, dim(0, 0, 1, 0, 0, 0, 0)),
    ("hour", 3600.0, dim(0, 0, 1, 0, 0, 0, 0)),
    ("t", 1000.0, dim(0, 1, 0, 0, 0, 0, 0)),
    ("tonne", 1000.0, dim(0, 1, 0, 0, 0, 0, 0)),
    ("rpm", 1.0 / 60.0, dim(0, 0, -1, 0, 0, 0, 0)),
    ("deg", std::f64::consts::PI / 180.0, NONE),
    ("degree", std::f64::consts::PI / 180.0, NONE),
    ("rad", 1.0, NONE),
    ("radian", 1.0, NONE),
    ("kph", 1.0 / 3.6, dim(1, 0, -1, 0, 0, 0, 0)),
    ("in", 0.0254, dim(1, 0, 0, 0, 0, 0, 0)),
    ("inch", 0.0254, dim(1, 0, 0, 0, 0, 0, 0)),
    ("ft", 0.3048, dim(1, 0, 0, 0, 0, 0, 0)),
    ("percent", 0.01, NONE),
];

const PREFIXES: &[(&str, &str, i32)] = &[
    ("G", "giga", 9),
    ("M", "mega", 6),
    ("k", "kilo", 3),
    ("m", "milli", -3),
    ("u", "micro", -6),
    ("µ", "micro", -6),
    ("μ", "micro", -6),
    ("n", "nano", -9),
    ("p", "pico", -12),
    ("T", "tera", 12),
];

/// `x` to an integer power by repeated squaring: the same bits on every platform, unlike `f64::powi`, whose
/// precision the standard library leaves unspecified.
pub fn ipow(x: f64, e: i32) -> f64 {
    let mut base = x;
    let mut n = e.unsigned_abs();
    let mut acc = 1.0;
    while n > 0 {
        if n & 1 == 1 {
            acc *= base;
        }
        base *= base;
        n >>= 1;
    }
    if e < 0 {
        1.0 / acc
    } else {
        acc
    }
}

/// `unit` → (scale to SI base units, dimension).
pub fn unit_scale(unit: &str) -> Option<(f64, Dim)> {
    let unit = unit.strip_prefix("SI::").unwrap_or(unit);
    if let Some((_, s, d)) = FIXED.iter().find(|(n, _, _)| *n == unit) {
        return Some((*s, *d));
    }
    if let Some((_, _, s, d)) = PREFIXABLE.iter().find(|(sym, long, _, _)| *sym == unit || *long == unit) {
        return Some((*s, *d));
    }
    for (psym, plong, exp) in PREFIXES {
        for (rest, long) in [(unit.strip_prefix(psym), false), (unit.strip_prefix(plong), true)] {
            let Some(rest) = rest else { continue };
            if rest.is_empty() {
                continue;
            }
            if let Some((_, _, s, d)) = PREFIXABLE.iter().find(|(sym, lg, _, _)| if long { *lg == rest } else { *sym == rest }) {
                return Some((s * ipow(10.0, *exp), *d));
            }
        }
    }
    None
}

/// The preferred symbol of a dimension in SI base/derived units, when it has one.
fn dim_symbol(d: &Dim) -> Option<&'static str> {
    const NAMES: &[(Dim, &str)] = &[
        (dim(1, 0, 0, 0, 0, 0, 0), "m"),
        (dim(0, 1, 0, 0, 0, 0, 0), "kg"),
        (dim(0, 0, 1, 0, 0, 0, 0), "s"),
        (dim(0, 0, 0, 1, 0, 0, 0), "A"),
        (dim(0, 0, 0, 0, 1, 0, 0), "K"),
        (dim(0, 0, 0, 0, 0, 1, 0), "mol"),
        (dim(0, 0, 0, 0, 0, 0, 1), "cd"),
        (dim(1, 1, -2, 0, 0, 0, 0), "N"),
        (dim(-1, 1, -2, 0, 0, 0, 0), "Pa"),
        (dim(2, 1, -2, 0, 0, 0, 0), "J"),
        (dim(2, 1, -3, 0, 0, 0, 0), "W"),
        (dim(2, 1, -3, -1, 0, 0, 0), "V"),
        (dim(2, 1, -3, -2, 0, 0, 0), "ohm"),
        (dim(-2, -1, 4, 2, 0, 0, 0), "F"),
        (dim(2, 1, -2, -2, 0, 0, 0), "H"),
        (dim(0, 0, 1, 1, 0, 0, 0), "C"),
        (dim(0, 0, -1, 0, 0, 0, 0), "Hz"),
        (dim(2, 0, 0, 0, 0, 0, 0), "m^2"),
        (dim(3, 0, 0, 0, 0, 0, 0), "m^3"),
        (dim(1, 0, -1, 0, 0, 0, 0), "m/s"),
        (dim(1, 0, -2, 0, 0, 0, 0), "m/s^2"),
    ];
    NAMES.iter().find(|(dd, _)| dd == d).map(|(_, s)| *s)
}

/// The unit of a quantity for display: its symbol, else the dimension string, `None` if dimensionless.
pub fn unit_label(d: &Dim) -> Option<String> {
    if *d == NONE {
        None
    } else {
        Some(dim_symbol(d).map(str::to_string).unwrap_or_else(|| d.human()))
    }
}

// ── Number formatting ────────────────────────────────────────────────────────

/// A number as `eval` prints it (12 significant digits, no binary noise).
pub fn format_number(v: f64) -> String {
    fmt_num(v)
}

fn fmt_num(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    // 15 significant digits (f64 carries ~15.9): enough that exact integers print exactly and
    // binary noise (0.1 + 0.2) does not.
    let sci = format!("{:.14e}", v);
    let (mant, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let sign = if neg { "-" } else { "" };
    if !(-5..15).contains(&exp) {
        let frac = &digits[1..];
        let m = if frac.is_empty() { digits[..1].to_string() } else { format!("{}.{}", &digits[..1], frac) };
        return format!("{sign}{m}e{exp}");
    }
    if exp >= 0 {
        let int_len = exp as usize + 1;
        if digits.len() <= int_len {
            format!("{sign}{digits}{}", "0".repeat(int_len - digits.len()))
        } else {
            format!("{sign}{}.{}", &digits[..int_len], &digits[int_len..])
        }
    } else {
        format!("{sign}0.{}{digits}", "0".repeat((-exp - 1) as usize))
    }
}

/// A value as `eval` prints it: SI base units with the unit symbol (or dimension).
pub fn format_value(v: &Value) -> String {
    match v {
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => s.clone(),
        Value::Num(q) => match unit_label(&q.dim) {
            Some(u) => format!("{} {u}", fmt_num(q.v)),
            None => fmt_num(q.v),
        },
    }
}

// ── Lexer ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Str(String),
    Id(String),
    Op(&'static str),
    LP,
    RP,
    Comma,
}

const KEYWORDS: &[&str] = &["and", "or", "not", "true", "false", "if", "then", "else"];

fn lex(src: &str) -> Result<Vec<Tok>, ExprError> {
    let cs: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < cs.len() {
        let c = cs[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == '.' && cs.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let start = i;
            while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '.') {
                i += 1;
            }
            if i < cs.len() && (cs[i] == 'e' || cs[i] == 'E') {
                let mut j = i + 1;
                if j < cs.len() && (cs[j] == '+' || cs[j] == '-') {
                    j += 1;
                }
                if j < cs.len() && cs[j].is_ascii_digit() {
                    while j < cs.len() && cs[j].is_ascii_digit() {
                        j += 1;
                    }
                    i = j;
                }
            }
            let text: String = cs[start..i].iter().collect();
            let n: f64 = text.parse().map_err(|_| ExprError::Parse(format!("bad number '{text}'")))?;
            if !n.is_finite() {
                return Err(ExprError::Parse(format!("number '{text}' is out of range")));
            }
            out.push(Tok::Num(n));
        } else if c.is_alphabetic() || c == '_' || c == 'µ' {
            let start = i;
            loop {
                while i < cs.len() && (cs[i].is_alphanumeric() || cs[i] == '_' || cs[i] == 'µ') {
                    i += 1;
                }
                // `a.b` and `A::B` continue one name; a trailing separator does not.
                if i + 1 < cs.len() && cs[i] == '.' && (cs[i + 1].is_alphabetic() || cs[i + 1] == '_') {
                    i += 1;
                } else if i + 2 < cs.len() && cs[i] == ':' && cs[i + 1] == ':' && (cs[i + 2].is_alphabetic() || cs[i + 2] == '_') {
                    i += 2;
                } else {
                    break;
                }
            }
            out.push(Tok::Id(cs[start..i].iter().collect()));
        } else if c == '\'' || c == '"' {
            let q = c;
            let mut j = i + 1;
            let mut s = String::new();
            loop {
                match cs.get(j) {
                    None => return Err(ExprError::Parse("unterminated string".to_string())),
                    Some(&ch) if ch == q => break,
                    Some(&ch) => s.push(ch),
                }
                j += 1;
            }
            i = j + 1;
            out.push(Tok::Str(s));
        } else {
            let two: String = cs[i..(i + 2).min(cs.len())].iter().collect();
            let (tok, len) = match two.as_str() {
                "==" => (Tok::Op("=="), 2),
                "!=" => (Tok::Op("!="), 2),
                "<=" => (Tok::Op("<="), 2),
                ">=" => (Tok::Op(">="), 2),
                "&&" => (Tok::Op("and"), 2),
                "||" => (Tok::Op("or"), 2),
                _ => match c {
                    '<' => (Tok::Op("<"), 1),
                    '>' => (Tok::Op(">"), 1),
                    '+' => (Tok::Op("+"), 1),
                    '-' => (Tok::Op("-"), 1),
                    '*' => (Tok::Op("*"), 1),
                    '/' => (Tok::Op("/"), 1),
                    '%' => (Tok::Op("%"), 1),
                    '!' => (Tok::Op("not"), 1),
                    '(' => (Tok::LP, 1),
                    ')' => (Tok::RP, 1),
                    ',' => (Tok::Comma, 1),
                    other => return Err(ExprError::Parse(format!("unexpected character '{other}'"))),
                },
            };
            out.push(tok);
            i += len;
        }
    }
    Ok(out)
}

// ── Parser ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Value),
    Name(String),
    Unary(&'static str, Box<Expr>),
    Bin(&'static str, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.at).cloned();
        self.at += 1;
        t
    }
    fn is_kw(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(Tok::Id(s)) if s == kw)
    }
    fn eat_kw(&mut self, kw: &str) -> bool {
        if self.is_kw(kw) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, t: Tok, what: &str) -> Result<(), ExprError> {
        if self.peek() == Some(&t) {
            self.at += 1;
            Ok(())
        } else {
            Err(ExprError::Parse(format!("expected {what}")))
        }
    }

    fn expr(&mut self) -> Result<Expr, ExprError> {
        if self.eat_kw("if") {
            let c = self.expr()?;
            if !self.eat_kw("then") {
                return Err(ExprError::Parse("expected 'then' after the condition of 'if'".to_string()));
            }
            let a = self.expr()?;
            if !self.eat_kw("else") {
                return Err(ExprError::Parse("expected 'else' in 'if … then … else …'".to_string()));
            }
            let b = self.expr()?;
            return Ok(Expr::If(Box::new(c), Box::new(a), Box::new(b)));
        }
        self.or()
    }

    fn or(&mut self) -> Result<Expr, ExprError> {
        let mut l = self.and()?;
        while self.is_kw("or") || self.peek() == Some(&Tok::Op("or")) {
            self.at += 1;
            let r = self.and()?;
            l = Expr::Bin("or", Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn and(&mut self) -> Result<Expr, ExprError> {
        let mut l = self.not()?;
        while self.is_kw("and") || self.peek() == Some(&Tok::Op("and")) {
            self.at += 1;
            let r = self.not()?;
            l = Expr::Bin("and", Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn not(&mut self) -> Result<Expr, ExprError> {
        if self.is_kw("not") || self.peek() == Some(&Tok::Op("not")) {
            self.at += 1;
            let e = self.not()?;
            return Ok(Expr::Unary("not", Box::new(e)));
        }
        self.cmp()
    }

    fn cmp(&mut self) -> Result<Expr, ExprError> {
        let l = self.add()?;
        if let Some(Tok::Op(op @ ("==" | "!=" | "<" | "<=" | ">" | ">="))) = self.peek().cloned() {
            self.at += 1;
            let r = self.add()?;
            return Ok(Expr::Bin(op, Box::new(l), Box::new(r)));
        }
        Ok(l)
    }

    fn add(&mut self) -> Result<Expr, ExprError> {
        let mut l = self.mul()?;
        while let Some(Tok::Op(op @ ("+" | "-"))) = self.peek().cloned() {
            self.at += 1;
            let r = self.mul()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn mul(&mut self) -> Result<Expr, ExprError> {
        let mut l = self.unary()?;
        while let Some(Tok::Op(op @ ("*" | "/" | "%"))) = self.peek().cloned() {
            self.at += 1;
            let r = self.unary()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn unary(&mut self) -> Result<Expr, ExprError> {
        if self.peek() == Some(&Tok::Op("-")) {
            self.at += 1;
            let e = self.unary()?;
            return Ok(Expr::Unary("-", Box::new(e)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ExprError> {
        match self.next() {
            Some(Tok::Num(n)) => {
                // A unit may follow the number: `50 ms`, `3.3V`.
                if let Some(Tok::Id(u)) = self.peek().cloned() {
                    if !KEYWORDS.contains(&u.as_str()) {
                        match unit_scale(&u) {
                            Some((scale, d)) => {
                                self.at += 1;
                                return Ok(Expr::Lit(Value::Num(Quantity { v: n * scale, dim: d })));
                            }
                            None => return Err(ExprError::Parse(format!("unknown unit '{u}' after {n}"))),
                        }
                    }
                }
                Ok(Expr::Lit(Value::Num(Quantity::scalar(n))))
            }
            Some(Tok::Str(s)) => Ok(Expr::Lit(Value::Str(s))),
            Some(Tok::LP) => {
                let e = self.expr()?;
                self.expect(Tok::RP, "')'")?;
                Ok(e)
            }
            Some(Tok::Id(id)) => match id.as_str() {
                "if" => {
                    // `if` is also an operand: `1 + if c then 2 else 3`.
                    self.at -= 1;
                    self.expr()
                }
                "true" => Ok(Expr::Lit(Value::Bool(true))),
                "false" => Ok(Expr::Lit(Value::Bool(false))),
                kw if KEYWORDS.contains(&kw) => Err(ExprError::Parse(format!("unexpected '{kw}'"))),
                _ => {
                    if self.peek() == Some(&Tok::LP) {
                        self.at += 1;
                        let mut args = Vec::new();
                        if self.peek() != Some(&Tok::RP) {
                            loop {
                                args.push(self.expr()?);
                                if self.peek() == Some(&Tok::Comma) {
                                    self.at += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                        self.expect(Tok::RP, "')' closing the call")?;
                        check_arity(&id, args.len())?;
                        Ok(Expr::Call(id, args))
                    } else {
                        Ok(Expr::Name(id))
                    }
                }
            },
            Some(t) => Err(ExprError::Parse(format!("unexpected {}", tok_text(&t)))),
            None => Err(ExprError::Parse("unexpected end of expression".to_string())),
        }
    }
}

fn tok_text(t: &Tok) -> String {
    match t {
        Tok::Num(n) => format!("number {n}"),
        Tok::Str(s) => format!("string '{s}'"),
        Tok::Id(s) => format!("'{s}'"),
        Tok::Op(o) => format!("'{o}'"),
        Tok::LP => "'('".to_string(),
        Tok::RP => "')'".to_string(),
        Tok::Comma => "','".to_string(),
    }
}

fn check_arity(f: &str, n: usize) -> Result<(), ExprError> {
    let ok = match f {
        "abs" | "sqrt" | "floor" | "ceil" | "round" | "exp" | "ln" | "sin" | "cos" | "tanh" => n == 1,
        "pow" => n == 2,
        "min" | "max" => n >= 1,
        _ => return Err(ExprError::Parse(format!("unknown function '{f}' (abs, min, max, sqrt, floor, ceil, round, pow, exp, ln, sin, cos, tanh)"))),
    };
    if ok {
        Ok(())
    } else {
        Err(ExprError::Parse(format!("wrong number of arguments ({n}) to {f}")))
    }
}

/// Parse an expression.
pub fn parse(src: &str) -> Result<Expr, ExprError> {
    let toks = lex(src)?;
    if toks.is_empty() {
        return Err(ExprError::Parse("empty expression".to_string()));
    }
    let mut p = Parser { toks, at: 0 };
    let e = p.expr()?;
    if let Some(t) = p.peek() {
        return Err(ExprError::Parse(format!("unexpected {} after the expression", tok_text(t))));
    }
    Ok(e)
}

/// Parse a `;`-separated list of `name := expression` statements (REQ-TRS-SMSCHEMA-001). A `;` inside a
/// quoted string or a parenthesis does not separate; empty statements (a trailing `;`) are ignored.
pub fn split_statements(src: &str) -> Result<Vec<(String, String)>, ExprError> {
    let mut parts: Vec<String> = Vec::new();
    let (mut cur, mut depth, mut quote) = (String::new(), 0i32, None::<char>);
    for c in src.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
                cur.push(c);
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    cur.push(c);
                }
                '(' => {
                    depth += 1;
                    cur.push(c);
                }
                ')' => {
                    depth -= 1;
                    cur.push(c);
                }
                ';' if depth == 0 => parts.push(std::mem::take(&mut cur)),
                _ => cur.push(c),
            },
        }
    }
    parts.push(cur);
    let mut out = Vec::new();
    for part in parts.iter().map(|p| p.trim()).filter(|p| !p.is_empty()) {
        let Some((name, rhs)) = part.split_once(":=") else {
            return Err(ExprError::Parse(format!("'{part}' is not an assignment `name := expression`")));
        };
        let name = name.trim();
        let ok = !name.is_empty()
            && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !ok {
            return Err(ExprError::Parse(format!("'{name}' is not a variable name")));
        }
        out.push((name.to_string(), rhs.trim().to_string()));
    }
    Ok(out)
}

/// [`split_statements`] with each right-hand side parsed.
pub fn parse_statements(src: &str) -> Result<Vec<(String, Expr)>, ExprError> {
    split_statements(src)?.into_iter().map(|(n, rhs)| Ok((n, parse(&rhs)?))).collect()
}

/// Read a binding value: `50ms`, `-3.5 V`, `true`, `'x'`, `ArmStatus::armed`, `12`.
pub fn parse_value(text: &str) -> Option<Value> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    let toks = lex(t).ok()?;
    // A bare qualified/plain identifier is an enumeration literal.
    if let [Tok::Id(id)] = toks.as_slice() {
        return match id.as_str() {
            "true" => Some(Value::Bool(true)),
            "false" => Some(Value::Bool(false)),
            _ if KEYWORDS.contains(&id.as_str()) => None,
            _ => Some(Value::Str(id.clone())),
        };
    }
    parse(t).ok()?.eval(&|_: &str| None).ok()
}

// ── Evaluation ───────────────────────────────────────────────────────────────

fn dim_add(a: &Dim, b: &Dim, sign: i8) -> Result<Dim, ExprError> {
    let mut out = [0i8; 7];
    for k in 0..7 {
        out[k] = sign
            .checked_mul(b.0[k])
            .and_then(|x| a.0[k].checked_add(x))
            .ok_or_else(|| ExprError::Dimension("dimension exponent out of range".to_string()))?;
    }
    Ok(Dim(out))
}

fn finite(v: f64, what: &str) -> Result<f64, ExprError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(ExprError::Domain(format!("{what} is not finite")))
    }
}

fn dim_name(d: &Dim) -> String {
    d.human()
}

fn want_num(v: &Value, ctx: &str) -> Result<Quantity, ExprError> {
    match v {
        Value::Num(q) => Ok(*q),
        other => Err(ExprError::Type(format!("{ctx} needs a number, got {}", other.kind()))),
    }
}

fn want_bool(v: &Value, ctx: &str) -> Result<bool, ExprError> {
    match v {
        Value::Bool(b) => Ok(*b),
        other => Err(ExprError::Type(format!("{ctx} needs a boolean, got {}", other.kind()))),
    }
}

fn last_segment(s: &str) -> &str {
    s.rsplit("::").next().unwrap_or(s)
}

/// Equality of two quantities of one dimension: relative 1e-9, with an absolute floor of 1e-15 (SI
/// base units) so that float noise around zero (`0.1 + 0.2 - 0.3 == 0`) compares equal.
fn same_qty(a: &Quantity, b: &Quantity) -> bool {
    let tol = (1e-9 * a.v.abs().max(b.v.abs())).max(1e-15);
    (a.v - b.v).abs() <= tol
}

fn compare(op: &str, l: &Value, r: &Value) -> Result<bool, ExprError> {
    match (l, r) {
        (Value::Num(a), Value::Num(b)) => {
            if a.dim != b.dim {
                return Err(ExprError::Dimension(format!("cannot compare {} with {}", dim_name(&a.dim), dim_name(&b.dim))));
            }
            Ok(match op {
                "==" => same_qty(a, b),
                "!=" => !same_qty(a, b),
                "<" => a.v < b.v && !same_qty(a, b),
                "<=" => a.v < b.v || same_qty(a, b),
                ">" => a.v > b.v && !same_qty(a, b),
                _ => a.v > b.v || same_qty(a, b),
            })
        }
        (Value::Bool(a), Value::Bool(b)) if op == "==" || op == "!=" => Ok((a == b) == (op == "==")),
        (Value::Str(a), Value::Str(b)) if op == "==" || op == "!=" => {
            let eq = a == b
                || ((a.contains("::") != b.contains("::")) && last_segment(a) == last_segment(b));
            Ok(eq == (op == "=="))
        }
        _ if op == "==" || op == "!=" => Err(ExprError::Type(format!("cannot compare {} with {}", l.kind(), r.kind()))),
        _ => Err(ExprError::Type(format!("'{op}' needs two numbers, got {} and {}", l.kind(), r.kind()))),
    }
}

fn arith(op: &str, l: &Value, r: &Value) -> Result<Value, ExprError> {
    let (a, b) = (want_num(l, op)?, want_num(r, op)?);
    let q = match op {
        "+" | "-" | "%" => {
            if a.dim != b.dim {
                return Err(ExprError::Dimension(format!("'{op}' needs equal dimensions, got {} and {}", dim_name(&a.dim), dim_name(&b.dim))));
            }
            let v = match op {
                "+" => a.v + b.v,
                "-" => a.v - b.v,
                _ => {
                    if b.v == 0.0 {
                        return Err(ExprError::Domain("remainder by zero".to_string()));
                    }
                    a.v % b.v
                }
            };
            Quantity { v: finite(v, "result")?, dim: a.dim }
        }
        "*" => Quantity { v: finite(a.v * b.v, "result")?, dim: dim_add(&a.dim, &b.dim, 1)? },
        _ => {
            if b.v == 0.0 {
                return Err(ExprError::Domain("division by zero".to_string()));
            }
            Quantity { v: finite(a.v / b.v, "result")?, dim: dim_add(&a.dim, &b.dim, -1)? }
        }
    };
    Ok(Value::Num(q))
}

fn call(f: &str, args: &[Value]) -> Result<Value, ExprError> {
    let nums: Vec<Quantity> = args.iter().map(|a| want_num(a, f)).collect::<Result<_, _>>()?;
    let q = match f {
        "abs" => Quantity { v: nums[0].v.abs(), dim: nums[0].dim },
        "floor" => Quantity { v: nums[0].v.floor(), dim: nums[0].dim },
        "ceil" => Quantity { v: nums[0].v.ceil(), dim: nums[0].dim },
        "round" => Quantity { v: nums[0].v.round(), dim: nums[0].dim },
        "sqrt" => {
            if nums[0].v < 0.0 {
                return Err(ExprError::Domain("sqrt of a negative number".to_string()));
            }
            if nums[0].dim.0.iter().any(|e| e % 2 != 0) {
                return Err(ExprError::Dimension(format!("sqrt needs even dimension exponents, got {}", dim_name(&nums[0].dim))));
            }
            let mut d = [0i8; 7];
            for (o, e) in d.iter_mut().zip(nums[0].dim.0) {
                *o = e / 2;
            }
            Quantity { v: nums[0].v.sqrt(), dim: Dim(d) }
        }
        // transcendental functions come from the pure-Rust `libm`, not the platform's, so a result is the same bits on
        // every machine (GH #340); each takes and gives a dimensionless number (an angle in radians)
        "exp" | "ln" | "sin" | "cos" | "tanh" => {
            let x = nums[0];
            if x.dim != NONE {
                return Err(ExprError::Dimension(format!("{f} needs a dimensionless argument, got {}", dim_name(&x.dim))));
            }
            if f == "ln" && x.v <= 0.0 {
                return Err(ExprError::Domain("ln of a number that is not positive".to_string()));
            }
            let v = match f {
                "exp" => libm::exp(x.v),
                "ln" => libm::log(x.v),
                "sin" => libm::sin(x.v),
                "cos" => libm::cos(x.v),
                _ => libm::tanh(x.v),
            };
            Quantity { v: finite(v, "result")?, dim: NONE }
        }
        "pow" => {
            let n = nums[1];
            if n.dim != NONE || n.v.fract() != 0.0 || n.v.abs() > 1024.0 {
                return Err(ExprError::Dimension("pow needs a dimensionless integer exponent (|n| <= 1024)".to_string()));
            }
            if nums[0].v == 0.0 && n.v < 0.0 {
                return Err(ExprError::Domain("0 to a negative power".to_string()));
            }
            let e = n.v as i32;
            let mut d = [0i8; 7];
            for (o, x) in d.iter_mut().zip(nums[0].dim.0) {
                *o = i8::try_from(i32::from(x) * e).map_err(|_| ExprError::Dimension("dimension exponent out of range".to_string()))?;
            }
            let v = finite(ipow(nums[0].v, e), "result")?;
            if v == 0.0 && nums[0].v != 0.0 {
                return Err(ExprError::Domain("result underflows to zero".to_string()));
            }
            Quantity { v, dim: Dim(d) }
        }
        _ => {
            // min / max: every argument the same dimension
            let first = nums[0];
            if nums.iter().any(|q| q.dim != first.dim) {
                return Err(ExprError::Dimension(format!("{f} needs equal dimensions")));
            }
            let v = nums.iter().map(|q| q.v).fold(first.v, |a, b| if f == "min" { a.min(b) } else { a.max(b) });
            Quantity { v, dim: first.dim }
        }
    };
    Ok(Value::Num(q))
}

impl Expr {
    /// Evaluate with `lookup` resolving names. Deterministic and side-effect free.
    pub fn eval(&self, lookup: &dyn Fn(&str) -> Option<Value>) -> Result<Value, ExprError> {
        match self {
            Expr::Lit(v) => Ok(v.clone()),
            Expr::Name(n) => match lookup(n) {
                Some(v) => Ok(v),
                None if n.contains("::") && !n.contains('.') => Ok(Value::Str(n.clone())),
                None => Err(ExprError::Unbound(n.clone())),
            },
            Expr::Unary(op, e) => {
                let v = e.eval(lookup)?;
                match *op {
                    "not" => Ok(Value::Bool(!want_bool(&v, "not")?)),
                    _ => {
                        let q = want_num(&v, "unary '-'")?;
                        Ok(Value::Num(Quantity { v: -q.v, dim: q.dim }))
                    }
                }
            }
            Expr::Bin(op, l, r) => match *op {
                "and" => {
                    if !want_bool(&l.eval(lookup)?, "and")? {
                        return Ok(Value::Bool(false));
                    }
                    Ok(Value::Bool(want_bool(&r.eval(lookup)?, "and")?))
                }
                "or" => {
                    if want_bool(&l.eval(lookup)?, "or")? {
                        return Ok(Value::Bool(true));
                    }
                    Ok(Value::Bool(want_bool(&r.eval(lookup)?, "or")?))
                }
                "==" | "!=" | "<" | "<=" | ">" | ">=" => Ok(Value::Bool(compare(op, &l.eval(lookup)?, &r.eval(lookup)?)?)),
                _ => arith(op, &l.eval(lookup)?, &r.eval(lookup)?),
            },
            Expr::If(c, a, b) => {
                if want_bool(&c.eval(lookup)?, "if")? {
                    a.eval(lookup)
                } else {
                    b.eval(lookup)
                }
            }
            Expr::Call(f, args) => {
                let vals: Vec<Value> = args.iter().map(|a| a.eval(lookup)).collect::<Result<_, _>>()?;
                call(f, &vals)
            }
        }
    }

    /// The atomic conditions of a Boolean expression: the maximal subexpressions that are not `and`, `or` or `not`.
    pub fn atoms(&self) -> Vec<&Expr> {
        match self {
            Expr::Bin("and", a, b) | Expr::Bin("or", a, b) => {
                let mut v = a.atoms();
                v.extend(b.atoms());
                v
            }
            Expr::Unary("not", a) => a.atoms(),
            other => vec![other],
        }
    }

    /// The expression as source text (fully parenthesised below the top level).
    pub fn to_source(&self) -> String {
        fn paren(e: &Expr) -> String {
            match e {
                Expr::Bin(..) | Expr::If(..) => format!("({})", e.to_source()),
                _ => e.to_source(),
            }
        }
        match self {
            Expr::Lit(Value::Str(s)) => format!("'{s}'"),
            Expr::Lit(v) => format_value(v),
            Expr::Name(n) => n.clone(),
            Expr::Unary(op, a) => if *op == "not" { format!("not {}", paren(a)) } else { format!("{op}{}", paren(a)) },
            Expr::Bin(op, a, b) => format!("{} {op} {}", paren(a), paren(b)),
            Expr::If(c, a, b) => format!("if {} then {} else {}", c.to_source(), a.to_source(), b.to_source()),
            Expr::Call(f, args) => format!("{f}({})", args.iter().map(|a| a.to_source()).collect::<Vec<_>>().join(", ")),
        }
    }

    /// Every identifier the expression uses, in first-use order, without duplicates.
    pub fn names(&self) -> Vec<String> {
        fn walk(e: &Expr, out: &mut Vec<String>) {
            match e {
                Expr::Lit(_) => {}
                Expr::Name(n) => {
                    if !out.contains(n) {
                        out.push(n.clone());
                    }
                }
                Expr::Unary(_, a) => walk(a, out),
                Expr::Bin(_, a, b) => {
                    walk(a, out);
                    walk(b, out);
                }
                Expr::If(c, a, b) => {
                    walk(c, out);
                    walk(a, out);
                    walk(b, out);
                }
                Expr::Call(_, args) => args.iter().for_each(|a| walk(a, out)),
            }
        }
        let mut out = Vec::new();
        walk(self, &mut out);
        out
    }
}

/// Convert a quantity to `unit` (`ms`, `kWh`, …); a dimension mismatch is an error.
pub fn to_unit(q: &Quantity, unit: &str) -> Result<f64, ExprError> {
    let (scale, d) = unit_scale(unit).ok_or_else(|| ExprError::Parse(format!("unknown unit '{unit}'")))?;
    if d != q.dim {
        return Err(ExprError::Dimension(format!("cannot express {} in {unit} ({})", dim_name(&q.dim), dim_name(&d))));
    }
    Ok(q.v / scale)
}
