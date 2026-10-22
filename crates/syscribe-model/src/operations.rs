//! Operations bound to behaviour (`REQ-TRS-OPS-001`..`-003`, GH #364): `errors:` on an operation and
//! `realizedBy:`, the `ActionDef` that implements it, with the check that its signature matches.
//!
//! Everything here is opt-in: an `operations:` entry that declares neither field is not looked at.

use serde_yaml::Value as Yaml;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// `(is_error, code, file, message)`.
pub type Finding = (bool, &'static str, String, String);

fn s<'a>(m: &'a serde_yaml::Mapping, k: &str) -> Option<&'a str> {
    m.get(k).and_then(Yaml::as_str)
}

fn refs(v: &Yaml) -> Vec<String> {
    match v {
        Yaml::String(x) => vec![x.clone()],
        Yaml::Sequence(q) => q.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

/// The qualified name `r` resolves to, else `r` as written without a standard-library prefix.
fn canon(elements: &[RawElement], resolver: &Resolver, r: &str) -> String {
    resolver
        .resolve_ref(elements, r)
        .map(|e| e.qualified_name.clone())
        .unwrap_or_else(|| r.trim().strip_prefix("ScalarValues::").unwrap_or(r.trim()).to_string())
}

struct Param<'a> {
    name: &'a str,
    direction: &'a str,
    typed_by: Option<&'a str>,
    unit: Option<&'a str>,
}

fn params<'a>(list: Option<&'a Vec<Yaml>>, default_direction: &'static str) -> Vec<Param<'a>> {
    list.into_iter()
        .flatten()
        .filter_map(|p| p.as_mapping())
        .filter_map(|m| {
            Some(Param {
                name: s(m, "name")?,
                direction: s(m, "direction").unwrap_or(default_direction),
                typed_by: s(m, "typedBy").or_else(|| s(m, "type")),
                unit: s(m, "unit"),
            })
        })
        .collect()
}

fn is_value_type(e: &RawElement) -> bool {
    matches!(
        e.frontmatter.element_type,
        Some(ElementType::ItemDef | ElementType::AttributeDef | ElementType::EnumerationDef | ElementType::Enumeration)
    )
}

pub fn check(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    for owner in elements.iter().filter(|e| e.frontmatter.operations.is_some()) {
        let file = owner.file_path.clone();
        for op in owner.frontmatter.operations.iter().flatten().filter_map(|o| o.as_mapping()) {
            let Some(name) = s(op, "name") else { continue };
            let is_async = op.get("isAsync").and_then(Yaml::as_bool) == Some(true);

            // errors
            if let Some(errs) = op.get("errors") {
                let errs = refs(errs);
                if is_async && !errs.is_empty() {
                    out.push((true, "E670", file.clone(), format!("asynchronous operation '{name}' of '{}' returns nothing, so it cannot declare errors", owner.qualified_name)));
                }
                for r in errs {
                    let ok = resolver.resolve_ref(elements, &r).is_some_and(is_value_type)
                        || (resolver.resolve_ref(elements, &r).is_none() && !matches!(crate::resolver::builtin_type_kind(&r), crate::resolver::BuiltinType::NotBuiltin));
                    if !ok {
                        let why = if resolver.resolve_ref(elements, &r).is_none() { "does not resolve" } else { "is not an ItemDef, AttributeDef or enumeration" };
                        out.push((true, "E670", file.clone(), format!("operation '{name}' of '{}': error '{r}' {why}", owner.qualified_name)));
                    }
                }
            }

            // realizedBy
            let Some(target) = s(op, "realizedBy") else { continue };
            let Some(act) = resolver.resolve_ref(elements, target).filter(|a| a.frontmatter.element_type == Some(ElementType::ActionDef)) else {
                let why = if resolver.resolve_ref(elements, target).is_none() { "does not resolve" } else { "is not an ActionDef" };
                out.push((true, "E671", file.clone(), format!("operation '{name}' of '{}': realizedBy '{target}' {why}", owner.qualified_name)));
                continue;
            };
            let at = |detail: String| format!("operation '{name}' of '{}' is realised by '{}' but {detail}", owner.qualified_name, act.qualified_name);
            let op_params = params(op.get("parameters").and_then(Yaml::as_sequence), "in");
            let act_all = params(act.frontmatter.parameters.as_ref(), "in");
            let (act_ret, act_params): (Vec<&Param>, Vec<&Param>) = act_all.iter().partition(|p| p.direction == "return");

            for p in &op_params {
                let Some(a) = act_params.iter().find(|a| a.name == p.name) else {
                    out.push((true, "E672", file.clone(), at(format!("the action lacks parameter '{}'", p.name))));
                    continue;
                };
                if a.direction != p.direction {
                    out.push((true, "E672", file.clone(), at(format!("parameter '{}' direction differs: operation {}, action {}", p.name, p.direction, a.direction))));
                }
                if let (Some(x), Some(y)) = (p.typed_by, a.typed_by) {
                    if canon(elements, resolver, x) != canon(elements, resolver, y) {
                        out.push((true, "E672", file.clone(), at(format!("parameter '{}' type differs: operation {x}, action {y}", p.name))));
                    }
                }
                if let (Some(x), Some(y)) = (p.unit, a.unit) {
                    if x.trim() != y.trim() {
                        out.push((true, "E672", file.clone(), at(format!("parameter '{}' unit differs: operation {x}, action {y}", p.name))));
                    }
                }
            }
            for a in &act_params {
                if !op_params.iter().any(|p| p.name == a.name) {
                    out.push((true, "E672", file.clone(), at(format!("the action has parameter '{}' that the operation does not", a.name))));
                }
            }

            // the return
            let ret = op.get("returnType").and_then(Yaml::as_str);
            match (ret, act_ret.first(), is_async) {
                (_, Some(r), true) => out.push((true, "E672", file.clone(), at(format!("the operation is asynchronous and returns nothing, but the action has return parameter '{}'", r.name)))),
                (Some(t), None, false) => out.push((true, "E672", file.clone(), at(format!("the operation returns {t} but the action has no return parameter")))),
                (None, Some(r), false) => out.push((true, "E672", file.clone(), at(format!("the operation returns nothing but the action has return parameter '{}'", r.name)))),
                (Some(t), Some(r), false) => {
                    if let Some(rt) = r.typed_by {
                        if canon(elements, resolver, t) != canon(elements, resolver, rt) {
                            out.push((true, "E672", file.clone(), at(format!("the return type differs: operation {t}, action {rt}"))));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    check_calls(elements, resolver, &mut out);
    out
}

/// The operation `r` (`<owner>::<name>`) names: the owner that declares it and the operation mapping.
pub(crate) fn find_operation<'a>(elements: &'a [RawElement], resolver: &Resolver, r: &str) -> Result<(&'a RawElement, &'a serde_yaml::Mapping), String> {
    let r = r.trim();
    let (owner_ref, op_name) = r.rsplit_once("::").ok_or_else(|| format!("'{r}' is not `<owner>::<operation>`"))?;
    let owner = resolver
        .resolve_ref(elements, owner_ref)
        .ok_or_else(|| format!("'{owner_ref}' in '{r}' does not resolve"))?;
    owner
        .frontmatter
        .operations
        .iter()
        .flatten()
        .filter_map(Yaml::as_mapping)
        .find(|o| s(o, "name") == Some(op_name))
        .map(|op| (owner, op))
        .ok_or_else(|| format!("'{}' declares no operation '{op_name}'", owner.qualified_name))
}

/// Every sub-action mapping in a tree, recursing through `then`, `else`, `body` and `subActions`.
fn walk_subs<'a>(list: &'a [Yaml], f: &mut impl FnMut(&'a serde_yaml::Mapping)) {
    for sa in list {
        let Some(m) = sa.as_mapping() else { continue };
        f(m);
        for branch in ["then", "else", "body", "subActions"] {
            if let Some(seq) = m.get(branch).and_then(Yaml::as_sequence) {
                walk_subs(seq, f);
            }
        }
    }
}

/// Call sites that name an operation (`REQ-TRS-OPCALL-001`..`-002`, GH #398).
fn check_calls(elements: &[RawElement], resolver: &Resolver, out: &mut Vec<Finding>) {
    for act in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::ActionDef)) {
        let Some(subs) = act.frontmatter.sub_actions.as_deref() else { continue };
        walk_subs(subs, &mut |m| {
            let Some(r) = s(m, "operation") else { return };
            let who = format!("sub-action '{}' of '{}'", s(m, "name").unwrap_or("?"), act.qualified_name);
            if !matches!(s(m, "kind"), Some("SendAction" | "PerformAction")) {
                out.push((true, "E673", act.file_path.clone(), format!("{who}: `operation:` is only valid on a SendAction or PerformAction")));
                return;
            }
            let op = match find_operation(elements, resolver, r) {
                Ok((_, op)) => op,
                Err(why) => {
                    out.push((true, "E673", act.file_path.clone(), format!("{who}: operation {why}")));
                    return;
                }
            };
            let op_params = params(op.get("parameters").and_then(Yaml::as_sequence), "in");
            let given: Vec<&str> = m
                .get("parameters")
                .and_then(Yaml::as_sequence)
                .into_iter()
                .flatten()
                .filter_map(|p| p.as_mapping().and_then(|p| s(p, "name")))
                .collect();
            for g in &given {
                if !op_params.iter().any(|p| p.name == *g && p.direction != "return" && p.direction != "out") {
                    out.push((true, "E674", act.file_path.clone(), format!("{who}: calls '{r}' with argument '{g}', which is not an input parameter of the operation")));
                }
            }
            // REQ-TRS-OPCALL-003 — the kind and unit dimension of each argument value
            for arg in m.get("parameters").and_then(Yaml::as_sequence).into_iter().flatten().filter_map(Yaml::as_mapping) {
                let (Some(name), Some(value)) = (s(arg, "name"), arg.get("value").map(|v| match v { Yaml::String(x) => x.clone(), other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string() })) else { continue };
                let Some(decl) = op.get("parameters").and_then(Yaml::as_sequence).into_iter().flatten().filter_map(Yaml::as_mapping).find(|p| s(p, "name") == Some(name)) else { continue };
                let Ok(ast) = crate::expr::parse(&value) else { continue };
                let Ok(got) = crate::typecheck::infer(&ast, &|_: &str| crate::typecheck::Ty::Unknown) else { continue };
                if let Some(why) = crate::port_types::mismatch(crate::port_types::field_ty(decl), got) {
                    out.push((true, "E675", act.file_path.clone(), format!("{who}: calls '{r}' with `{name}: {value}`, which is {why}, not what the parameter declares")));
                }
            }
            for p in op_params.iter().filter(|p| p.direction == "in" || p.direction == "inout") {
                if !given.contains(&p.name) {
                    out.push((true, "E674", act.file_path.clone(), format!("{who}: calls '{r}' without its parameter '{}'", p.name)));
                }
            }
        });
    }
}
