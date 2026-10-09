//! SBOM generation (§18, GH #66). Read-only: emits a CycloneDX 1.6 or SPDX 2.3 JSON
//! Software Bill of Materials from the `implementedBy:` links on Part/PartDef elements
//! (and, with `--include-tests`, `TestCase.sourceFile:`). Local paths become file
//! components; `<registry>:<pkg>@<version>` values become external package components.

use std::time::{SystemTime, UNIX_EPOCH};
use syscribe_model::{
    element::{ElementType, RawElement},
    resolver::Resolver,
};

fn is_part(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::PartDef) | Some(ElementType::Part))
}

/// (year, month, day) from days since the Unix epoch (Howard Hinnant's civil algorithm).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn iso8601_now() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, rem / 3600, (rem % 3600) / 60, rem % 60)
}

/// A pseudo-UUID (valid format) seeded from the current time — unique per generation.
fn serial_uuid() -> String {
    let n = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let h = (n as u64).wrapping_mul(0x9E3779B97F4A7C15);
    let lo = (n >> 64) as u64 ^ h.rotate_left(21);
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        h & 0xffff_ffff,
        (h >> 32) & 0xffff,
        (h >> 48) & 0xfff,
        (lo & 0x3fff) | 0x8000,
        lo & 0xffff_ffff_ffff
    )
}

struct Component {
    name: String,
    version: Option<String>,
    purl: Option<String>,
    location: Option<String>, // local file path
    requirements: Vec<String>, // requirement ids this component traces to
    owners: Vec<String>,       // qualified names of the Part/PartDef elements that declare it
}

/// Parse `<registry>:<package>@<version>[#path]` into a remote component, or `None` for a
/// local path. The registry prefixes and grammar are the shared
/// `syscribe_model::config::parse_package_ref` definition — the same one the
/// validator uses to treat these values as external (no `W023`, GH #134).
fn parse_remote(v: &str) -> Option<Component> {
    let r = syscribe_model::config::parse_package_ref(v)?;
    Some(Component {
        name: r.package.to_string(),
        version: Some(r.version.to_string()),
        purl: Some(r.purl()),
        location: None,
        requirements: Vec::new(),
        owners: Vec::new(),
    })
}

/// Derive a component name from a local path (`src/scheduler/mod.rs` → `scheduler`).
fn name_from_path(path: &str) -> String {
    let p = path.strip_prefix("repo:").unwrap_or(path).trim_end_matches('/');
    let segs: Vec<&str> = p.split('/').filter(|s| !s.is_empty()).collect();
    let last = *segs.last().unwrap_or(&p);
    let stem = last.rsplit_once('.').map(|(s, _)| s).unwrap_or(last);
    if matches!(stem, "mod" | "lib" | "index" | "main") && segs.len() >= 2 {
        segs[segs.len() - 2].to_string()
    } else if last.contains('.') {
        stem.to_string()
    } else {
        last.to_string()
    }
}

pub struct SbomOptions<'a> {
    pub format: &'a str,
    pub scope: Option<&'a str>,
    pub include_tests: bool,
    pub output: Option<&'a str>,
    pub root_name: &'a str,
    pub tool_version: &'a str,
}

fn collect_components(elements: &[RawElement], resolver: &Resolver, opts: &SbomOptions) -> Vec<Component> {
    let scope_prefix = opts.scope.and_then(|q| resolver.resolve_ref(elements, q)).map(|e| e.qualified_name.clone());
    let in_scope = |e: &RawElement| -> bool {
        match &scope_prefix {
            None => true,
            Some(p) => e.qualified_name == *p || e.qualified_name.starts_with(&format!("{}::", p)),
        }
    };

    let mut comps: Vec<Component> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let req_ids = |e: &RawElement| -> Vec<String> {
        e.frontmatter
            .satisfies
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .filter_map(|r| resolver.resolve_ref(elements, r).and_then(|t| t.frontmatter.id.clone()).or_else(|| Some(r.clone())))
            .collect()
    };

    for e in elements.iter().filter(|e| is_part(e) && in_scope(e)) {
        for v in e.frontmatter.implemented_by.as_deref().unwrap_or(&[]) {
            if let Some(mut c) = parse_remote(v) {
                let key = c.purl.clone().unwrap_or_else(|| c.name.clone());
                if seen.insert(key.clone()) {
                    c.requirements = req_ids(e);
                    c.owners.push(e.qualified_name.clone());
                    comps.push(c);
                } else if let Some(existing) =
                    comps.iter_mut().find(|x| x.purl.clone().unwrap_or_else(|| x.name.clone()) == key)
                {
                    if !existing.owners.contains(&e.qualified_name) {
                        existing.owners.push(e.qualified_name.clone());
                    }
                }
            } else {
                let loc = v.strip_prefix("repo:").unwrap_or(v).to_string();
                if seen.insert(format!("file:{}", loc)) {
                    comps.push(Component {
                        name: name_from_path(v),
                        version: None,
                        purl: None,
                        location: Some(loc),
                        requirements: req_ids(e),
                        owners: vec![e.qualified_name.clone()],
                    });
                }
            }
        }
    }

    if opts.include_tests {
        for e in elements.iter().filter(|e| Resolver::is_native_testcase(e) && in_scope(e)) {
            if let Some(sf) = &e.frontmatter.source_file {
                let loc = sf.strip_prefix("repo:").unwrap_or(sf).to_string();
                if seen.insert(format!("file:{}", loc)) {
                    comps.push(Component {
                        name: name_from_path(sf),
                        version: None,
                        purl: None,
                        location: Some(loc),
                        requirements: e.frontmatter.id.iter().cloned().collect(),
                        owners: Vec::new(),
                    });
                }
            }
        }
    }
    comps
}

/// Stable CycloneDX `bom-ref` for a component: its purl, else a file/name key.
fn bom_ref(c: &Component) -> String {
    c.purl
        .clone()
        .or_else(|| c.location.as_ref().map(|l| format!("file:{}", l)))
        .unwrap_or_else(|| c.name.clone())
}

/// CycloneDX `analysis.state` for a `VulnerabilityReport.status`.
fn vex_state(status: &str) -> &'static str {
    match status {
        "mitigated" | "resolved" | "fixed" | "closed" => "resolved",
        "not_affected" => "not_affected",
        "false_positive" => "false_positive",
        "accepted" | "wont_fix" => "exploitable",
        _ => "in_triage",
    }
}

/// CycloneDX `vulnerabilities` (VEX) from the model's `VulnerabilityReport`s. A
/// report affects a component when one of its `affectedElements:` is the
/// component's purl (`pkg:...`) or the qualified name / id of a Part/PartDef
/// that declares the component via `implementedBy:`.
fn vulnerabilities(elements: &[RawElement], resolver: &Resolver, comps: &[Component]) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for vr in elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::VulnerabilityReport))
    {
        let fm = &vr.frontmatter;
        let mut refs: Vec<String> = Vec::new();
        for a in fm.affected_elements.iter().flatten() {
            if a.starts_with("pkg:") {
                for c in comps.iter().filter(|c| c.purl.as_deref() == Some(a.as_str())) {
                    refs.push(bom_ref(c));
                }
            } else if let Some(t) = resolver.resolve_ref(elements, a) {
                for c in comps.iter().filter(|c| c.owners.contains(&t.qualified_name)) {
                    refs.push(bom_ref(c));
                }
            }
        }
        refs.sort();
        refs.dedup();
        let id = fm
            .cve_id
            .clone()
            .or_else(|| fm.id.clone())
            .unwrap_or_else(|| vr.qualified_name.clone());
        let mut v = serde_json::json!({
            "bom-ref": format!("vuln:{}", fm.id.as_deref().unwrap_or(&vr.qualified_name)),
            "id": id,
            "source": { "name": if fm.cve_id.is_some() { "NVD" } else { "syscribe-model" } },
            "description": fm.name.clone().unwrap_or_default(),
        });
        if fm.cvss_score.is_some() || fm.cvss_vector.is_some() {
            let mut rating = serde_json::Map::new();
            if let Some(s) = fm.cvss_score {
                rating.insert("score".into(), serde_json::json!(s));
                let sev = fm
                    .cvss_severity
                    .clone()
                    .unwrap_or_else(|| syscribe_model::security_checks::cvss_bucket(s).to_string());
                rating.insert("severity".into(), serde_json::json!(sev));
            }
            if let Some(vec) = &fm.cvss_vector {
                rating.insert("vector".into(), serde_json::json!(vec));
                let method = if vec.starts_with("CVSS:4") {
                    "CVSSv4"
                } else if vec.starts_with("CVSS:3.0") {
                    "CVSSv3"
                } else if vec.starts_with("CVSS:3") {
                    "CVSSv31"
                } else {
                    "CVSSv2"
                };
                rating.insert("method".into(), serde_json::json!(method));
            }
            v["ratings"] = serde_json::json!([rating]);
        }
        if let Some(status) = fm.status.as_deref() {
            let mut analysis = serde_json::json!({ "state": vex_state(status) });
            if matches!(status, "accepted" | "wont_fix") {
                analysis["response"] = serde_json::json!(["will_not_fix"]);
            }
            if let Some(r) = fm.rationale.as_deref().filter(|r| !r.trim().is_empty()) {
                analysis["detail"] = serde_json::json!(r);
            }
            v["analysis"] = analysis;
        }
        if let Some(fixed) = fm.fixed_in.as_deref().filter(|f| !f.trim().is_empty()) {
            v["recommendation"] = serde_json::json!(format!("Fixed in {}", fixed));
        }
        if !refs.is_empty() {
            v["affects"] = serde_json::json!(refs.iter().map(|r| serde_json::json!({ "ref": r })).collect::<Vec<_>>());
        }
        out.push(v);
    }
    out
}

fn cyclonedx(comps: &[Component], vulns: Vec<serde_json::Value>, opts: &SbomOptions) -> serde_json::Value {
    let components: Vec<serde_json::Value> = comps
        .iter()
        .map(|c| {
            let mut o = serde_json::json!({ "type": "library", "name": c.name, "bom-ref": bom_ref(c) });
            if let Some(v) = &c.version {
                o["version"] = serde_json::json!(v);
            }
            if let Some(p) = &c.purl {
                o["purl"] = serde_json::json!(p);
            }
            if let Some(l) = &c.location {
                o["evidence"] = serde_json::json!({ "occurrences": [{ "location": l }] });
            }
            if !c.requirements.is_empty() {
                o["externalReferences"] = serde_json::json!(c
                    .requirements
                    .iter()
                    .map(|r| serde_json::json!({ "type": "model", "url": format!("syscribe://{}", r) }))
                    .collect::<Vec<_>>());
            }
            o
        })
        .collect();
    let mut doc = serde_json::json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "serialNumber": format!("urn:uuid:{}", serial_uuid()),
        "version": 1,
        "metadata": {
            "timestamp": iso8601_now(),
            "tools": [{ "vendor": "Syscribe", "name": "syscribe", "version": opts.tool_version }],
            "component": { "type": "firmware", "name": opts.root_name }
        },
        "components": components
    });
    if !vulns.is_empty() {
        doc["vulnerabilities"] = serde_json::Value::Array(vulns);
    }
    doc
}

fn spdx(comps: &[Component], opts: &SbomOptions) -> serde_json::Value {
    let root_id = "SPDXRef-Package-root";
    let mut packages: Vec<serde_json::Value> = vec![serde_json::json!({
        "SPDXID": root_id, "name": opts.root_name, "downloadLocation": "NOASSERTION", "filesAnalyzed": false
    })];
    let mut relationships: Vec<serde_json::Value> = vec![
        serde_json::json!({ "spdxElementId": "SPDXRef-DOCUMENT", "relatedSpdxElement": root_id, "relationshipType": "DESCRIBES" }),
    ];
    let mut req_pkgs: std::collections::BTreeSet<String> = Default::default();
    for (i, c) in comps.iter().enumerate() {
        let pid = format!("SPDXRef-Package-{}", i);
        let mut p = serde_json::json!({
            "SPDXID": pid, "name": c.name, "downloadLocation": "NOASSERTION",
            "filesAnalyzed": c.location.is_some()
        });
        if let Some(v) = &c.version {
            p["versionInfo"] = serde_json::json!(v);
        }
        packages.push(p);
        relationships.push(serde_json::json!({ "spdxElementId": root_id, "relatedSpdxElement": pid, "relationshipType": "CONTAINS" }));
        for r in &c.requirements {
            let rid = format!("SPDXRef-Requirement-{}", r);
            req_pkgs.insert(r.clone());
            relationships.push(serde_json::json!({ "spdxElementId": pid, "relatedSpdxElement": rid, "relationshipType": "GENERATED_FROM" }));
        }
    }
    for r in &req_pkgs {
        packages.push(serde_json::json!({
            "SPDXID": format!("SPDXRef-Requirement-{}", r), "name": r,
            "downloadLocation": "NOASSERTION", "filesAnalyzed": false
        }));
    }
    serde_json::json!({
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": opts.root_name,
        "documentNamespace": format!("https://syscribe/{}-{}", opts.root_name, serial_uuid()),
        "creationInfo": { "created": iso8601_now(), "creators": [format!("Tool: syscribe-{}", opts.tool_version)] },
        "packages": packages,
        "relationships": relationships
    })
}

pub fn cmd_sbom(elements: &[RawElement], opts: &SbomOptions) {
    // REQ-TRS-LINKTYPE-006 — a component's satisfied-requirement ids are semantic
    // output: a `coverage = true` link extending `satisfies` counts (reporting
    // view). Nothing here serializes authored frontmatter.
    let cov = syscribe_model::link_types::coverage_view(elements, &syscribe_model::link_types::active());
    let elements: &[RawElement] = &cov;
    let resolver = Resolver::new(elements);
    let comps = collect_components(elements, &resolver, opts);
    let doc = if opts.format == "spdx" { spdx(&comps, opts) } else { cyclonedx(&comps, vulnerabilities(elements, &resolver, &comps), opts) };
    let out = serde_json::to_string_pretty(&doc).unwrap();
    match opts.output {
        Some(path) => {
            if let Err(e) = std::fs::write(path, format!("{}\n", out)) {
                eprintln!("sbom: failed to write '{}': {}", path, e);
                std::process::exit(1);
            }
            eprintln!("Wrote {} ({} components) to {}", opts.format, comps.len(), path);
        }
        None => println!("{}", out),
    }
}
