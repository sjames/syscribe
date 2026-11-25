//! `syscribe -m <root> view render <View|ViewDef> [--format tree|table|json|mermaid|latex|markdown|csv|html]`
//! — materialise a view: the elements its `expose:` entries (and `filterCondition:`)
//! select, shown as a containment tree, a table, JSON, or a Mermaid flowchart (GH #205).
//! The default format follows the view's `rendering:` (`asTreeDiagram` → tree,
//! `asTableView`/`asElementTable` → table, `asInterconnectionDiagram` → mermaid).
//!
//! GH #531: a view that declares `columns:` is a table for a document — `table`, `json` and the
//! document formats `latex`, `markdown`, `csv`, `html` print its declared columns, rows and order
//! (`syscribe_model::view_table`).

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;
use syscribe_model::view_exposure::exposure;

use crate::query::type_label;

fn tl(e: &RawElement) -> String {
    e.frontmatter.element_type.as_ref().map(|t| type_label(t).to_string()).unwrap_or_default()
}

fn default_format(view: &RawElement) -> &'static str {
    // GH #531: a view that declares columns is a table.
    if view.frontmatter.extra.contains_key("columns") {
        return "table";
    }
    let r = view.frontmatter.rendering.as_deref().unwrap_or("").to_ascii_lowercase();
    if r.contains("table") || r.contains("grid") {
        "table"
    } else if r.contains("interconnection") || r.contains("general") || r.contains("diagram") && !r.contains("tree") {
        "mermaid"
    } else {
        "tree"
    }
}

fn mermaid_id(q: &str) -> String {
    q.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}

const FORMATS: &str = "tree, table, json, mermaid, latex, markdown, csv, html";

/// `full` is the whole model; `elems` the model `--config` projects it onto (the same without one).
pub fn cmd_view(full: &[RawElement], elems: &[RawElement], resolver: &Resolver, args: &[String]) -> i32 {
    if args.first().map(String::as_str) != Some("render") {
        eprintln!("usage: syscribe -m <root> view render <View|ViewDef> [--format tree|table|json|mermaid|latex|markdown|csv|html] [--config <C>] [--out <file>]");
        return 2;
    }
    let mut rest = args[1..].iter();
    let mut target: Option<&str> = None;
    let mut format: Option<String> = None;
    let mut config: Option<String> = None;
    let mut out_file: Option<String> = None;
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--format" => format = rest.next().cloned(),
            "--config" => config = rest.next().cloned(),
            "--out" => out_file = rest.next().cloned(),
            s if s.starts_with("--") => {
                eprintln!("view render: unknown option {s}");
                return 2;
            }
            s => target = Some(s),
        }
    }
    let Some(target) = target else {
        eprintln!("view render: name a View or ViewDef");
        return 2;
    };
    let Some(view) = resolver.resolve_ref(elems, target) else {
        eprintln!("view render: '{target}' does not resolve to an element");
        return 1;
    };
    if !matches!(view.frontmatter.element_type, Some(ElementType::View) | Some(ElementType::ViewDef)) {
        eprintln!("view render: '{target}' is a {}, not a View or ViewDef", tl(view));
        return 1;
    }
    let ex = exposure(elems, resolver, view);
    for u in &ex.unresolved {
        eprintln!("warning: expose target '{u}' does not resolve");
    }
    for n in &ex.notes {
        eprintln!("warning: {n}");
    }
    let format = format.unwrap_or_else(|| default_format(view).to_string());
    // GH #531: the declared table. A document format on a view with no `columns:` prints the
    // default columns (element, type, name).
    let declared = match syscribe_model::view_table::parse(&view.frontmatter.extra) {
        Ok(t) => t,
        Err(errs) => {
            for m in &errs {
                eprintln!("view render: malformed table view (E507): {m}");
            }
            return 1;
        }
    };
    let doc_format = syscribe_model::view_table::Format::parse(&format);
    if doc_format.is_some() || (declared.is_some() && matches!(format.as_str(), "table" | "json")) {
        use syscribe_model::view_table as vt;
        let cfg = match crate::docrender::find_config(full, config.as_deref()) {
            Ok(c) => c,
            Err(m) => {
                eprintln!("view render: {m}");
                return 1;
            }
        };
        for (is_error, m) in vt::findings(elems, view, &ex.items) {
            if !is_error {
                eprintln!("warning: {m} (W504)");
            }
        }
        let table = declared.unwrap_or_else(vt::default_table);
        let built = vt::build(&vt::Ctx { elements: elems, full, config: cfg }, view, &ex.items, &table);
        let text = match (doc_format, format.as_str()) {
            (Some(f), _) => built.write(f),
            (None, "json") => {
                let mut v = built.json();
                v["view"] = serde_json::json!(view.qualified_name);
                v["unresolved"] = serde_json::json!(ex.unresolved);
                format!("{}\n", serde_json::to_string_pretty(&v).unwrap_or_default())
            }
            // The terminal table: the Markdown form without its caption line.
            _ => vt::Built { caption: None, ..built }.write(vt::Format::Markdown),
        };
        return match out_file {
            None => {
                print!("{text}");
                0
            }
            Some(dest) => {
                let dest = std::path::Path::new(&dest);
                if let Some(parent) = dest.parent().filter(|p| !p.as_os_str().is_empty()) {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(dest, text) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("view render: cannot write {}: {e}", dest.display());
                        1
                    }
                }
            }
        };
    }
    if out_file.is_some() {
        eprintln!("view render: --out applies to the table formats (table, json, latex, markdown, csv, html)");
        return 2;
    }
    match format.as_str() {
        "tree" => {
            println!("{} ({})", view.qualified_name, tl(view));
            let base = ex.items.iter().map(|e| e.qualified_name.matches("::").count()).min().unwrap_or(0);
            for e in &ex.items {
                let depth = e.qualified_name.matches("::").count() - base;
                let leaf = e.qualified_name.rsplit("::").next().unwrap_or(&e.qualified_name);
                println!("{}{} [{}]", "  ".repeat(depth + 1), leaf, tl(e));
            }
        }
        "table" => {
            println!("| Element | Type | Name |\n|---|---|---|");
            for e in &ex.items {
                println!(
                    "| {} | {} | {} |",
                    e.qualified_name,
                    tl(e),
                    e.frontmatter.name.as_deref().unwrap_or("")
                );
            }
        }
        "json" => {
            let items: Vec<_> = ex
                .items
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "qualifiedName": e.qualified_name,
                        "type": tl(e),
                        "name": e.frontmatter.name,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "view": view.qualified_name,
                    "unresolved": ex.unresolved,
                    "elements": items,
                }))
                .unwrap_or_default()
            );
        }
        "mermaid" => {
            println!("flowchart TD");
            let names: std::collections::HashSet<&str> =
                ex.items.iter().map(|e| e.qualified_name.as_str()).collect();
            for e in &ex.items {
                let leaf = e.qualified_name.rsplit("::").next().unwrap_or(&e.qualified_name);
                println!("  {}[\"{} : {}\"]", mermaid_id(&e.qualified_name), leaf, tl(e));
            }
            for e in &ex.items {
                if let Some((parent, _)) = e.qualified_name.rsplit_once("::") {
                    if names.contains(parent) {
                        println!("  {} --> {}", mermaid_id(parent), mermaid_id(&e.qualified_name));
                    }
                }
            }
        }
        other => {
            eprintln!("view render: unknown format '{other}' ({FORMATS})");
            return 2;
        }
    }
    0
}
