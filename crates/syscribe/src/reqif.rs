//! ReqIF 1.2 export (§21, GH #73). Read-only, export-only: maps native `Requirement`
//! elements (and their packages) to a ReqIF (OMG Requirements Interchange Format) XML
//! document for import into DOORS Next / Jama / Polarion / PTC. No XML dependency — the
//! document is built by hand with proper escaping.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use syscribe_model::{element::RawElement, resolver::Resolver};

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

fn now_iso() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, rem / 3600, (rem % 3600) / 60, rem % 60)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// A stable ReqIF IDENTIFIER token (alphanumerics, `-`, `_`); other chars become `_`.
fn ident(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

/// Best-effort Markdown → XHTML for the requirement body. Gherkin blocks are dropped.
fn md_to_xhtml(body: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    let mut in_gherkin = false;
    let mut list: Option<&str> = None; // "ul" | "ol"
    let close_list = |out: &mut String, list: &mut Option<&str>| {
        if let Some(t) = list.take() {
            out.push_str(&format!("</xhtml:{}>", t));
        }
    };
    let inline = |s: &str| -> String {
        let mut t = esc(s);
        // crude inline: **bold**, `code`, *italic*
        while let (Some(a), Some(b)) = (t.find("**"), t.rfind("**")) {
            if a == b { break; }
            t = format!("{}<xhtml:strong>{}</xhtml:strong>{}", &t[..a], &t[a + 2..b], &t[b + 2..]);
        }
        while let (Some(a), Some(b)) = (t.find('`'), t.rfind('`')) {
            if a == b { break; }
            t = format!("{}<xhtml:code>{}</xhtml:code>{}", &t[..a], &t[a + 1..b], &t[b + 1..]);
        }
        t
    };
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if trimmed.contains("gherkin") {
                in_gherkin = !in_gherkin;
                continue;
            }
            close_list(&mut out, &mut list);
            if in_code {
                out.push_str("</xhtml:code></xhtml:pre>");
            } else {
                out.push_str("<xhtml:pre><xhtml:code>");
            }
            in_code = !in_code;
            continue;
        }
        if in_gherkin {
            continue;
        }
        if in_code {
            out.push_str(&esc(line));
            out.push('\n');
            continue;
        }
        if trimmed.is_empty() {
            close_list(&mut out, &mut list);
            continue;
        }
        if let Some(h) = trimmed.strip_prefix("## ") {
            close_list(&mut out, &mut list);
            out.push_str(&format!("<xhtml:h2>{}</xhtml:h2>", inline(h)));
        } else if let Some(it) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            if list != Some("ul") {
                close_list(&mut out, &mut list);
                out.push_str("<xhtml:ul>");
                list = Some("ul");
            }
            out.push_str(&format!("<xhtml:li>{}</xhtml:li>", inline(it)));
        } else if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && trimmed.contains(". ") {
            let item = trimmed.split_once(". ").map(|x| x.1).unwrap_or(trimmed);
            if list != Some("ol") {
                close_list(&mut out, &mut list);
                out.push_str("<xhtml:ol>");
                list = Some("ol");
            }
            out.push_str(&format!("<xhtml:li>{}</xhtml:li>", inline(item)));
        } else {
            close_list(&mut out, &mut list);
            out.push_str(&format!("<xhtml:p>{}</xhtml:p>", inline(trimmed)));
        }
    }
    close_list(&mut out, &mut list);
    if in_code {
        out.push_str("</xhtml:code></xhtml:pre>");
    }
    if out.is_empty() {
        out.push_str("<xhtml:p/>");
    }
    out
}

pub struct ReqifOptions<'a> {
    pub scope: Option<&'a str>,
    pub include_tests: bool,
    pub output: Option<&'a str>,
    pub zip: bool,
    pub title: &'a str,
    pub tool_version: &'a str,
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Build a `.reqifz` archive: a ZIP with a single stored (uncompressed) `content.reqif`.
fn zip_store(name: &str, data: &[u8]) -> Vec<u8> {
    let crc = crc32(data);
    let n = name.as_bytes();
    let sz = data.len() as u32;
    let mut out = Vec::new();
    // Local file header
    out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
    out.extend_from_slice(&20u16.to_le_bytes());
    for _ in 0..4 {
        out.extend_from_slice(&0u16.to_le_bytes()); // flags, method=stored, time, date
    }
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&sz.to_le_bytes());
    out.extend_from_slice(&sz.to_le_bytes());
    out.extend_from_slice(&(n.len() as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(n);
    out.extend_from_slice(data);
    // Central directory
    let cd_start = out.len();
    out.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
    out.extend_from_slice(&20u16.to_le_bytes());
    out.extend_from_slice(&20u16.to_le_bytes());
    for _ in 0..4 {
        out.extend_from_slice(&0u16.to_le_bytes());
    }
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&sz.to_le_bytes());
    out.extend_from_slice(&sz.to_le_bytes());
    out.extend_from_slice(&(n.len() as u16).to_le_bytes());
    for _ in 0..3 {
        out.extend_from_slice(&0u16.to_le_bytes()); // extra, comment, disk
    }
    out.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
    out.extend_from_slice(&0u32.to_le_bytes()); // external attrs
    out.extend_from_slice(&0u32.to_le_bytes()); // local header offset
    out.extend_from_slice(n);
    let cd_size = (out.len() - cd_start) as u32;
    // End of central directory
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&(cd_start as u32).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

/// Package-tree node for the SPEC-HIERARCHY.
#[derive(Default)]
struct Pkg {
    children: BTreeMap<String, Pkg>,
    reqs: Vec<String>, // requirement ids at this level
}

pub fn cmd_export_reqif(elements: &[RawElement], opts: &ReqifOptions) {
    let resolver = Resolver::new(elements);
    let scope_prefix = opts.scope.and_then(|q| resolver.resolve_ref(elements, q)).map(|e| e.qualified_name.clone());
    let in_scope = |e: &RawElement| match &scope_prefix {
        None => true,
        Some(p) => e.qualified_name == *p || e.qualified_name.starts_with(&format!("{}::", p)),
    };

    let reqs: Vec<&RawElement> = elements.iter().filter(|e| Resolver::is_native_requirement(e) && in_scope(e)).collect();
    let tests: Vec<&RawElement> = if opts.include_tests {
        elements.iter().filter(|e| Resolver::is_native_testcase(e) && in_scope(e)).collect()
    } else {
        Vec::new()
    };

    // Build the package tree from requirement qualified names.
    let mut root = Pkg::default();
    for r in &reqs {
        let segs: Vec<&str> = r.qualified_name.split("::").collect();
        let mut node = &mut root;
        for seg in &segs[..segs.len().saturating_sub(1)] {
            node = node.children.entry(seg.to_string()).or_default();
        }
        if let Some(id) = &r.frontmatter.id {
            node.reqs.push(id.clone());
        }
    }

    let now = now_iso();
    let mut x = String::new();
    x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    x.push_str("<REQ-IF xmlns=\"http://www.omg.org/spec/ReqIF/20110401/reqif.xsd\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n");
    // Header
    x.push_str("  <THE-HEADER>\n    <REQ-IF-HEADER IDENTIFIER=\"HDR-1\">\n");
    x.push_str(&format!("      <CREATION-TIME>{}</CREATION-TIME>\n", now));
    x.push_str(&format!("      <REQ-IF-TOOL-ID>syscribe {}</REQ-IF-TOOL-ID>\n", esc(opts.tool_version)));
    x.push_str("      <REQ-IF-VERSION>1.0</REQ-IF-VERSION>\n");
    x.push_str("      <SOURCE-TOOL-ID>syscribe</SOURCE-TOOL-ID>\n");
    x.push_str(&format!("      <TITLE>{}</TITLE>\n", esc(opts.title)));
    x.push_str("    </REQ-IF-HEADER>\n  </THE-HEADER>\n");
    x.push_str("  <CORE-CONTENT>\n    <REQ-IF-CONTENT>\n");

    // Datatypes
    x.push_str("      <DATATYPES>\n");
    x.push_str(&format!("        <DATATYPE-DEFINITION-STRING IDENTIFIER=\"DT-STRING\" LONG-NAME=\"String\" LAST-CHANGE=\"{}\" MAX-LENGTH=\"4096\"/>\n", now));
    x.push_str(&format!("        <DATATYPE-DEFINITION-INTEGER IDENTIFIER=\"DT-INT\" LONG-NAME=\"Integer\" LAST-CHANGE=\"{}\" MIN=\"0\" MAX=\"4\"/>\n", now));
    x.push_str(&format!("        <DATATYPE-DEFINITION-XHTML IDENTIFIER=\"DT-XHTML\" LONG-NAME=\"XHTML\" LAST-CHANGE=\"{}\"/>\n", now));
    let enum_dt = |id: &str, name: &str, vals: &[&str]| -> String {
        let mut s = format!("        <DATATYPE-DEFINITION-ENUMERATION IDENTIFIER=\"{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{}\">\n          <SPECIFIED-VALUES>\n", id, name, now);
        for (i, v) in vals.iter().enumerate() {
            s.push_str(&format!("            <ENUM-VALUE IDENTIFIER=\"{}-{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{}\"><PROPERTIES><EMBEDDED-VALUE KEY=\"{}\" OTHER-CONTENT=\"{}\"/></PROPERTIES></ENUM-VALUE>\n", id, v, v, now, i, v));
        }
        s.push_str("          </SPECIFIED-VALUES>\n        </DATATYPE-DEFINITION-ENUMERATION>\n");
        s
    };
    x.push_str(&enum_dt("DT-STATUS", "Status", &["draft", "review", "approved", "implemented", "verified"]));
    x.push_str(&enum_dt("DT-DOMAIN", "Domain", &["system", "hardware", "software"]));
    x.push_str("      </DATATYPES>\n");

    // Spec types
    x.push_str("      <SPEC-TYPES>\n");
    x.push_str(&format!("        <SPEC-OBJECT-TYPE IDENTIFIER=\"REQ_TYPE\" LONG-NAME=\"Requirement\" LAST-CHANGE=\"{}\">\n          <SPEC-ATTRIBUTES>\n", now));
    let str_attr = |id: &str, name: &str| format!("            <ATTRIBUTE-DEFINITION-STRING IDENTIFIER=\"{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{}\"><TYPE><DATATYPE-DEFINITION-STRING-REF>DT-STRING</DATATYPE-DEFINITION-STRING-REF></TYPE></ATTRIBUTE-DEFINITION-STRING>\n", id, name, now);
    x.push_str(&str_attr("AD-ID", "SYSCRIBE_ID"));
    x.push_str(&str_attr("AD-NAME", "NAME"));
    x.push_str(&str_attr("AD-QNAME", "QUALIFIED_NAME"));
    x.push_str(&str_attr("AD-ASIL", "ASIL_LEVEL"));
    x.push_str(&str_attr("AD-PL", "PL_LEVEL"));
    x.push_str(&str_attr("AD-VM", "VERIFICATION_METHOD"));
    x.push_str(&format!("            <ATTRIBUTE-DEFINITION-INTEGER IDENTIFIER=\"AD-SIL\" LONG-NAME=\"SIL_LEVEL\" LAST-CHANGE=\"{}\"><TYPE><DATATYPE-DEFINITION-INTEGER-REF>DT-INT</DATATYPE-DEFINITION-INTEGER-REF></TYPE></ATTRIBUTE-DEFINITION-INTEGER>\n", now));
    x.push_str(&format!("            <ATTRIBUTE-DEFINITION-XHTML IDENTIFIER=\"AD-DESC\" LONG-NAME=\"DESC\" LAST-CHANGE=\"{}\"><TYPE><DATATYPE-DEFINITION-XHTML-REF>DT-XHTML</DATATYPE-DEFINITION-XHTML-REF></TYPE></ATTRIBUTE-DEFINITION-XHTML>\n", now));
    x.push_str(&format!("            <ATTRIBUTE-DEFINITION-ENUMERATION IDENTIFIER=\"AD-STATUS\" LONG-NAME=\"STATUS\" LAST-CHANGE=\"{}\" MULTI-VALUED=\"false\"><TYPE><DATATYPE-DEFINITION-ENUMERATION-REF>DT-STATUS</DATATYPE-DEFINITION-ENUMERATION-REF></TYPE></ATTRIBUTE-DEFINITION-ENUMERATION>\n", now));
    x.push_str(&format!("            <ATTRIBUTE-DEFINITION-ENUMERATION IDENTIFIER=\"AD-DOMAIN\" LONG-NAME=\"DOMAIN\" LAST-CHANGE=\"{}\" MULTI-VALUED=\"false\"><TYPE><DATATYPE-DEFINITION-ENUMERATION-REF>DT-DOMAIN</DATATYPE-DEFINITION-ENUMERATION-REF></TYPE></ATTRIBUTE-DEFINITION-ENUMERATION>\n", now));
    x.push_str("          </SPEC-ATTRIBUTES>\n        </SPEC-OBJECT-TYPE>\n");
    // Folder + TestCase + Specification + relation types
    x.push_str(&format!("        <SPEC-OBJECT-TYPE IDENTIFIER=\"FOLDER_TYPE\" LONG-NAME=\"Package\" LAST-CHANGE=\"{}\"><SPEC-ATTRIBUTES>{}</SPEC-ATTRIBUTES></SPEC-OBJECT-TYPE>\n", now, str_attr("AD-FOLDER-NAME", "NAME").trim()));
    x.push_str(&format!("        <SPEC-OBJECT-TYPE IDENTIFIER=\"TEST_CASE\" LONG-NAME=\"TestCase\" LAST-CHANGE=\"{}\"><SPEC-ATTRIBUTES>{}</SPEC-ATTRIBUTES></SPEC-OBJECT-TYPE>\n", now, str_attr("AD-TC-NAME", "NAME").trim()));
    x.push_str(&format!("        <SPECIFICATION-TYPE IDENTIFIER=\"SPEC_TYPE\" LONG-NAME=\"Specification\" LAST-CHANGE=\"{}\"/>\n", now));
    x.push_str(&format!("        <SPEC-RELATION-TYPE IDENTIFIER=\"REL_DERIVED\" LONG-NAME=\"DERIVED_FROM\" LAST-CHANGE=\"{}\"/>\n", now));
    x.push_str(&format!("        <SPEC-RELATION-TYPE IDENTIFIER=\"REL_VERIFIED\" LONG-NAME=\"VERIFIED_BY\" LAST-CHANGE=\"{}\"/>\n", now));
    x.push_str("      </SPEC-TYPES>\n");

    // Spec objects (requirements + folders + tests)
    x.push_str("      <SPEC-OBJECTS>\n");
    let str_val = |def: &str, v: &str| {
        if v.is_empty() {
            String::new()
        } else {
            format!("            <ATTRIBUTE-VALUE-STRING THE-VALUE=\"{}\"><DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>{}</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION></ATTRIBUTE-VALUE-STRING>\n", esc(v), def)
        }
    };
    for r in &reqs {
        let fm = &r.frontmatter;
        let id = fm.id.clone().unwrap_or_else(|| r.qualified_name.clone());
        x.push_str(&format!("        <SPEC-OBJECT IDENTIFIER=\"SO-{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{}\">\n", ident(&id), esc(&id), now));
        x.push_str("          <TYPE><SPEC-OBJECT-TYPE-REF>REQ_TYPE</SPEC-OBJECT-TYPE-REF></TYPE>\n          <VALUES>\n");
        x.push_str(&str_val("AD-ID", &id));
        x.push_str(&str_val("AD-NAME", fm.name.as_deref().unwrap_or("")));
        x.push_str(&str_val("AD-QNAME", &r.qualified_name));
        x.push_str(&str_val("AD-ASIL", fm.asil_level.as_deref().unwrap_or("")));
        x.push_str(&str_val("AD-PL", fm.pl_level.as_deref().unwrap_or("")));
        x.push_str(&str_val("AD-VM", fm.verification_method.as_deref().unwrap_or("")));
        if let Some(sil) = fm.sil_level {
            x.push_str(&format!("            <ATTRIBUTE-VALUE-INTEGER THE-VALUE=\"{}\"><DEFINITION><ATTRIBUTE-DEFINITION-INTEGER-REF>AD-SIL</ATTRIBUTE-DEFINITION-INTEGER-REF></DEFINITION></ATTRIBUTE-VALUE-INTEGER>\n", sil));
        }
        if let Some(s) = &fm.status {
            x.push_str(&format!("            <ATTRIBUTE-VALUE-ENUMERATION><DEFINITION><ATTRIBUTE-DEFINITION-ENUMERATION-REF>AD-STATUS</ATTRIBUTE-DEFINITION-ENUMERATION-REF></DEFINITION><VALUES><ENUM-VALUE-REF>DT-STATUS-{}</ENUM-VALUE-REF></VALUES></ATTRIBUTE-VALUE-ENUMERATION>\n", esc(s)));
        }
        if let Some(dm) = &fm.req_domain {
            x.push_str(&format!("            <ATTRIBUTE-VALUE-ENUMERATION><DEFINITION><ATTRIBUTE-DEFINITION-ENUMERATION-REF>AD-DOMAIN</ATTRIBUTE-DEFINITION-ENUMERATION-REF></DEFINITION><VALUES><ENUM-VALUE-REF>DT-DOMAIN-{}</ENUM-VALUE-REF></VALUES></ATTRIBUTE-VALUE-ENUMERATION>\n", esc(dm)));
        }
        x.push_str(&format!("            <ATTRIBUTE-VALUE-XHTML><DEFINITION><ATTRIBUTE-DEFINITION-XHTML-REF>AD-DESC</ATTRIBUTE-DEFINITION-XHTML-REF></DEFINITION><THE-VALUE><xhtml:div>{}</xhtml:div></THE-VALUE></ATTRIBUTE-VALUE-XHTML>\n", md_to_xhtml(&r.doc)));
        x.push_str("          </VALUES>\n        </SPEC-OBJECT>\n");
    }
    // Folder objects for every package node.
    fn emit_folders(node: &Pkg, prefix: &str, now: &str, out: &mut String) {
        for (name, child) in &node.children {
            let fid = ident(&format!("{}_{}", prefix, name));
            out.push_str(&format!("        <SPEC-OBJECT IDENTIFIER=\"FO-{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{}\"><TYPE><SPEC-OBJECT-TYPE-REF>FOLDER_TYPE</SPEC-OBJECT-TYPE-REF></TYPE><VALUES><ATTRIBUTE-VALUE-STRING THE-VALUE=\"{}\"><DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>AD-FOLDER-NAME</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION></ATTRIBUTE-VALUE-STRING></VALUES></SPEC-OBJECT>\n", fid, esc(name), now, esc(name)));
            emit_folders(child, &format!("{}_{}", prefix, name), now, out);
        }
    }
    emit_folders(&root, "root", &now, &mut x);
    for tc in &tests {
        let id = tc.frontmatter.id.clone().unwrap_or_else(|| tc.qualified_name.clone());
        x.push_str(&format!("        <SPEC-OBJECT IDENTIFIER=\"TC-{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{}\"><TYPE><SPEC-OBJECT-TYPE-REF>TEST_CASE</SPEC-OBJECT-TYPE-REF></TYPE><VALUES><ATTRIBUTE-VALUE-STRING THE-VALUE=\"{}\"><DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>AD-TC-NAME</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION></ATTRIBUTE-VALUE-STRING></VALUES></SPEC-OBJECT>\n", ident(&id), esc(&id), now, esc(tc.frontmatter.name.as_deref().unwrap_or(&id))));
    }
    x.push_str("      </SPEC-OBJECTS>\n");

    // Spec relations
    x.push_str("      <SPEC-RELATIONS>\n");
    let mut rel = 0;
    for r in &reqs {
        let cid = r.frontmatter.id.clone().unwrap_or_else(|| r.qualified_name.clone());
        for df in r.frontmatter.derived_from.as_deref().unwrap_or(&[]) {
            if let Some(p) = resolver.resolve_ref(elements, df).and_then(|e| e.frontmatter.id.clone()) {
                rel += 1;
                x.push_str(&format!("        <SPEC-RELATION IDENTIFIER=\"SR-{}\" LAST-CHANGE=\"{}\"><TYPE><SPEC-RELATION-TYPE-REF>REL_DERIVED</SPEC-RELATION-TYPE-REF></TYPE><SOURCE><SPEC-OBJECT-REF>SO-{}</SPEC-OBJECT-REF></SOURCE><TARGET><SPEC-OBJECT-REF>SO-{}</SPEC-OBJECT-REF></TARGET></SPEC-RELATION>\n", rel, now, ident(&cid), ident(&p)));
            }
        }
    }
    if opts.include_tests {
        for tc in &tests {
            let tid = tc.frontmatter.id.clone().unwrap_or_else(|| tc.qualified_name.clone());
            for v in tc.frontmatter.verifies.as_deref().unwrap_or(&[]) {
                if let Some(r) = resolver.resolve_ref(elements, v).and_then(|e| e.frontmatter.id.clone()) {
                    rel += 1;
                    x.push_str(&format!("        <SPEC-RELATION IDENTIFIER=\"SR-{}\" LAST-CHANGE=\"{}\"><TYPE><SPEC-RELATION-TYPE-REF>REL_VERIFIED</SPEC-RELATION-TYPE-REF></TYPE><SOURCE><SPEC-OBJECT-REF>SO-{}</SPEC-OBJECT-REF></SOURCE><TARGET><SPEC-OBJECT-REF>TC-{}</SPEC-OBJECT-REF></TARGET></SPEC-RELATION>\n", rel, now, ident(&r), ident(&tid)));
                }
            }
        }
    }
    x.push_str("      </SPEC-RELATIONS>\n");

    // Specification with the package hierarchy.
    x.push_str("      <SPECIFICATIONS>\n");
    x.push_str(&format!("        <SPECIFICATION IDENTIFIER=\"SPEC-1\" LONG-NAME=\"{}\" LAST-CHANGE=\"{}\"><TYPE><SPECIFICATION-TYPE-REF>SPEC_TYPE</SPECIFICATION-TYPE-REF></TYPE>\n          <CHILDREN>\n", esc(opts.title), now));
    fn emit_hierarchy(node: &Pkg, prefix: &str, now: &str, counter: &mut usize, out: &mut String) {
        for id in &node.reqs {
            *counter += 1;
            out.push_str(&format!("            <SPEC-HIERARCHY IDENTIFIER=\"SH-{}\" LAST-CHANGE=\"{}\"><OBJECT><SPEC-OBJECT-REF>SO-{}</SPEC-OBJECT-REF></OBJECT></SPEC-HIERARCHY>\n", counter, now, ident(id)));
        }
        for (name, child) in &node.children {
            *counter += 1;
            let fid = ident(&format!("{}_{}", prefix, name));
            out.push_str(&format!("            <SPEC-HIERARCHY IDENTIFIER=\"SH-{}\" LAST-CHANGE=\"{}\"><OBJECT><SPEC-OBJECT-REF>FO-{}</SPEC-OBJECT-REF></OBJECT><CHILDREN>\n", counter, now, fid));
            emit_hierarchy(child, &format!("{}_{}", prefix, name), now, counter, out);
            out.push_str("            </CHILDREN></SPEC-HIERARCHY>\n");
        }
    }
    let mut counter = 0usize;
    emit_hierarchy(&root, "root", &now, &mut counter, &mut x);
    x.push_str("          </CHILDREN>\n        </SPECIFICATION>\n");
    x.push_str("      </SPECIFICATIONS>\n");

    x.push_str("    </REQ-IF-CONTENT>\n  </CORE-CONTENT>\n</REQ-IF>\n");

    match opts.output {
        Some(path) => {
            let base = path.trim_end_matches(".reqif").trim_end_matches(".reqifz");
            let (file, bytes) = if opts.zip {
                (format!("{}.reqifz", base), zip_store("content.reqif", x.as_bytes()))
            } else {
                (format!("{}.reqif", base), x.into_bytes())
            };
            if let Err(e) = std::fs::write(&file, &bytes) {
                eprintln!("export-reqif: failed to write '{}': {}", file, e);
                std::process::exit(1);
            }
            eprintln!("Wrote {} requirement(s) to {}", reqs.len(), file);
        }
        None => print!("{}", x),
    }
}
