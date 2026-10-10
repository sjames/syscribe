//! ReqIF import (GH #241, REQ-TRS-REQIFIMP-001): create native `Requirement` elements from the
//! SPEC-OBJECTs of a ReqIF document. Counterpart of the export in `reqif.rs`.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::json;
use std::collections::HashMap;
use syscribe_model::element::RawElement;
use syscribe_model::mutate::{plan_create_in, write_confined};

/// One requirement object read from a ReqIF document.
#[derive(Debug, Default, Clone)]
pub struct ImportedReq {
    /// OEM identifier: ForeignID / ID / SYSCRIBE_ID attribute, else the object IDENTIFIER.
    pub ext_id: String,
    pub name: String,
    pub text: String,
}

const NAME_ATTRS: &[&str] = &["name", "reqif.name", "reqif.chaptername", "title"];
const TEXT_ATTRS: &[&str] = &["desc", "reqif.text", "text", "description"];
const ID_ATTRS: &[&str] = &["reqif.foreignid", "id", "syscribe_id"];
/// Body of an object that carried no text.
const NO_TEXT: &str = "No text in the ReqIF source.";
const SKIP_TYPES: &[&str] = &["package", "testcase"];

fn attr(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
    e.attributes().flatten().find(|a| a.key.as_ref() == key).map(|a| match a.normalized_value(quick_xml::XmlVersion::Implicit1_0) {
        Ok(v) => v.into_owned(),
        Err(_) => String::from_utf8_lossy(&a.value).into_owned(),
    })
}

fn entity(name: &str) -> Option<String> {
    match name {
        "lt" => Some("<".into()),
        "gt" => Some(">".into()),
        "amp" => Some("&".into()),
        "quot" => Some("\"".into()),
        "apos" => Some("'".into()),
        n => {
            let num = n.strip_prefix('#')?;
            let cp = match num.strip_prefix(['x', 'X']) {
                Some(h) => u32::from_str_radix(h, 16).ok()?,
                None => num.parse().ok()?,
            };
            char::from_u32(cp).map(|c| c.to_string())
        }
    }
}

/// Collapse an XHTML-derived text run to paragraphs separated by one blank line.
fn tidy(s: &str) -> String {
    let paras: Vec<String> = s
        .split("\n\n")
        .map(|p| p.lines().map(|l| l.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n"))
        .filter(|p| !p.is_empty())
        .collect();
    paras.join("\n\n")
}

/// Parse the requirement objects of a ReqIF document. `Err` on malformed XML or when the
/// document holds no importable object.
pub fn parse_reqif(text: &str) -> Result<Vec<ImportedReq>, String> {
    let mut reader = Reader::from_str(text);
    let mut buf = Vec::new();
    let mut type_names: HashMap<String, String> = HashMap::new();
    let mut attr_names: HashMap<String, String> = HashMap::new();
    let mut out: Vec<ImportedReq> = Vec::new();
    let mut depth_stack: Vec<String> = Vec::new();

    // Per SPEC-OBJECT state.
    let mut in_obj = false;
    let (mut obj_id, mut obj_long, mut obj_type) = (String::new(), String::new(), String::new());
    let mut values: Vec<(String, String)> = Vec::new();
    // Per attribute value state.
    let mut in_value = false;
    let mut in_xhtml = false;
    let mut in_def = false;
    let mut cur_val = String::new();
    let mut cur_def = String::new();
    let mut ref_text_target: Option<&'static str> = None;
    // Inside THE-ORIGINAL-VALUE (the pre-edit copy of an XHTML value): not part of the text.
    let mut in_original = false;

    loop {
        let ev = reader.read_event_into(&mut buf).map_err(|e| format!("malformed XML at byte {}: {e}", reader.buffer_position()))?;
        match ev {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let is_empty = matches!(ev, Event::Empty(_));
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                match local.as_str() {
                    "SPEC-OBJECT-TYPE" => {
                        if let (Some(id), Some(n)) = (attr(e, b"IDENTIFIER"), attr(e, b"LONG-NAME")) {
                            type_names.insert(id, n);
                        }
                    }
                    "SPEC-OBJECT" => {
                        in_obj = !is_empty;
                        obj_id = attr(e, b"IDENTIFIER").unwrap_or_default();
                        obj_long = attr(e, b"LONG-NAME").unwrap_or_default();
                        obj_type.clear();
                        values.clear();
                    }
                    l if l.starts_with("ATTRIBUTE-DEFINITION-") && !l.ends_with("-REF") => {
                        if let (Some(id), Some(n)) = (attr(e, b"IDENTIFIER"), attr(e, b"LONG-NAME")) {
                            attr_names.insert(id, n);
                        }
                    }
                    l if in_obj && l.starts_with("ATTRIBUTE-VALUE-") => {
                        in_value = !is_empty;
                        in_xhtml = l == "ATTRIBUTE-VALUE-XHTML";
                        cur_val = attr(e, b"THE-VALUE").unwrap_or_default();
                        cur_def.clear();
                    }
                    "DEFINITION" if in_value => in_def = true,
                    "THE-ORIGINAL-VALUE" => in_original = !is_empty,
                    "SPEC-OBJECT-TYPE-REF" if in_obj && !in_value => ref_text_target = Some("type"),
                    l if in_def && l.starts_with("ATTRIBUTE-DEFINITION-") && l.ends_with("-REF") => ref_text_target = Some("def"),
                    _ => {}
                }
                if in_xhtml && matches!(local.as_str(), "br") {
                    cur_val.push('\n');
                }
                depth_stack.push(local);
                if is_empty {
                    // an empty element has no End event
                    let l = depth_stack.pop().unwrap_or_default();
                    if l.starts_with("ATTRIBUTE-VALUE-") && in_obj {
                        in_value = false;
                        in_xhtml = false;
                    }
                }
            }
            Event::Text(ref t) => {
                let s = t.decode().map(|c| c.into_owned()).unwrap_or_default();
                match ref_text_target {
                    Some("type") => obj_type = s.trim().to_string(),
                    Some("def") => cur_def = s.trim().to_string(),
                    _ if in_xhtml && !in_original => cur_val.push_str(&s),
                    _ => {}
                }
            }
            Event::CData(ref t) => {
                let s = t.decode().map(|c| c.into_owned()).unwrap_or_default();
                match ref_text_target {
                    Some("type") => obj_type = s.trim().to_string(),
                    Some("def") => cur_def = s.trim().to_string(),
                    _ if in_xhtml && !in_original => cur_val.push_str(&s),
                    _ => {}
                }
            }
            Event::GeneralRef(ref r) => {
                if in_xhtml && !in_original {
                    if let Some(c) = r.decode().ok().and_then(|n| entity(&n)) {
                        cur_val.push_str(&c);
                    }
                }
            }
            Event::End(ref e) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                depth_stack.pop();
                ref_text_target = None;
                if local == "DEFINITION" {
                    in_def = false;
                }
                if local == "THE-ORIGINAL-VALUE" {
                    in_original = false;
                }
                if in_xhtml && matches!(local.as_str(), "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "pre" | "ul" | "ol" | "table" | "blockquote") {
                    cur_val.push_str("\n\n");
                } else if in_xhtml && matches!(local.as_str(), "li" | "tr") {
                    cur_val.push('\n');
                } else if in_xhtml && matches!(local.as_str(), "td" | "th") {
                    cur_val.push(' ');
                }
                if local.starts_with("ATTRIBUTE-VALUE-") && in_obj {
                    let v = if in_xhtml { tidy(&cur_val) } else { cur_val.clone() };
                    values.push((cur_def.clone(), v));
                    in_value = false;
                    in_xhtml = false;
                }
                if local == "SPEC-OBJECT" && in_obj {
                    in_obj = false;
                    let tname = type_names.get(&obj_type).map(|s| s.to_ascii_lowercase()).unwrap_or_default();
                    if SKIP_TYPES.contains(&tname.as_str()) {
                        // folders and test cases are not requirements
                    } else {
                        let by = |names: &[&str]| -> Option<String> {
                            names.iter().find_map(|n| {
                                values.iter().find(|(d, v)| !v.trim().is_empty() && attr_names.get(d).is_some_and(|ln| ln.eq_ignore_ascii_case(n))).map(|(_, v)| v.clone())
                            })
                        };
                        out.push(ImportedReq {
                            ext_id: by(ID_ATTRS).unwrap_or_else(|| obj_id.clone()),
                            name: by(NAME_ATTRS)
                                .map(|n| n.split_whitespace().collect::<Vec<_>>().join(" "))
                                .or_else(|| Some(obj_long.trim().to_string()).filter(|n| !n.is_empty()))
                                .unwrap_or_else(|| obj_id.clone()),
                            // E012 forbids an empty normative text; say plainly that the source had none.
                            text: by(TEXT_ATTRS).unwrap_or_else(|| NO_TEXT.to_string()),
                        });
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    if out.is_empty() {
        return Err("no requirement objects found in the ReqIF document".into());
    }
    Ok(out)
}

pub struct ImportOptions<'a> {
    pub file: &'a str,
    pub into: &'a str,
    pub id_prefix: &'a str,
    pub req_class: &'a str,
    pub req_domain: &'a str,
    pub update: bool,
    pub dry_run: bool,
}

fn ext_ref_of(ext_id: &str) -> String {
    format!("reqif:{ext_id}")
}

/// Rewrite `name:` (and its wrapped continuation lines) and, when `body` is given, the body of an
/// existing requirement file; every other byte is kept. `None` for CRLF or frontmatter-less files.
fn rewrite_name_and_body(content: &str, name: &str, body: Option<&str>) -> Option<String> {
    if content.contains('\r') {
        return None; // CRLF files are not rewritten (byte preservation); the caller says so
    }
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let (yaml, after) = rest.split_at(end);
    let after = &after[4..]; // past "\n---"
    let name_line = format!("name: {}", serde_yaml::to_string(&serde_yaml::Value::String(name.to_string())).ok()?.trim_end());
    let mut replaced = false;
    let mut in_name = false;
    let mut lines: Vec<String> = Vec::new();
    for l in yaml.lines() {
        if !replaced && l.starts_with("name:") {
            lines.push(name_line.clone());
            replaced = true;
            in_name = true;
        } else if in_name && l.starts_with([' ', '\t']) {
            // continuation of a folded/wrapped `name:` value — replaced along with it
        } else {
            in_name = false;
            lines.push(l.to_string());
        }
    }
    if !replaced {
        lines.push(name_line);
    }
    Some(match body {
        Some(b) => format!("---\n{}\n---\n\n{}\n", lines.join("\n"), b.trim_end()),
        None => format!("---\n{}\n---{}", lines.join("\n"), after),
    })
}

/// A body that has been worked on after import: headings or code fences (rationale, acceptance
/// criteria, Gherkin). `--update` keeps such a body and refreshes only the name.
fn body_is_enriched(doc: &str) -> bool {
    doc.lines().any(|l| l.starts_with('#') || l.trim_start().starts_with("```"))
}

const REQ_CLASSES: &[&str] = &["stakeholder", "system", "software", "hardware", "process", "regulatory", "deliverable"];
const REQ_DOMAINS: &[&str] = &["system", "hardware", "software"];

/// Entry point. Returns the process exit code.
pub fn cmd_import_reqif(model_root: &std::path::Path, elems: &[RawElement], opts: &ImportOptions) -> i32 {
    let text = match std::fs::read_to_string(opts.file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("import-reqif: cannot read {}: {e}", opts.file);
            return 1;
        }
    };
    let reqs = match parse_reqif(&text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("import-reqif: {e}");
            return 1;
        }
    };
    if !REQ_CLASSES.contains(&opts.req_class) {
        eprintln!("import-reqif: --class '{}' is not one of {}", opts.req_class, REQ_CLASSES.join(", "));
        return 1;
    }
    if !REQ_DOMAINS.contains(&opts.req_domain) {
        eprintln!("import-reqif: --domain '{}' is not one of {}", opts.req_domain, REQ_DOMAINS.join(", "));
        return 1;
    }
    // Existing native requirements by OEM identifier (`extRef: reqif:<id>`), and by their own id so
    // that a Syscribe export (SYSCRIBE_ID) imports back onto the model it came from.
    let mut existing: HashMap<String, &RawElement> = HashMap::new();
    for e in elems.iter().filter(|e| syscribe_model::resolver::Resolver::is_native_requirement(e)) {
        for r in e.frontmatter.ext_ref.iter().flatten() {
            existing.insert(r.clone(), e);
        }
        if let Some(id) = &e.frontmatter.id {
            existing.entry(ext_ref_of(id)).or_insert(e);
        }
    }
    // Next free number for the prefix.
    let pfx = format!("{}-", opts.id_prefix);
    let mut next = elems
        .iter()
        .filter_map(|e| e.frontmatter.id.as_deref())
        .filter_map(|i| i.strip_prefix(&pfx))
        .filter_map(|n| n.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let (mut created, mut updated, mut skipped, mut failed, mut dups) = (0, 0, 0, 0, 0);
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in &reqs {
        let xr = ext_ref_of(&r.ext_id);
        if !seen.insert(xr.clone()) {
            eprintln!("import-reqif: duplicate identifier '{}' in the file — later object skipped", r.ext_id);
            dups += 1;
            continue;
        }
        if let Some(e) = existing.get(&xr) {
            let id = e.frontmatter.id.clone().unwrap_or_else(|| e.qualified_name.clone());
            let enriched = body_is_enriched(&e.doc);
            let name_changed = e.frontmatter.name.as_deref() != Some(r.name.as_str());
            let body_changed = !enriched && e.doc.trim() != r.text.trim();
            let changed = name_changed || body_changed;
            if !(opts.update && changed) {
                println!("exists {id} ({}){}", r.ext_id, if changed { " — text differs, use --update" } else { "" });
                skipped += 1;
                continue;
            }
            let stem_ok = std::path::Path::new(&e.file_path).file_stem().and_then(|s| s.to_str()) == e.frontmatter.id.as_deref();
            if !stem_ok {
                eprintln!("import-reqif: cannot update {id}: it is not stored in its own <id>.md file ({})", e.file_path);
                failed += 1;
                continue;
            }
            if opts.dry_run {
                println!("would update {id} ({}){}", r.ext_id, if enriched { " — name only, body kept" } else { "" });
                updated += 1;
                continue;
            }
            let Ok(old) = std::fs::read_to_string(&e.file_path) else {
                eprintln!("import-reqif: cannot read {}", e.file_path);
                failed += 1;
                continue;
            };
            let body = (!enriched).then_some(r.text.as_str());
            let Some(new) = rewrite_name_and_body(&old, &r.name, body) else {
                eprintln!("import-reqif: cannot update {id}: {} has CRLF line endings or no frontmatter", e.file_path);
                failed += 1;
                continue;
            };
            if let Err(err) = std::fs::write(&e.file_path, new) {
                eprintln!("import-reqif: cannot write {}: {err}", e.file_path);
                failed += 1;
                continue;
            }
            println!("updated {id} ({}){}", r.ext_id, if enriched { " — name only, body kept (it has sections of its own)" } else { "" });
            updated += 1;
            continue;
        }
        let id = format!("{pfx}{next:03}");
        next += 1;
        let fields = json!({
            "id": id, "name": r.name, "status": "draft",
            "reqClass": opts.req_class, "reqDomain": opts.req_domain,
            "extRef": [xr],
        });
        let plan = match plan_create_in(elems, Some(opts.into), None, "Requirement", Some(&fields), Some(r.text.as_str())) {
            Ok(p) => p,
            Err(e) => {
                let hint = if matches!(e, syscribe_model::mutate::CreateError::InvalidId(_)) {
                    " (a custom --id-prefix other than REQ-… must be listed under [ids.prefixes] Requirement in .syscribe.toml)"
                } else {
                    ""
                };
                eprintln!("import-reqif: cannot create {id} for {}: {e}{hint}", r.ext_id);
                failed += 1;
                // The same cause (prefix, target package) fails every object: stop rather than spam.
                break;
            }
        };
        if opts.dry_run {
            println!("would create {id} ({}) {}", r.ext_id, r.name);
            created += 1;
            continue;
        }
        if let Err(e) = write_confined(model_root, &plan.rel, &plan.content) {
            eprintln!("import-reqif: cannot write {}: {e}", plan.rel);
            failed += 1;
            break;
        }
        println!("created {id} ({}) {}", r.ext_id, r.name);
        created += 1;
    }
    let (c, u) = if opts.dry_run { ("Would create", "would update") } else { ("Created", "updated") };
    println!("{c} {created}, {u} {updated}, existing {skipped}, duplicate {dups}, failed {failed} ({} object(s) in {}).", reqs.len(), opts.file);
    i32::from(failed > 0)
}
