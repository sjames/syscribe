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
}

/// Map a registry prefix to a PURL ecosystem.
fn purl_type(reg: &str) -> Option<&'static str> {
    match reg {
        "crates.io" => Some("cargo"),
        "npm" => Some("npm"),
        "pypi" => Some("pypi"),
        "maven" => Some("maven"),
        "nuget" => Some("nuget"),
        "github" => Some("github"),
        _ => None,
    }
}

/// Parse `<registry>:<package>@<version>[#path]` into a remote component, or `None` for a
/// local path. The local `repo:` link prefix is not a registry.
fn parse_remote(v: &str) -> Option<Component> {
    let (reg, rest) = v.split_once(':')?;
    let eco = purl_type(reg)?;
    let rest = rest.split('#').next().unwrap_or(rest);
    let (pkg, ver) = rest.rsplit_once('@')?;
    if pkg.is_empty() || ver.is_empty() {
        return None;
    }
    Some(Component {
        name: pkg.to_string(),
        version: Some(ver.to_string()),
        purl: Some(format!("pkg:{}/{}@{}", eco, pkg, ver)),
        location: None,
        requirements: Vec::new(),
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
                if seen.insert(c.purl.clone().unwrap_or_else(|| c.name.clone())) {
                    c.requirements = req_ids(e);
                    comps.push(c);
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
                    });
                }
            }
        }
    }
    comps
}

fn cyclonedx(comps: &[Component], opts: &SbomOptions) -> serde_json::Value {
    let components: Vec<serde_json::Value> = comps
        .iter()
        .map(|c| {
            let mut o = serde_json::json!({ "type": "library", "name": c.name });
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
    serde_json::json!({
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
    })
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
    let resolver = Resolver::new(elements);
    let comps = collect_components(elements, &resolver, opts);
    let doc = if opts.format == "spdx" { spdx(&comps, opts) } else { cyclonedx(&comps, opts) };
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
