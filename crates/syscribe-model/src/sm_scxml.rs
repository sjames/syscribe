//! W3C SCXML export of the machine IR (REQ-TRS-SMSCXML-001, GH #283). Pure function of the IR; constructs
//! with no SCXML form are written as comments and returned as warnings, never silently dropped.

use crate::expr::{format_value, Expr, Value};
use crate::ir::{Action, IrExpr, Kind, Machine, Trigger};

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Dotted state paths are valid NCNames and unique, so they are the SCXML ids.
fn id_of(path: &str) -> String {
    esc(path)
}

/// The SCXML event name of a syscribe event (`<port>/<payload>` → dotted).
pub fn scxml_event(n: &str) -> String {
    event_name(n)
}

fn event_name(n: &str) -> String {
    n.replace("::", ".").replace('/', ".")
}

fn num(v: f64) -> String {
    format!("{v}")
}

fn ecma(e: &Expr, bind: Option<&str>, warnings: &mut Vec<String>) -> String {
    let sub = |x: &Expr, w: &mut Vec<String>| match x {
        Expr::Bin(..) | Expr::If(..) => format!("({})", ecma(x, bind, w)),
        _ => ecma(x, bind, w),
    };
    match e {
        Expr::Lit(Value::Num(q)) => num(q.v),
        Expr::Lit(Value::Bool(b)) => b.to_string(),
        Expr::Lit(Value::Str(s)) => format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")),
        Expr::Name(n) => {
            let root = n.split('.').next().unwrap_or(n);
            if n.contains('.') && Some(root) == bind {
                format!("_event.data.{}", &n[root.len() + 1..])
            } else if n.contains("::") && !n.contains('.') {
                format!("'{n}'")
            } else {
                if n.contains('.') {
                    warnings.push(format!("attribute path '{n}' has no value in the SCXML datamodel"));
                }
                n.clone()
            }
        }
        Expr::Unary("not", a) => format!("!{}", sub_paren(a, bind, warnings)),
        Expr::Unary(op, a) => format!("{op}{}", sub_paren(a, bind, warnings)),
        Expr::Bin(op, a, b) => {
            let o = match *op {
                "and" => "&&",
                "or" => "||",
                "==" => "===",
                "!=" => "!==",
                other => other,
            };
            format!("{} {o} {}", sub(a, warnings), sub(b, warnings))
        }
        Expr::If(c, a, b) => format!("{} ? {} : {}", sub(c, warnings), sub(a, warnings), sub(b, warnings)),
        Expr::Call(f, args) => {
            let name = match f.as_str() {
                "abs" => "Math.abs",
                "min" => "Math.min",
                "max" => "Math.max",
                "sqrt" => "Math.sqrt",
                "floor" => "Math.floor",
                "ceil" => "Math.ceil",
                "round" => {
                    warnings.push("round(): JavaScript Math.round rounds halves towards +infinity, syscribe rounds them away from zero".to_string());
                    "Math.round"
                }
                "pow" => "Math.pow",
                other => {
                    warnings.push(format!("function '{other}' has no ECMAScript equivalent"));
                    other
                }
            };
            format!("{name}({})", args.iter().map(|a| ecma(a, bind, warnings)).collect::<Vec<_>>().join(", "))
        }
    }
}

fn sub_paren(a: &Expr, bind: Option<&str>, w: &mut Vec<String>) -> String {
    format!("({})", ecma(a, bind, w))
}

struct Out {
    s: String,
    warnings: Vec<String>,
}

impl Out {
    fn line(&mut self, depth: usize, text: &str) {
        self.s.push_str(&"  ".repeat(depth));
        self.s.push_str(text);
        self.s.push('\n');
    }

    fn actions(&mut self, m: &Machine, depth: usize, acts: &[Action], bind: Option<&str>, owner: &str) {
        for a in acts {
            match a {
                Action::Assign { var, expr } => {
                    let e = ecma(&expr.ast, bind, &mut self.warnings);
                    self.line(depth, &format!("<assign location=\"{}\" expr=\"{}\"/>", esc(&m.variables[*var].name), esc(&e)));
                }
                Action::Behavior { target, role } => {
                    if *role == Some("do") {
                        self.warnings.push(format!("doAction {target} of {owner} runs to completion on entry in syscribe; it has no SCXML form and is not represented"));
                        self.line(depth, &format!("<!-- doAction {target} (runs to completion on entry in syscribe; not represented) -->"));
                    } else if m.behaviors.contains_key(target) {
                        self.warnings.push(format!("action {target} of {owner} is executed by syscribe but has no SCXML form; the exported machine does not run it and may differ"));
                        self.line(depth, &format!("<!-- action {target} (executed by syscribe; no SCXML form, not represented) -->"));
                    } else {
                        self.warnings.push(format!("behavior {target} of {owner} is a model reference that syscribe records but does not execute; not represented"));
                        self.line(depth, &format!("<!-- behavior {target} (a model reference; not executed) -->"));
                    }
                }
            }
        }
    }

    fn cond(&mut self, g: &IrExpr, bind: Option<&str>) -> String {
        ecma(&g.ast, bind, &mut self.warnings)
    }

    fn state(&mut self, m: &Machine, id: usize, depth: usize) {
        let st = &m.states[id];
        let sid = id_of(&st.path);
        let kids: Vec<usize> = m.states.iter().filter(|s| s.parent == Some(id)).map(|s| s.id).collect();
        let out: Vec<&crate::ir::Transition> = m.transitions.iter().filter(|t| t.source == id).collect();
        let empty = kids.is_empty() && out.is_empty() && st.entry.is_empty() && st.exit.is_empty();
        if st.kind == Kind::Final {
            if st.parent.is_none() {
                self.warnings.push(format!("top-level final state {}: an SCXML engine ends the session on entering it, while syscribe keeps it in the configuration", st.path));
            }
            if empty {
                self.line(depth, &format!("<final id=\"{sid}\"/>"));
            } else {
                self.line(depth, &format!("<final id=\"{sid}\">"));
                self.body(m, id, depth + 1, &out);
                self.line(depth, "</final>");
            }
            return;
        }
        let tag = if st.kind == Kind::Parallel { "parallel" } else { "state" };
        let initial = match (st.kind, st.initial) {
            (Kind::Compound, Some(i)) => format!(" initial=\"{}\"", id_of(&m.states[i].path)),
            _ => String::new(),
        };
        if empty {
            self.line(depth, &format!("<{tag} id=\"{sid}\"{initial}/>"));
            return;
        }
        self.line(depth, &format!("<{tag} id=\"{sid}\"{initial}>"));
        self.body(m, id, depth + 1, &out);
        for k in kids {
            self.state(m, k, depth + 1);
        }
        self.line(depth, &format!("</{tag}>"));
    }

    fn body(&mut self, m: &Machine, id: usize, depth: usize, out: &[&crate::ir::Transition]) {
        let st = &m.states[id];
        let owner = format!("state {}", st.path);
        let arming: Vec<&&crate::ir::Transition> = out.iter().filter(|t| matches!(t.trigger, Trigger::Timer { .. } | Trigger::Change { .. })).collect();
        if !st.entry.is_empty() || !arming.is_empty() {
            self.line(depth, "<onentry>");
            self.actions(m, depth + 1, &st.entry, None, &owner);
            for t in &arming {
                match &t.trigger {
                    Trigger::Timer { delay_seconds } => {
                        if (delay_seconds * 1000.0).fract().abs() > 1e-9 {
                            self.warnings.push(format!("transition t{}: the delay {delay_seconds} s is rounded to whole milliseconds", t.id));
                        }
                        if t.internal {
                            self.warnings.push(format!("transition t{}: an internal `after` transition is re-armed after it fires in syscribe, but the SCXML <send> is not repeated", t.id));
                        }
                        self.line(depth + 1, &format!("<send id=\"timer_t{0}\" event=\"timer.t{0}\" delay=\"{1}ms\"/>", t.id, num((delay_seconds * 1000.0).round())));
                    }
                    Trigger::Change { .. } => self.line(depth + 1, &format!("<assign location=\"armed_t{}\" expr=\"true\"/>", t.id)),
                    _ => {}
                }
            }
            self.line(depth, "</onentry>");
        }
        if !st.exit.is_empty() || !arming.is_empty() {
            self.line(depth, "<onexit>");
            self.actions(m, depth + 1, &st.exit, None, &owner);
            for t in &arming {
                match &t.trigger {
                    Trigger::Timer { .. } => self.line(depth + 1, &format!("<cancel sendid=\"timer_t{}\"/>", t.id)),
                    Trigger::Change { .. } => self.line(depth + 1, &format!("<assign location=\"armed_t{}\" expr=\"false\"/>", t.id)),
                    _ => {}
                }
            }
            self.line(depth, "</onexit>");
        }
        for t in out {
            self.transition(m, t, depth);
        }
    }

    fn transition(&mut self, m: &Machine, t: &crate::ir::Transition, depth: usize) {
        let bind = match &t.trigger {
            Trigger::Event { bind, .. } => bind.as_deref(),
            _ => None,
        };
        let owner = format!("transition t{}", t.id);
        let event = match &t.trigger {
            Trigger::Event { name, .. } => Some(event_name(name)),
            Trigger::Timer { .. } => Some(format!("timer.t{}", t.id)),
            Trigger::Change { .. } => {
                // SCXML eventless transitions are level-triggered; the SysML v2 one-shot observation is
                // reproduced with an armed flag and a raised signal.
                let Trigger::Change { cond } = &t.trigger else { unreachable!() };
                let c = ecma(&cond.ast, None, &mut self.warnings);
                self.line(depth, &format!("<transition cond=\"{}\">", esc(&format!("armed_t{} && ({c})", t.id))));
                self.line(depth + 1, &format!("<assign location=\"armed_t{}\" expr=\"false\"/>", t.id));
                self.line(depth + 1, &format!("<raise event=\"signal.t{}\"/>", t.id));
                self.line(depth, "</transition>");
                if t.internal {
                    self.warnings.push(format!("{owner}: an internal transition with a `when` trigger is not re-armed after it fires in the SCXML export (syscribe waits for a new false→true change)"));
                }
                Some(format!("signal.t{}", t.id))
            }
            Trigger::Done { state } => Some(format!("done.state.{}", id_of(&m.states[*state].path))),
            Trigger::Eventless => None,
        };
        let mut attrs = String::new();
        if let Some(e) = &event {
            attrs.push_str(&format!(" event=\"{}\"", esc(e)));
        }
        if let Some(g) = &t.guard {
            let c = self.cond(g, bind);
            attrs.push_str(&format!(" cond=\"{}\"", esc(&c)));
        }
        if let Some(target) = t.target {
            attrs.push_str(&format!(" target=\"{}\"", id_of(&m.states[target].path)));
        } else {
            attrs.push_str(" type=\"internal\"");
        }
        if t.actions.is_empty() {
            self.line(depth, &format!("<transition{attrs}/>"));
        } else {
            self.line(depth, &format!("<transition{attrs}>"));
            self.actions(m, depth + 1, &t.actions, bind, &owner);
            self.line(depth, "</transition>");
        }
    }
}

/// The SCXML document for a machine and the list of constructs that could not be represented.
pub fn export(m: &Machine) -> (String, Vec<String>) {
    let mut o = Out { s: String::new(), warnings: Vec::new() };
    o.s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    o.line(0, &format!("<!-- generated by syscribe from machine {} (IR hash {}). -->", m.machine, m.hash));
    o.line(0, "<!-- Numbers are SI base units; units are flattened. Change triggers (`when`) are SysML v2 one-shot observations,");
    o.line(0, "     emulated with armed_<id> flags because SCXML eventless transitions are level-triggered. -->");
    let tops: Vec<usize> = m.states.iter().filter(|s| s.parent.is_none()).map(|s| s.id).collect();
    let initial_attr = if m.parallel { "__machine".to_string() } else { m.initial.map(|i| id_of(&m.states[i].path)).unwrap_or_default() };
    let use_initial_element = !m.initial_actions.is_empty() && !m.parallel;
    let initial_attr_final = if use_initial_element { "__init".to_string() } else { initial_attr.clone() };
    o.line(0, &format!("<scxml xmlns=\"http://www.w3.org/2005/07/scxml\" version=\"1.0\" datamodel=\"ecmascript\" initial=\"{initial_attr_final}\">"));
    o.line(1, "<datamodel>");
    for v in &m.variables {
        let init = match &v.value {
            Value::Num(q) => num(q.v),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")),
        };
        let _ = format_value;
        o.line(2, &format!("<data id=\"{}\" expr=\"{}\"/>", esc(&v.name), esc(&init)));
    }
    for t in m.transitions.iter().filter(|t| matches!(t.trigger, Trigger::Change { .. })) {
        o.line(2, &format!("<data id=\"armed_t{}\" expr=\"false\"/>", t.id));
    }
    o.line(1, "</datamodel>");
    if use_initial_element {
        // SCXML allows only the `initial` attribute on <scxml>, so the initial effect runs on the transition
        // out of a synthetic first state.
        o.line(1, "<state id=\"__init\">");
        o.line(2, &format!("<transition target=\"{initial_attr}\">"));
        let acts = m.initial_actions.clone();
        o.actions(m, 3, &acts, None, "initialEffect");
        o.line(2, "</transition>");
        o.line(1, "</state>");
    }
    if m.parallel {
        o.line(1, "<state id=\"__machine\" initial=\"__regions\">");
        if !m.initial_actions.is_empty() {
            o.line(2, "<onentry>");
            let acts = m.initial_actions.clone();
            o.actions(m, 3, &acts, None, "initialEffect");
            o.line(2, "</onentry>");
        }
        o.line(2, "<parallel id=\"__regions\">");
        for t in tops {
            o.state(m, t, 3);
        }
        o.line(2, "</parallel>");
        o.line(1, "</state>");
    } else {
        for t in tops {
            o.state(m, t, 1);
        }
    }
    o.line(0, "</scxml>");
    o.warnings.sort();
    o.warnings.dedup();
    (o.s, o.warnings)
}
