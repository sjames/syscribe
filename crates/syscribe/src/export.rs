//! `syscribe model export` — structured, machine-readable dump of the whole
//! model graph (issue #2).
//!
//! Emits every element with its identity, typed frontmatter, and resolved
//! relationships (the reverse indices the validator already computes), so
//! external tools — CI gates, dashboards, LLM agents — can consume the model
//! without re-implementing the parser.

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::validator;

/// Stable schema version for consumers to key on. Bump on breaking changes.
pub const SCHEMA_VERSION: &str = "1.0";

/// Recursively drop `null` object members so the export carries only the
/// frontmatter fields that are actually set.
fn strip_nulls(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let cleaned: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .filter(|(_, val)| !val.is_null())
                .map(|(k, val)| (k, strip_nulls(val)))
                .collect();
            serde_json::Value::Object(cleaned)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(strip_nulls).collect())
        }
        other => other,
    }
}

/// Build the JSON object for a single element.
fn element_json(
    elem: &RawElement,
    result: &validator::ValidationResult,
    config: &ValidateConfig,
    endpoints: &std::collections::HashMap<String, Vec<syscribe_model::connections::ConnectionEndpoint>>,
) -> serde_json::Value {
    let fm = &elem.frontmatter;

    // Typed frontmatter with unset fields removed.
    let frontmatter = strip_nulls(serde_json::to_value(fm).unwrap_or(serde_json::Value::Null));

    // Resolved reverse-index relationships, keyed by the element's stable id.
    let mut computed = serde_json::Map::new();
    if let Some(id) = fm.id.as_deref() {
        if let Some(tcs) = result.verified_by.get(id) {
            computed.insert("verifiedBy".into(), serde_json::json!(tcs));
        }
        if let Some(children) = result.derived_children.get(id) {
            computed.insert("derivedChildren".into(), serde_json::json!(children));
        }
    }
    // REQ-TRS-MG-001/002 — refinedBy / actorIn key by stable id when present,
    // else qualified name (matching the validator's index_key).
    let mg_key = fm.id.as_deref().unwrap_or(elem.qualified_name.as_str());
    if let Some(ucs) = result.refined_by.get(mg_key) {
        computed.insert("refinedBy".into(), serde_json::json!(ucs));
    }
    if let Some(ucs) = result.actor_in.get(mg_key) {
        computed.insert("actorIn".into(), serde_json::json!(ucs));
    }
    // REQ-TRS-MG-008 — mopRefinedBy: the MoPs refining this MoE.
    if let Some(mops) = result.mop_refined_by.get(mg_key) {
        computed.insert("mopRefinedBy".into(), serde_json::json!(mops));
    }
    // REQ-TRS-ALLOC-001 — allocatedFrom: the sources allocated to this target,
    // over both authoring forms. Keyed by stable id else qname.
    if let Some(srcs) = result.allocated_from.get(mg_key) {
        computed.insert("allocatedFrom".into(), serde_json::json!(srcs));
    }

    let mut obj = serde_json::Map::new();
    obj.insert("qname".into(), serde_json::json!(elem.qualified_name));
    obj.insert("file".into(), serde_json::json!(elem.file_path));
    // REQ-TRS-LINK-001/004 — hosted source URL when `[links]` is configured.
    if let Some(url) = config.hosted_url_for(
        &elem.file_path,
        &elem.qualified_name,
        fm.id.as_deref().unwrap_or(""),
    ) {
        obj.insert("url".into(), serde_json::json!(url));
    }
    if let Some(id) = &fm.id {
        obj.insert("id".into(), serde_json::json!(id));
    }
    obj.insert(
        "type".into(),
        serde_json::to_value(&fm.element_type).unwrap_or(serde_json::Value::Null),
    );
    if let Some(name) = &fm.name {
        obj.insert("name".into(), serde_json::json!(name));
    }
    // GH #486: a connection's endpoints beside its kind (`typedBy` is in the frontmatter).
    // `source`/`target` are the first two ends; `ends` lists every end (n-ary included).
    if let Some(eps) = endpoints.get(&elem.qualified_name).filter(|e| e.len() >= 2) {
        obj.insert("source".into(), serde_json::json!(eps[0].chain));
        obj.insert("target".into(), serde_json::json!(eps[1].chain));
        let ends: Vec<_> = eps
            .iter()
            .map(|e| match &e.role {
                Some(r) => serde_json::json!({"end": r, "binds": e.chain}),
                None => serde_json::json!({"binds": e.chain}),
            })
            .collect();
        obj.insert("ends".into(), serde_json::Value::Array(ends));
    }
    obj.insert("frontmatter".into(), frontmatter);
    if !computed.is_empty() {
        obj.insert("computed".into(), serde_json::Value::Object(computed));
    }
    serde_json::Value::Object(obj)
}

/// What `export --resolved` adds (GH #529, `REQ-TRS-EXPRES-001`): everything a document generator would
/// otherwise re-derive from the files — bodies, effective parameter values, reverse links, questions and the
/// build identity. Purely additive to the plain export.
pub struct Resolved<'a> {
    pub model_root: &'a std::path::Path,
    /// The unprojected model: a Configuration's values are read here even when `--config` projects the view.
    pub full: &'a [RawElement],
    /// The `--config` argument, when given.
    pub config: Option<&'a str>,
}

fn label(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone())
}

/// The effective parameters of Configuration `full[idx]`: one row per parameter with its value and where it
/// came from (`provenance`), joined with the parameter's declaration (unit, range, type, binding time).
fn effective_parameters(full: &[RawElement], idx: usize) -> Vec<serde_json::Value> {
    use syscribe_model::element::ElementType;
    // `<FeatureDef qname>.<param>` -> (feature element, declaration)
    let mut decl: std::collections::HashMap<String, (&RawElement, &serde_yaml::Mapping)> = Default::default();
    for fd in full.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::FeatureDef)) {
        for p in fd.frontmatter.parameters.iter().flatten() {
            let serde_yaml::Value::Mapping(m) = p else { continue };
            if let Some(name) = m.get(serde_yaml::Value::String("name".into())).and_then(|v| v.as_str()) {
                decl.insert(format!("{}.{}", fd.qualified_name, name), (fd, m));
            }
        }
    }
    let prov = syscribe_model::provenance::to_json(&syscribe_model::provenance::provenance(full, idx));
    let mut rows = Vec::new();
    for q in prov["parameters"].as_array().into_iter().flatten() {
        let key = q["key"].as_str().unwrap_or_default();
        let (feature, name) = key.rsplit_once('.').unwrap_or((key, ""));
        let mut row = serde_json::Map::new();
        row.insert("key".into(), key.into());
        row.insert("feature".into(), feature.into());
        row.insert("name".into(), name.into());
        row.insert("value".into(), q["value"].clone());
        if let Some((fd, m)) = decl.get(key) {
            if let Some(id) = &fd.frontmatter.id {
                row.insert("featureId".into(), id.clone().into());
            }
            let get = |k: &str| m.get(serde_yaml::Value::String(k.into()));
            for k in ["unit", "type", "bindingTime", "latestBindingTime", "description"] {
                if let Some(v) = get(k).and_then(|v| v.as_str()) {
                    row.insert(k.into(), v.into());
                }
            }
            if let Some(r) = get("range").and_then(|v| v.as_str()) {
                let bounds = r.split_once("..").and_then(|(lo, hi)| {
                    let hi = hi.trim().strip_prefix('=').unwrap_or(hi.trim()).trim();
                    Some((lo.trim().parse::<f64>().ok()?, hi.parse::<f64>().ok()?))
                });
                row.insert(
                    "range".into(),
                    match bounds {
                        Some((lo, hi)) => serde_json::json!({ "min": lo, "max": hi }),
                        None => r.into(),
                    },
                );
            }
        }
        row.insert("source".into(), q["source"]["kind"].clone());
        if let Some(from) = q["source"].get("from") {
            row.insert("from".into(), from.clone());
        }
        if let Some(expr) = q["source"].get("expr") {
            row.insert("expression".into(), expr.clone());
        }
        if !q["stage"].is_null() {
            row.insert("stage".into(), q["stage"].clone());
        }
        rows.push(serde_json::Value::Object(row));
    }
    rows
}

/// The indices `--resolved` reads per element, built once.
struct ResolvedIndex<'a> {
    opts: &'a Resolved<'a>,
    resolver: syscribe_model::resolver::Resolver,
    /// requirement qname -> satisfying architecture elements
    satisfied_by: std::collections::HashMap<String, Vec<String>>,
    /// target qname -> citing elements with the reference as written
    cited_by: std::collections::HashMap<String, Vec<serde_json::Value>>,
    policy_human: bool,
}

impl<'a> ResolvedIndex<'a> {
    fn new(elements: &[RawElement], opts: &'a Resolved<'a>) -> Self {
        let resolver = syscribe_model::resolver::Resolver::new(elements);
        let by_qname: std::collections::HashMap<&str, &RawElement> = elements.iter().map(|e| (e.qualified_name.as_str(), e)).collect();
        let satisfied_by = syscribe_model::models::satisfiers(elements, &resolver)
            .into_iter()
            .map(|(req, list)| {
                let mut ids: Vec<String> = list.iter().map(|q| by_qname.get(q.as_str()).map_or(q.clone(), |e| label(e))).collect();
                ids.sort();
                (req, ids)
            })
            .collect();
        let mut cited_by: std::collections::HashMap<String, Vec<serde_json::Value>> = Default::default();
        for c in syscribe_model::req_items::all_citations(elements, &resolver) {
            let row = serde_json::json!({ "by": label(c.source), "ref": c.citation.raw });
            let list = cited_by.entry(c.target_qname).or_default();
            if !list.contains(&row) {
                list.push(row);
            }
        }
        let policy = syscribe_model::agent_policy::load(opts.model_root);
        let policy_human = syscribe_model::agent_policy::default_human(policy.as_ref());
        ResolvedIndex { opts, resolver, satisfied_by, cited_by, policy_human }
    }

    /// Add the resolved members to one element's object.
    fn extend(&self, elements: &[RawElement], elem: &RawElement, obj: &mut serde_json::Map<String, serde_json::Value>) {
        use syscribe_model::element::ElementType;
        obj.insert(
            "body".into(),
            serde_json::json!({
                "raw": elem.doc,
                "rendered": syscribe_model::value_refs::render(elements, &self.resolver, &elem.doc),
            }),
        );
        let mut extra = serde_json::Map::new();
        if let Some(list) = self.satisfied_by.get(&elem.qualified_name) {
            extra.insert("satisfiedBy".into(), serde_json::json!(list));
        }
        if let Some(list) = self.cited_by.get(&elem.qualified_name) {
            extra.insert("citedBy".into(), serde_json::json!(list));
        }
        if !extra.is_empty() {
            let computed = obj.entry("computed").or_insert_with(|| serde_json::Value::Object(Default::default()));
            if let Some(c) = computed.as_object_mut() {
                c.extend(extra);
            }
        }
        match elem.frontmatter.element_type {
            Some(ElementType::PlanningItem) => {
                let qs = syscribe_model::questions::parse(elem).0;
                if !qs.is_empty() {
                    let open = qs.iter().filter(|q| q.status == "open").count();
                    let rows: Vec<serde_json::Value> = qs
                        .iter()
                        .map(|q| {
                            let mut v = serde_json::to_value(q).unwrap_or_default();
                            v["ref"] = serde_json::json!(format!("{}#{}", q.item, q.id));
                            v["answerBy"] = serde_json::json!(q.effective_answer_by(self.policy_human));
                            v
                        })
                        .collect();
                    obj.insert("questions".into(), serde_json::Value::Array(rows));
                    obj.insert("openQuestions".into(), open.into());
                }
            }
            Some(ElementType::Configuration) => {
                if let Some(idx) = self.opts.full.iter().position(|e| e.qualified_name == elem.qualified_name) {
                    obj.insert("effectiveParameters".into(), serde_json::Value::Array(effective_parameters(self.opts.full, idx)));
                }
            }
            _ => {}
        }
    }

    /// The top-level members: `meta`, and under `--config` that Configuration's `effectiveParameters`.
    fn top(&self) -> serde_json::Map<String, serde_json::Value> {
        let git = |args: &[&str]| syscribe_model::config::git_output(self.opts.model_root, args);
        let commit = git(&["rev-parse", "HEAD"]);
        // `git status --porcelain` prints nothing for a clean tree, which `git_output` reads as `None`.
        let dirty = commit.as_ref().map(|_| git(&["status", "--porcelain"]).is_some());
        let cfg_idx = self.opts.config.and_then(|c| crate::residual::find_configuration(self.opts.full, c));
        let mut top = serde_json::Map::new();
        top.insert("resolved".into(), true.into());
        top.insert(
            "meta".into(),
            serde_json::json!({
                "commit": commit,
                "dirty": dirty,
                "syscribeVersion": env!("CARGO_PKG_VERSION"),
                "generatedAt": crate::claim::iso8601_now(),
                "config": cfg_idx.map(|i| label(&self.opts.full[i])),
            }),
        );
        if let Some(i) = cfg_idx {
            top.insert("effectiveParameters".into(), serde_json::Value::Array(effective_parameters(self.opts.full, i)));
        }
        top
    }
}

/// `export` subcommand. `ndjson` switches from a single pretty JSON document to
/// newline-delimited JSON (one header line, then one element per line).
pub fn cmd_export(elements: &[RawElement], config: &ValidateConfig, ndjson: bool, resolved: Option<&Resolved<'_>>) {
    let result = validator::validate_with_config(elements, config);
    let endpoints = syscribe_model::connections::endpoint_index(elements);
    let index = resolved.map(|r| ResolvedIndex::new(elements, r));
    let one = |e: &RawElement| {
        let mut v = element_json(e, &result, config, &endpoints);
        if let (Some(ix), Some(obj)) = (&index, v.as_object_mut()) {
            ix.extend(elements, e, obj);
        }
        v
    };
    let mut head = serde_json::Map::new();
    head.insert("schemaVersion".into(), SCHEMA_VERSION.into());
    if ndjson {
        head.insert("kind".into(), "header".into());
    }
    head.insert("elementCount".into(), elements.len().into());
    if let Some(ix) = &index {
        head.extend(ix.top());
    }

    if ndjson {
        println!("{}", serde_json::to_string(&serde_json::Value::Object(head)).unwrap());
        for elem in elements {
            println!("{}", serde_json::to_string(&one(elem)).unwrap());
        }
        return;
    }

    head.insert("elements".into(), serde_json::Value::Array(elements.iter().map(one).collect()));
    println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(head)).unwrap());
}
