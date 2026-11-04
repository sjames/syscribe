//! `timeline <Configuration>` — what each stage of a `derivedFrom:` chain decided, with provenance (GH #418,
//! `REQ-TRS-TIMELINE-001`). One payload for the CLI, the MCP tool `timeline` and the LSP request `syscribe/timeline`.
//!
//! The chain runs from the root Configuration to the one asked for. Each step lists only what it decided itself (its
//! own `features:` and `parameterBindings:`), so the steps add up to the effective values, which are listed with their
//! provenance (source kind and the stage that bound them). What no step has decided yet is `open`.

use std::collections::BTreeSet;

use serde_json::{json, Value};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;

pub type Fail = (i32, String);

fn ident(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

pub fn compute(elements: &[RawElement], conf: &str) -> Result<Value, Fail> {
    let resolver = Resolver::new(elements);
    let find = |k: &str| elements.iter().position(|e| e.frontmatter.element_type == Some(ElementType::Configuration) && (e.frontmatter.id.as_deref() == Some(k) || e.qualified_name == k));
    let leaf = find(conf).ok_or_else(|| (1, format!("Configuration not found: {conf}")))?;

    // leaf -> root along the first derivedFrom, guarding against a cycle.
    let mut chain: Vec<usize> = vec![leaf];
    let mut seen: BTreeSet<usize> = BTreeSet::from([leaf]);
    while let Some(parent) = elements[*chain.last().unwrap()]
        .frontmatter
        .derived_from
        .iter()
        .flatten()
        .next()
        .and_then(|r| resolver.resolve_ref(elements, r))
        .and_then(|p| elements.iter().position(|e| std::ptr::eq(e, p)))
        .filter(|i| elements[*i].frontmatter.element_type == Some(ElementType::Configuration))
    {
        if !seen.insert(parent) {
            break;
        }
        chain.push(parent);
    }
    chain.reverse();

    let steps: Vec<Value> = chain
        .iter()
        .map(|&i| {
            let e = &elements[i];
            let selections: Vec<Value> = e.frontmatter.declared_feature_selections().into_iter().map(|(f, s)| json!({ "feature": f, "selected": s })).collect();
            let bindings: Vec<Value> = match &e.frontmatter.parameter_bindings {
                Some(serde_yaml::Value::Mapping(m)) => m
                    .iter()
                    .filter_map(|(k, v)| Some(json!({ "key": k.as_str()?, "value": serde_json::to_value(v).unwrap_or(Value::Null) })))
                    .collect(),
                _ => vec![],
            };
            json!({
                "configuration": ident(e),
                "file": e.file_path,
                "bindingStage": e.frontmatter.binding_stage,
                "decided": { "selections": selections, "bindings": bindings },
            })
        })
        .collect();

    let prov = syscribe_model::provenance::to_json(&syscribe_model::provenance::provenance(elements, leaf));
    let open = crate::residual::residual_for(elements, conf)
        .map(|r| r.to_json()["groups"].clone())
        .unwrap_or_else(|| json!([]));
    Ok(json!({
        "schemaVersion": "1.0",
        "configuration": ident(&elements[leaf]),
        "bindingStage": elements[leaf].frontmatter.binding_stage,
        "chain": steps,
        "effective": { "selections": prov["selections"], "parameters": prov["parameters"] },
        "open": open,
    }))
}

pub fn cmd_timeline(elements: &[RawElement], args: &[String]) -> i32 {
    let json_out = args.iter().any(|a| a == "--json");
    if let Some(bad) = args.iter().find(|a| a.starts_with('-') && *a != "--json") {
        eprintln!("timeline: unknown option '{bad}'");
        return 2;
    }
    let pos: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    let [conf] = pos.as_slice() else {
        eprintln!("Usage: syscribe -m <root> timeline <Configuration> [--json]");
        return 2;
    };
    let out = match compute(elements, conf) {
        Ok(o) => o,
        Err((c, m)) => {
            eprintln!("{m}");
            return c;
        }
    };
    if json_out {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return 0;
    }
    println!("# Timeline of {}\n", out["configuration"].as_str().unwrap_or(""));
    for s in out["chain"].as_array().into_iter().flatten() {
        println!("## {} — stage {}", s["configuration"].as_str().unwrap_or(""), s["bindingStage"].as_str().unwrap_or("(none)"));
        for x in s["decided"]["selections"].as_array().into_iter().flatten() {
            println!("- select {} = {}", x["feature"].as_str().unwrap_or(""), x["selected"]);
        }
        for x in s["decided"]["bindings"].as_array().into_iter().flatten() {
            println!("- bind {} = {}", x["key"].as_str().unwrap_or(""), x["value"]);
        }
        println!();
    }
    let open: Vec<&Value> = out["open"].as_array().into_iter().flatten().flat_map(|g| g["items"].as_array().into_iter().flatten()).collect();
    println!("## Still open ({})", open.len());
    for i in open {
        println!("- {} {} (bindable {}..{})", i["kind"].as_str().unwrap_or(""), i["key"].as_str().unwrap_or(""), i["window"]["earliest"].as_str().unwrap_or(""), i["window"]["latest"].as_str().unwrap_or(""));
    }
    0
}
