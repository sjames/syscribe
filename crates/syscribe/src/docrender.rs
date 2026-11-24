//! `render <file>` for an external document (GH #530, `REQ-TRS-DOCREF-001`): replace each value
//! reference with the current value and unit, written for the document's language. The model part
//! lives in `syscribe_model::doc_render`; this is the command.

use std::path::Path;

use syscribe_model::doc_render::{self as dr, Delims, DocumentsConfig, Escape, Scope};
use syscribe_model::element::RawElement;

pub const USAGE: &str = "Usage: syscribe -m <root> render <file> [--out <file>] [--escape latex|typst|html|markdown|none] [--open <s> --close <s>] [--config <C>]\n       syscribe -m <root> render <diagram-path>";

/// The options that select the document form of `render`.
const DOC_FLAGS: &[&str] = &["--out", "--escape", "--open", "--close", "--config"];

/// Whether `render <key> <args>` is the document form: one of its options is given, or `key` is a
/// file that is not an element of the model.
pub fn is_document_call(elements: &[RawElement], key: &str, args: &[String]) -> bool {
    if args.iter().any(|a| DOC_FLAGS.contains(&a.as_str())) {
        return true;
    }
    let is_element = elements.iter().any(|e| e.file_path == key || e.file_path.ends_with(&format!("/{key}")));
    !is_element && Path::new(key).is_file()
}

/// Print `[documents]` problems once, as warnings.
pub fn warn_config(cfg: &DocumentsConfig) {
    for p in &cfg.problems {
        eprintln!("warning: .syscribe.toml: {p}");
    }
}

/// The `Configuration` element `conf` names.
pub fn find_config<'a>(full: &'a [RawElement], conf: Option<&str>) -> Result<Option<&'a RawElement>, String> {
    match conf {
        None => Ok(None),
        Some(c) => crate::residual::find_configuration(full, c).map(|i| Some(&full[i])).ok_or_else(|| format!("Configuration not found: {c}")),
    }
}

/// `args` are the tokens after `render`. `view` is the model projected onto `--config` (the whole
/// model without one). Returns the exit code: 0 rendered, 1 an unresolvable reference or a usage error.
pub fn cmd_render_document(full: &[RawElement], view: &[RawElement], model_root: &Path, args: &[String]) -> i32 {
    let value = |flag: &str| args.windows(2).find(|w| w[0] == flag).map(|w| w[1].as_str());
    let mut file: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if DOC_FLAGS.contains(&a) {
            if i + 1 >= args.len() {
                eprintln!("render: {a} needs a value\n{USAGE}");
                return 1;
            }
            i += 2;
        } else if a.starts_with("--") {
            eprintln!("render: unknown option {a}\n{USAGE}");
            return 1;
        } else if file.is_none() {
            file = Some(a);
            i += 1;
        } else {
            eprintln!("render: one file at a time (got `{}` and `{a}`)\n{USAGE}", file.unwrap_or_default());
            return 1;
        }
    }
    let Some(file) = file else {
        eprintln!("{USAGE}");
        return 1;
    };
    let path = Path::new(file);
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("render: cannot read {file}: {e}");
            return 1;
        }
    };
    let cfg = DocumentsConfig::load(model_root);
    warn_config(&cfg);
    let delims = match (value("--open"), value("--close")) {
        (None, None) => cfg.delims_for(path),
        (Some(o), Some(c)) => match Delims::new(o, c) {
            Ok(d) => d,
            Err(m) => {
                eprintln!("render: {m}");
                return 1;
            }
        },
        _ => {
            eprintln!("render: --open and --close are given together\n{USAGE}");
            return 1;
        }
    };
    let escape = match value("--escape") {
        None => Escape::for_path(path),
        Some(s) => match Escape::parse(s) {
            Some(e) => e,
            None => {
                eprintln!("render: unknown --escape `{s}`; one of: {}", Escape::NAMES.join(", "));
                return 1;
            }
        },
    };
    let config = match find_config(full, value("--config")) {
        Ok(c) => c,
        Err(m) => {
            eprintln!("render: {m}");
            return 1;
        }
    };
    let scope = Scope { view, full, config };
    match dr::render(&scope, &text, &delims, escape, dr::is_markdown(path)) {
        Err(bad) => {
            for u in &bad {
                eprintln!("{file}:{}: E331: unresolvable value reference `{}`: {}", u.line, u.raw, u.why);
            }
            eprintln!("render: {} unresolvable reference(s) in {file}; nothing written", bad.len());
            1
        }
        Ok(out) => match value("--out") {
            None => {
                print!("{out}");
                0
            }
            Some(dest) => {
                let dest = Path::new(dest);
                if let Some(parent) = dest.parent().filter(|p| !p.as_os_str().is_empty()) {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("render: cannot create {}: {e}", parent.display());
                        return 1;
                    }
                }
                if let Err(e) = std::fs::write(dest, out) {
                    eprintln!("render: cannot write {}: {e}", dest.display());
                    return 1;
                }
                0
            }
        },
    }
}

/// One use of an attribute or parameter in an external document.
pub struct DocUse {
    pub file: String,
    /// 1-based.
    pub line: usize,
    pub raw: String,
}

/// The references to `target`.`attr` (as [`dr::Quantity`] names them) in the documents of
/// `[documents] paths` (`REQ-TRS-DOCREF-003`).
pub fn uses_in_documents(elements: &[RawElement], model_root: &Path, target: &str, attr: &str) -> Vec<DocUse> {
    let cfg = DocumentsConfig::load(model_root);
    warn_config(&cfg);
    let scope = Scope { view: elements, full: elements, config: None };
    let mut out = Vec::new();
    for f in cfg.files() {
        let Ok(text) = std::fs::read_to_string(&f) else { continue };
        for (r, q) in dr::resolve_all(&scope, &text, &cfg.delims_for(&f), dr::is_markdown(&f)) {
            // A parameter with no value outside a Configuration is still a use of it.
            let hit = match &q {
                Ok(q) => q.target == target && q.attr == attr,
                Err(_) => false,
            };
            if hit {
                out.push(DocUse { file: f.display().to_string(), line: r.line + 1, raw: r.raw });
            }
        }
    }
    out
}

/// `refs <Feature>.<param>`: the uses of a feature parameter in the documents of `[documents] paths`,
/// printed after the element listing (a parameter has no `{{…}}` uses of the value-reference kind).
pub fn print_parameter_uses(elements: &[RawElement], model_root: &Path, key: &str) {
    use syscribe_model::value_refs as vr;
    let Some((elem, param)) = key.rsplit_once('.') else { return };
    let resolver = syscribe_model::resolver::Resolver::new(elements);
    let vr::ElemMatch::Found(fd) = vr::resolve_element(elements, &resolver, elem) else { return };
    if fd.frontmatter.element_type != Some(syscribe_model::element::ElementType::FeatureDef) {
        return;
    }
    let cfg = DocumentsConfig::load(model_root);
    let target = vr::label(fd);
    let mut rows = Vec::new();
    for f in cfg.files() {
        let Ok(text) = std::fs::read_to_string(&f) else { continue };
        for r in dr::scan(&text, &cfg.delims_for(&f), dr::is_markdown(&f)) {
            let same = matches!(vr::resolve_element(elements, &resolver, &r.elem), vr::ElemMatch::Found(t) if std::ptr::eq(t, fd));
            if same && r.path == param {
                rows.push((f.display().to_string(), r.line + 1, r.raw));
            }
        }
    }
    if rows.is_empty() {
        return;
    }
    println!();
    println!("## Uses in documents ({target}.{param})");
    println!();
    println!("| File | Line | Reference |");
    println!("|---|---|---|");
    for (file, line, raw) in rows {
        println!("| {file} | {line} | `{raw}` |");
    }
}
