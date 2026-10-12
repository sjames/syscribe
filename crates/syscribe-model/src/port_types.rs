//! Typed ports (GH #331): a port typed by a `PortDef` carries the events its items name, and each item's `ItemDef`
//! gives the payload fields with their types and units. A machine's `send`/`accept` through such a port is checked
//! against them: an event the port does not carry or carries the other way, a payload field the item does not have
//! (`E932`), and a payload value or use of a field of the wrong type or dimension (`E935`). An untyped port is checked
//! as before, by name and direction only.

use serde_yaml::{Mapping, Value as Y};

use crate::element::{ElementType, RawElement};
use crate::ir::{self, Action, IrExpr, Machine, Trigger};
use crate::resolver::Resolver;
use crate::sm_exec::field;
use crate::system::{self, Dir};
use crate::typecheck::{self, Ty};

/// `(code, message)`.
pub type Finding = (&'static str, String);

/// One payload field of an item.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Ty,
}

/// One item a port type carries: the event it names and, when its `ItemDef` resolves, the payload fields.
#[derive(Debug, Clone)]
pub struct Item {
    /// The event name a machine uses: the `ItemDef`'s name, or the usage's own name when it has no type.
    pub event: String,
    /// The direction relative to a part that owns the port unconjugated; `None` when the item does not say.
    pub dir: Option<Dir>,
    /// `None` when the item has no resolvable `ItemDef` (its fields are not known, so not checked).
    pub fields: Option<Vec<Field>>,
}

/// A port with a type.
#[derive(Debug, Clone)]
pub struct TypedPort {
    pub name: String,
    /// The qualified name of the `PortDef`.
    pub ty: String,
    pub conjugated: bool,
    pub items: Vec<Item>,
}

impl TypedPort {
    fn item(&self, event: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.event == event)
    }

    /// The direction `item` flows at this port: the item's own, reversed when the port is conjugated.
    fn flow(&self, item: &Item) -> Option<Dir> {
        item.dir.map(|d| match (self.conjugated, d) {
            (true, Dir::In) => Dir::Out,
            (true, Dir::Out) => Dir::In,
            (_, d) => d,
        })
    }

    fn carried(&self) -> String {
        if self.items.is_empty() {
            "no items".into()
        } else {
            self.items
                .iter()
                .map(|i| i.event.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

fn parse_dir(s: Option<&str>) -> Option<Dir> {
    match s? {
        "in" => Some(Dir::In),
        "out" => Some(Dir::Out),
        "inout" => Some(Dir::InOut),
        _ => None,
    }
}

fn last(q: &str) -> &str {
    q.rsplit("::").next().unwrap_or(q)
}

/// The static type of a field declared `typedBy:`/`type:` with an optional `unit:`. A number with no unit is
/// dimensionless; a type that is neither a scalar nor an ISQ quantity is unknown (compatible with everything).
fn field_ty(m: &Mapping) -> Ty {
    let t = field(m, "typedBy")
        .or_else(|| field(m, "type"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let unit = field(m, "unit").and_then(|v| v.as_str());
    match last(t) {
        "Boolean" => Ty::Bool,
        "String" => Ty::Str,
        "Real" | "Integer" | "Natural" | "Positive" | "Rational" | "Number" | "Complex"
        | "NumericalValue" => match unit {
            Some(u) => Ty::Num(crate::units::unit_dimension(u)),
            None => Ty::Num(Some(crate::units::Dim([0; 7]))),
        },
        _ => match (
            unit.and_then(crate::units::unit_dimension),
            crate::units::quantity_dimension(t),
        ) {
            (Some(d), _) | (None, Some(d)) => Ty::Num(Some(d)),
            _ => Ty::Unknown,
        },
    }
}

/// The payload fields of an `ItemDef`: its non-port `features:` along its definition chain.
fn item_fields(elements: &[RawElement], resolver: &Resolver, def: &RawElement) -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    for d in system::chain(elements, resolver, def) {
        for f in d.frontmatter.features.iter().flatten() {
            let Some(m) = f.as_mapping() else { continue };
            let Some(n) = field(m, "name").and_then(|n| n.as_str()) else {
                continue;
            };
            if field(m, "type").and_then(|t| t.as_str()) == Some("Port")
                || out.iter().any(|x| x.name == n)
            {
                continue;
            }
            out.push(Field {
                name: n.to_string(),
                ty: field_ty(m),
            });
        }
    }
    out
}

/// The items of a `PortDef` (its `features:` along its chain), or `None` when `ty` is not a `PortDef`.
fn items_of(elements: &[RawElement], resolver: &Resolver, ty: &str) -> Option<(String, Vec<Item>)> {
    let pd = resolver
        .resolve_ref(elements, ty)
        .filter(|d| matches!(d.frontmatter.element_type, Some(ElementType::PortDef)))?;
    let mut items: Vec<Item> = Vec::new();
    for d in system::chain(elements, resolver, pd) {
        for f in d.frontmatter.features.iter().flatten() {
            let Some(m) = f.as_mapping() else { continue };
            let def = field(m, "typedBy")
                .and_then(|t| t.as_str())
                .and_then(|t| resolver.resolve_ref(elements, t));
            let (event, fields) = match def {
                Some(id) if matches!(id.frontmatter.element_type, Some(ElementType::ItemDef)) => (
                    last(&id.qualified_name).to_string(),
                    Some(item_fields(elements, resolver, id)),
                ),
                Some(_) => continue, // an attribute or a nested port, not an item
                None => match field(m, "name").and_then(|n| n.as_str()) {
                    Some(n) if field(m, "typedBy").is_none() => (n.to_string(), None),
                    _ => continue,
                },
            };
            if !items.iter().any(|i| i.event == event) {
                items.push(Item {
                    event,
                    dir: parse_dir(field(m, "direction").and_then(|v| v.as_str())),
                    fields,
                });
            }
        }
    }
    Some((pd.qualified_name.clone(), items))
}

fn conjugated(m: &Mapping) -> bool {
    field(m, "isConjugated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// The typed ports of a part definition: inline port features and child `Port` elements along its chain, nearest first.
pub fn typed_ports(
    elements: &[RawElement],
    resolver: &Resolver,
    def: &RawElement,
) -> Vec<TypedPort> {
    let mut out: Vec<TypedPort> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let chain = system::chain(elements, resolver, def);
    for d in &chain {
        for f in d.frontmatter.features.iter().flatten() {
            let Some(m) = f.as_mapping() else { continue };
            if field(m, "type").and_then(|t| t.as_str()) != Some("Port") {
                continue;
            }
            let Some(n) = field(m, "name").and_then(|n| n.as_str()) else {
                continue;
            };
            if seen.iter().any(|s| s == n) {
                continue;
            }
            seen.push(n.to_string());
            if let Some((ty, items)) = field(m, "typedBy")
                .and_then(|t| t.as_str())
                .and_then(|t| items_of(elements, resolver, t))
            {
                out.push(TypedPort {
                    name: n.to_string(),
                    ty,
                    conjugated: conjugated(m),
                    items,
                });
            }
        }
        for p in elements
            .iter()
            .filter(|e| matches!(e.frontmatter.element_type, Some(ElementType::Port)))
        {
            let Some((parent, n)) = p.qualified_name.rsplit_once("::") else {
                continue;
            };
            if parent != d.qualified_name || seen.iter().any(|s| s == n) {
                continue;
            }
            seen.push(n.to_string());
            let ty = p.frontmatter.typed_by.as_ref().and_then(|v| match v {
                Y::String(s) => Some(s.clone()),
                Y::Sequence(xs) => xs.first().and_then(|x| x.as_str()).map(String::from),
                _ => None,
            });
            if let Some((ty, items)) = ty.and_then(|t| items_of(elements, resolver, &t)) {
                out.push(TypedPort {
                    name: n.to_string(),
                    ty,
                    conjugated: p.frontmatter.is_conjugated.unwrap_or(false),
                    items,
                });
            }
        }
    }
    out
}

/// The `PortDef` (qualified name) port `name` of `def` is typed by, and whether the port is conjugated.
pub fn port_type_of(
    elements: &[RawElement],
    resolver: &Resolver,
    def: &RawElement,
    name: &str,
) -> Option<(String, bool)> {
    typed_ports(elements, resolver, def)
        .into_iter()
        .find(|p| p.name == name)
        .map(|p| (p.ty, p.conjugated))
}

/// How a port type reads in a message: `T` or `~T` (conjugated).
pub fn show(ty: &(String, bool)) -> String {
    format!("{}{}", if ty.1 { "~" } else { "" }, ty.0)
}

fn var_ty(m: &Machine, slot: usize) -> Ty {
    m.variables
        .get(slot)
        .map(|v| Ty::of_value(&v.value))
        .unwrap_or(Ty::Unknown)
}

/// The type of `e` with event-payload references `bind.f` typed from `fields` (and `None` for an unknown field).
fn infer_with(m: &Machine, e: &IrExpr, bind: Option<(&str, &[Field])>) -> Result<Ty, String> {
    let env = |n: &str| -> Ty {
        match e.refs.iter().find(|r| r.name == n) {
            Some(r) if r.kind == "var" => r.slot.map(|s| var_ty(m, s)).unwrap_or(Ty::Unknown),
            Some(r) if r.kind == "event" => match (bind, n.split_once('.')) {
                (Some((b, fs)), Some((root, f))) if root == b => fs
                    .iter()
                    .find(|x| x.name == f)
                    .map(|x| x.ty)
                    .unwrap_or(Ty::Unknown),
                _ => Ty::Unknown,
            },
            _ => Ty::Unknown,
        }
    };
    typecheck::infer(&e.ast, &env).map_err(|why| format!("`{}`: {why}", e.src))
}

/// Report what the typed fields make certainly wrong and the untyped check does not already report.
fn typed_error(m: &Machine, e: &IrExpr, bind: (&str, &[Field]), want_bool: bool) -> Option<String> {
    let plain = typecheck::infer(
        &e.ast,
        &|n: &str| match e.refs.iter().find(|r| r.name == n) {
            Some(r) if r.kind == "var" => r.slot.map(|s| var_ty(m, s)).unwrap_or(Ty::Unknown),
            _ => Ty::Unknown,
        },
    );
    if plain.is_err() {
        return None; // already E935 from the machine's own check
    }
    match infer_with(m, e, Some(bind)) {
        Err(why) => Some(why),
        Ok(t)
            if want_bool
                && !matches!(t, Ty::Bool | Ty::Unknown)
                && matches!(plain, Ok(Ty::Unknown | Ty::Bool)) =>
        {
            Some(format!("`{}` is {}, not a boolean", e.src, t.describe()))
        }
        _ => None,
    }
}

fn exprs_of<'a>(acts: &'a [Action], out: &mut Vec<(&'a IrExpr, Option<usize>)>) {
    for a in acts {
        match a {
            Action::Assign { var, expr } => out.push((expr, Some(*var))),
            Action::Send { payload, .. } => out.extend(payload.iter().map(|(_, e)| (e, None))),
            Action::Behavior { .. } => {}
        }
    }
}

fn walk_actions<'a>(m: &'a Machine) -> Vec<(String, &'a [Action])> {
    let mut v: Vec<(String, &[Action])> = vec![("initialEffect".into(), &m.initial_actions[..])];
    for s in &m.states {
        v.push((format!("state '{}' entry", s.path), &s.entry[..]));
        v.push((format!("state '{}' exit", s.path), &s.exit[..]));
    }
    for t in &m.transitions {
        v.push((transition_ctx(m, t), &t.actions[..]));
    }
    v
}

fn transition_ctx(m: &Machine, t: &ir::Transition) -> String {
    format!(
        "transition '{}' → '{}'",
        m.states[t.source].path,
        t.target
            .map(|x| m.states[x].path.as_str())
            .unwrap_or("(internal)")
    )
}

/// The findings for one part definition that exhibits a state machine and has typed ports. `holder` names it.
pub fn findings(elements: &[RawElement], resolver: &Resolver, def: &RawElement) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some(machine) = def.frontmatter.exhibits_states.iter().flatten().next() else {
        return out;
    };
    let ports = typed_ports(elements, resolver, def);
    if ports.is_empty() {
        return out;
    }
    let Some(sm) = resolver.resolve_ref(elements, machine) else {
        return out;
    };
    let Ok(m) = ir::lower(elements, &sm.qualified_name) else {
        return out;
    };
    let q = &def.qualified_name;

    // sends
    for (ctx, acts) in walk_actions(&m) {
        for a in acts {
            let Action::Send {
                event,
                via: Some(p),
                payload,
            } = a
            else {
                continue;
            };
            let Some(tp) = ports.iter().find(|x| &x.name == p) else {
                continue;
            };
            let Some(item) = tp.item(event) else {
                out.push(("E932", format!("{q}: {ctx} sends {event} via '{p}', but its type '{}' carries no {event} (it carries {})", tp.ty, tp.carried())));
                continue;
            };
            if let Some(f) = tp.flow(item).filter(|f| !f.sends()) {
                let _ = f;
                out.push((
                    "E932",
                    format!(
                        "{q}: {ctx} sends {event} via '{p}', but '{}' carries {event} inward{}",
                        tp.ty,
                        if tp.conjugated {
                            " at this conjugated port"
                        } else {
                            ""
                        }
                    ),
                ));
            }
            let Some(fields) = &item.fields else { continue };
            for (k, e) in payload {
                let Some(fd) = fields.iter().find(|x| &x.name == k) else {
                    out.push(("E932", format!("{q}: {ctx} sends {event} via '{p}' with payload field '{k}', which {event} does not have (it has {})", names(fields))));
                    continue;
                };
                if let Ok(t) = infer_with(&m, e, None) {
                    if let Some(why) = mismatch(fd.ty, t) {
                        out.push((
                            "E935",
                            format!(
                                "{q}: {ctx}: payload '{k}' of {event} is {}, but `{}` is {why}",
                                fd.ty.describe(),
                                e.src
                            ),
                        ));
                    }
                }
            }
        }
    }

    // accepts
    for t in &m.transitions {
        let Trigger::Event { name, bind } = &t.trigger else {
            continue;
        };
        let Some((p, event)) = name.split_once('/') else {
            continue;
        };
        let Some(tp) = ports.iter().find(|x| x.name == p) else {
            continue;
        };
        let ctx = transition_ctx(&m, t);
        let Some(item) = tp.item(event) else {
            out.push(("E932", format!("{q}: {ctx} accepts {event} via '{p}', but its type '{}' carries no {event} (it carries {})", tp.ty, tp.carried())));
            continue;
        };
        if tp.flow(item).is_some_and(|f| !f.receives()) {
            out.push((
                "E932",
                format!(
                    "{q}: {ctx} accepts {event} via '{p}', but '{}' carries {event} outward{}",
                    tp.ty,
                    if tp.conjugated {
                        " at this conjugated port"
                    } else {
                        ""
                    }
                ),
            ));
        }
        let (Some(b), Some(fields)) = (bind.as_deref(), &item.fields) else {
            continue;
        };
        let mut exprs: Vec<(&IrExpr, Option<usize>)> = Vec::new();
        if let Some(g) = &t.guard {
            exprs.push((g, None));
        }
        exprs_of(&t.actions, &mut exprs);
        for (i, (e, var)) in exprs.iter().enumerate() {
            for r in e.refs.iter().filter(|r| r.kind == "event") {
                if let Some((root, f)) = r.name.split_once('.') {
                    let f = f.split('.').next().unwrap_or(f);
                    if root == b && !fields.iter().any(|x| x.name == f) {
                        out.push((
                            "E932",
                            format!(
                                "{q}: {ctx} uses '{}', but {event} has no field '{f}' (it has {})",
                                r.name,
                                names(fields)
                            ),
                        ));
                    }
                }
            }
            let is_guard = i == 0 && t.guard.is_some();
            if let Some(why) = typed_error(&m, e, (b, fields), is_guard) {
                out.push(("E935", format!("{q}: {ctx}: {why}")));
            } else if let Some(v) = var {
                let lhs = var_ty(&m, *v);
                if let (Ok(rhs), Ok(plain)) = (
                    infer_with(&m, e, Some((b, fields))),
                    infer_with(&m, e, None),
                ) {
                    if typecheck::assign_mismatch(lhs, plain).is_none() {
                        if let Some(why) = typecheck::assign_mismatch(lhs, rhs) {
                            out.push((
                                "E935",
                                format!(
                                    "{q}: {ctx}: `{} := {}` assigns {why}",
                                    m.variables[*v].name, e.src
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    out
}

fn names(fields: &[Field]) -> String {
    if fields.is_empty() {
        "no fields".into()
    } else {
        fields
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Why a value of type `got` cannot fill a field of type `want`, if it certainly cannot.
fn mismatch(want: Ty, got: Ty) -> Option<String> {
    match (want, got) {
        (Ty::Num(Some(a)), Ty::Num(Some(b))) if a != b => {
            Some(format!("{}", Ty::Num(Some(b)).describe()))
        }
        (Ty::Num(_), Ty::Bool | Ty::Str)
        | (Ty::Bool, Ty::Num(_) | Ty::Str)
        | (Ty::Str, Ty::Num(_) | Ty::Bool) => Some(got.describe()),
        _ => None,
    }
}
