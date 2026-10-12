//! `syscribe -m <root> diagram export <qname> --format plantuml|mermaid|svg
//! [--out <file>]` (`REQ-TRS-VIS-009`, `REQ-TRS-VIS-010`).
//!
//! The one `diagram` subcommand left after `ADR-SYS-VIS-001` retired the CLI
//! toolkit: it writes one `Diagram` element's PlantUML, Mermaid or static
//! SVG — each a pure function of the Diagram IR (`syscribe_model::vis`) — to
//! stdout or to `--out`. SVG works on any diagram with an IR: pins are
//! honoured, and an unpinned diagram is laid out by the embedded ELK
//! (`REQ-TRS-VIS-016`). Mermaid and SVG take their `[links]` hyperlinks from
//! `ValidateConfig::hosted_url_for` (inert when `[links]` is absent); PlantUML
//! keeps its own `[plantuml] base_url` links.
//!
//! `diagram export --all [--check] [--out-dir <dir>]` (GH #339) renders every
//! diagram that has an IR to `<dir>/<qualified name>.<ext>` (default
//! `<model>/generated`, or `<model>/generated/<CONF-id>` with `--config`; SVG)
//! and records what it wrote in `<dir>/.syscribe-generated`; only a file named
//! there is ever removed. With `--check` it writes nothing and exits 1 when a
//! committed file differs from a fresh render, is missing, or is left over, or
//! when a diagram fails to render. Text is sized with the bundled metrics
//! (`vis::metrics::diagram_metrics`) and a PlantUML style `!include` is written
//! relative to the file, so a render is the same file on every machine.

use std::path::{Path, PathBuf};

use syscribe_model::config::{load_plantuml_config, ValidateConfig};
use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::resolver::Resolver;
use syscribe_model::vis;

const FORMATS: &[&str] = &["plantuml", "mermaid", "svg", "dot"];
const USAGE: &str = "Usage: syscribe -m <model> diagram export <qname> [--format plantuml|mermaid|svg|dot] [--config <C> [--config <D>]] [--out <file>]\n       syscribe -m <model> diagram export --all [--format svg|mermaid|dot|plantuml] [--config <C>] [--out-dir <dir>] [--check]";

/// Entry point for `syscribe -m <root> diagram <sub> …`. Returns the exit code.
pub fn cmd_diagram(
    elements: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    config: &ValidateConfig,
    args: &[String],
) -> i32 {
    match args.first().map(String::as_str) {
        Some("export") => cmd_export(elements, resolver, model_root, config, &args[1..]),
        Some(other) => {
            eprintln!("error: unrecognized subcommand '{other}' — the diagram command has one subcommand: export");
            eprintln!("{USAGE}");
            1
        }
        None => {
            eprintln!("{USAGE}");
            1
        }
    }
}

/// The hosted URL of the element a diagram reference names, per the model's
/// `[links]` table — `None` when the reference resolves to no element or no
/// `[links]` is configured.
pub fn link_resolver<'a>(
    elements: &'a [RawElement],
    resolver: &'a Resolver,
    config: &'a ValidateConfig,
) -> impl Fn(&str) -> Option<String> + 'a {
    move |qname: &str| {
        let target = resolver.resolve_ref(elements, qname)?;
        config.hosted_url_for(&target.file_path, &target.qualified_name, target.frontmatter.id.as_deref().unwrap_or(""))
    }
}

/// Render one `Diagram` element in `format`. `Err` carries the message the
/// caller prints (CLI) or returns (MCP), without an `error: ` prefix.
pub fn export_diagram(
    elem: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    config: &ValidateConfig,
    format: &str,
) -> Result<String, String> {
    export_classified(elem, elements, resolver, model_root, config, format).map_err(|e| e.msg)
}

/// Why a diagram was not rendered. `expected` when the diagram has, on purpose, nothing this format draws: a
/// hand-authored body, a kind with no mapping in the format, or a companion picture with no IR. Anything else (a
/// layout failure, a derived diagram that came out empty) is a failure `diagram export --all` must not skip.
struct NotRendered {
    msg: String,
    expected: bool,
}

impl NotRendered {
    fn expected(msg: String) -> Self {
        NotRendered { msg, expected: true }
    }
    fn failed(msg: String) -> Self {
        NotRendered { msg, expected: false }
    }
}

fn export_classified(
    elem: &RawElement,
    elements: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    config: &ValidateConfig,
    format: &str,
) -> Result<String, NotRendered> {
    let qname = &elem.qualified_name;
    if elem.frontmatter.element_type != Some(ElementType::Diagram) {
        return Err(NotRendered::failed(format!("'{qname}' is not a Diagram")));
    }
    match format {
        "plantuml" => {
            let cfg = load_plantuml_config(model_root);
            syscribe_model::plantuml::render_plantuml_in(elem, elements, Some(&cfg), model_root)
                .ok_or_else(|| NotRendered::expected(format!("'{qname}' has a diagramKind with no PlantUML mapping — export mermaid instead")))
        }
        "mermaid" | "svg" | "dot" => {
            let Some((graph, _issues)) = vis::build_graph_in(elem, elements, resolver, model_root) else {
                return Err(NotRendered::expected(hand_authored(elem)));
            };
            render_graph_as(elem, graph, elements, resolver, model_root, config, format)
        }
        other => Err(NotRendered::failed(format!("invalid value '{other}' for --format; valid values: {}", FORMATS.join(", ")))),
    }
}

fn hand_authored(elem: &RawElement) -> String {
    format!(
        "'{}' is a hand-authored {} diagram — its body is the source; use `render`",
        elem.qualified_name,
        elem.frontmatter.diagram_kind.as_deref().unwrap_or("Custom")
    )
}

/// A diagram whose picture is a companion SVG file (`svgFile:` / `svgMode: companion`).
fn is_companion(elem: &RawElement) -> bool {
    elem.frontmatter.svg_file.is_some() || elem.frontmatter.svg_mode.as_deref() == Some("companion")
}

/// Render an already-built graph of `elem` in `format`.
fn render_graph_as(
    elem: &RawElement,
    graph: vis::DiagramGraph,
    elements: &[RawElement],
    resolver: &Resolver,
    model_root: &Path,
    config: &ValidateConfig,
    format: &str,
) -> Result<String, NotRendered> {
    let qname = &elem.qualified_name;
    let links = link_resolver(elements, resolver, config);
    match format {
        "plantuml" => syscribe_model::plantuml::render_plantuml_graph(elem, graph, Some(&load_plantuml_config(model_root)))
            .ok_or_else(|| NotRendered::expected(format!("'{qname}' has a diagramKind with no PlantUML mapping — export mermaid instead"))),
        "mermaid" => vis::render_mermaid(&graph, &links).ok_or_else(|| NotRendered::expected(format!("'{qname}' has no Mermaid mapping"))),
        "dot" => Ok(vis::render_dot(&graph)),
        // REQ-TRS-VIS-016: drawn from pins when fully pinned, else laid out by the embedded ELK first.
        _ => vis::render_svg(&graph, &links).map_err(|e| match e {
            // a companion picture has no shapes on purpose: its SVG file is the picture
            vis::SvgError::Empty if is_companion(elem) => NotRendered::expected(format!("'{qname}' is a companion picture (`svgFile:`), not rendered")),
            vis::SvgError::Empty => NotRendered::failed(format!("'{qname}' has no shapes to draw")),
            other => NotRendered::failed(format!("'{qname}': {other}")),
        }),
    }
}

/// The variant view (GH #338): `elem` drawn for two Configurations at once, what only one has marked.
fn export_variant(elements: &[RawElement], model_root: &Path, config: &ValidateConfig, qname: &str, configs: [&str; 2], format: &str) -> Result<String, String> {
    let mut graphs = Vec::new();
    let mut labels = Vec::new();
    for c in configs {
        let sel = match syscribe_model::projection::resolve_config_flag(elements, c) {
            syscribe_model::projection::SelectionOutcome::Resolved(sel) => sel,
            syscribe_model::projection::SelectionOutcome::Dormant => return Err(format!("the model has no product line, so '{c}' selects nothing")),
            syscribe_model::projection::SelectionOutcome::Error(m) => return Err(m),
        };
        let mut view = syscribe_model::projection::project(elements, &sel);
        syscribe_model::vis::settle_variant(&mut view);
        let resolver = Resolver::new(&view);
        let Some(elem) = resolver.resolve_ref(&view, qname) else {
            return Err(format!("'{qname}' is not active in configuration '{c}'"));
        };
        if elem.frontmatter.element_type != Some(ElementType::Diagram) {
            return Err(format!("'{qname}' is not a Diagram"));
        }
        let (graph, _issues) = vis::build_graph_in(elem, &view, &resolver, model_root).ok_or_else(|| hand_authored(elem))?;
        graphs.push((graph, elem.clone()));
        // the Configuration's id names it on the diagram
        labels.push(resolver.resolve_ref(&view, c).and_then(|e| e.frontmatter.id.clone()).unwrap_or_else(|| c.to_string()));
    }
    let (b, _) = graphs.pop().unwrap();
    let (a, elem) = graphs.pop().unwrap();
    let merged = syscribe_model::vis::variant::diff(a, b, &labels[0], &labels[1]);
    let resolver = Resolver::new(elements);
    render_graph_as(&elem, merged, elements, &resolver, model_root, config, format).map_err(|e| e.msg)
}

fn cmd_export(elements: &[RawElement], resolver: &Resolver, model_root: &Path, config: &ValidateConfig, args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--all") {
        return cmd_export_all(elements, model_root, config, args);
    }
    // a gate that silently exports one diagram would always pass
    if let Some(flag) = args.iter().find(|a| *a == "--check" || *a == "--out-dir") {
        eprintln!("error: {flag} applies only with --all\n{USAGE}");
        return 1;
    }
    let format = crate::cliargs::or_exit(crate::cliargs::enum_value("diagram export", args, "--format", FORMATS, "plantuml"));
    let out_file = args.windows(2).find(|w| w[0] == "--out").map(|w| w[1].as_str());
    let configs: Vec<&str> = args.windows(2).filter(|w| w[0] == "--config").map(|w| w[1].as_str()).collect();
    let config_flag = configs.first().copied();
    let qname = {
        let mut found = None;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--format" || args[i] == "--out" || args[i] == "--config" {
                i += 2;
                continue;
            }
            if args[i].starts_with('-') {
                i += 1;
                continue;
            }
            found = Some(args[i].as_str());
            break;
        }
        found
    };
    let Some(qname) = qname else {
        eprintln!("{USAGE}");
        return 1;
    };
    if configs.len() > 2 {
        eprintln!("error: --config is given at most twice (two products for a variant view)");
        return 1;
    }
    if configs.len() == 2 {
        return emit(export_variant(elements, model_root, config, qname, [configs[0], configs[1]], format), out_file);
    }
    // `--config`: derive from the model projected onto that Configuration (REQ-TRS-BDDCFG-001).
    let projected: Option<Vec<RawElement>> = match config_flag {
        None => None,
        Some(c) => match syscribe_model::projection::resolve_config_flag(elements, c) {
            syscribe_model::projection::SelectionOutcome::Dormant => None,
            syscribe_model::projection::SelectionOutcome::Resolved(sel) => {
                let mut view = syscribe_model::projection::project(elements, &sel);
                // in one product every part drawn is present: no `optional` badge (GH #336)
                syscribe_model::vis::settle_variant(&mut view);
                Some(view)
            }
            syscribe_model::projection::SelectionOutcome::Error(m) => {
                eprintln!("{m}");
                return 1;
            }
        },
    };
    let (full_elements, full_resolver) = (elements, resolver);
    let view_resolver;
    let (elements, resolver): (&[RawElement], &Resolver) = match &projected {
        Some(v) => {
            view_resolver = Resolver::new(v);
            (v.as_slice(), &view_resolver)
        }
        None => (elements, resolver),
    };
    let Some(elem) = resolver.resolve_ref(elements, qname) else {
        match config_flag {
            Some(c) if full_resolver.resolve_ref(full_elements, qname).is_some() => {
                eprintln!("error: '{qname}' exists but is not active in configuration '{c}'")
            }
            Some(_) => eprintln!("error: element '{qname}' not found"),
            None => eprintln!("error: element '{qname}' not found"),
        }
        return 1;
    };
    emit(export_diagram(elem, elements, resolver, model_root, config, format), out_file)
}

/// The list of files `diagram export --all` wrote, kept in the output directory: only a file named in it is ever
/// reported left over or removed, so a hand-drawn picture that happens to sit there is never touched.
const MANIFEST: &str = ".syscribe-generated";

/// `diagram export --all` (GH #339): every diagram with an IR, sorted by qualified name, to `--out-dir`.
fn cmd_export_all(elements: &[RawElement], model_root: &Path, config: &ValidateConfig, args: &[String]) -> i32 {
    // SVG unless asked otherwise: the format a committed, reviewed render is kept in
    let format = crate::cliargs::or_exit(crate::cliargs::enum_value("diagram export", args, "--format", FORMATS, "svg"));
    let check = args.iter().any(|a| a == "--check");
    let configs: Vec<&str> = args.windows(2).filter(|w| w[0] == "--config").map(|w| w[1].as_str()).collect();
    if configs.len() > 1 {
        eprintln!("error: `--all` takes one --config (a variant view is one diagram's export)");
        return 1;
    }
    // `--all` renders every diagram: a qname or a single output file means the one-diagram form was meant
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" | "--config" | "--out-dir" => i += 2,
            "--out" => {
                eprintln!("error: --out names one file; with --all use --out-dir\n{USAGE}");
                return 1;
            }
            a if !a.starts_with('-') => {
                eprintln!("error: --all renders every diagram; drop '{a}' or drop --all\n{USAGE}");
                return 1;
            }
            _ => i += 1,
        }
    }
    let projected: Option<Vec<RawElement>> = match configs.first() {
        None => None,
        Some(c) => match syscribe_model::projection::resolve_config_flag(elements, c) {
            syscribe_model::projection::SelectionOutcome::Dormant => None,
            syscribe_model::projection::SelectionOutcome::Resolved(sel) => {
                let mut view = syscribe_model::projection::project(elements, &sel);
                syscribe_model::vis::settle_variant(&mut view);
                Some(view)
            }
            syscribe_model::projection::SelectionOutcome::Error(m) => {
                eprintln!("{m}");
                return 1;
            }
        },
    };
    // one product's renders go beside the whole model's, never over them
    let default_dir = match configs.first() {
        None => model_root.join("generated"),
        Some(c) => {
            let id = Resolver::new(elements).resolve_ref(elements, c).and_then(|e| e.frontmatter.id.clone()).unwrap_or_else(|| c.to_string());
            model_root.join("generated").join(id)
        }
    };
    let out_dir = args.windows(2).find(|w| w[0] == "--out-dir").map(|w| PathBuf::from(&w[1])).unwrap_or(default_dir);
    // the directory is the export's own; one that holds the model would put the model's own files in its reach
    let (abs_out, abs_root) = (normalize(&out_dir), normalize(model_root));
    if abs_root.starts_with(&abs_out) {
        eprintln!("error: --out-dir {} contains the model; give the export a directory of its own", out_dir.display());
        return 1;
    }
    let elements = projected.as_deref().unwrap_or(elements);
    let resolver = Resolver::new(elements);
    let ext = match format {
        "plantuml" => "puml",
        "mermaid" => "mmd",
        other => other,
    };
    // a PlantUML export names its style file relative to itself, so the text does not depend on where the checkout is
    let style = if format == "plantuml" { load_plantuml_config(model_root).style_file.map(|f| (f.display().to_string(), normalize(&f))) } else { None };
    let mut diagrams: Vec<&RawElement> = elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Diagram)).collect();
    diagrams.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
    let mut fresh: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut failed = 0;
    // a diagram that failed keeps its file: it is still rendered by a diagram, just not today
    let mut kept_rels: Vec<String> = Vec::new();
    for d in diagrams {
        let rel = format!("{}.{ext}", d.qualified_name.split("::").collect::<Vec<_>>().join("/"));
        match export_classified(d, elements, &resolver, model_root, config, format) {
            Ok(mut text) => {
                if let Some((abs, target)) = &style {
                    let dir = abs_out.join(&rel);
                    let rel_style = relative_to(target, dir.parent().unwrap_or(&abs_out));
                    text = text.replacen(&format!("!include {abs}\n"), &format!("!include {}\n", rel_style.display()), 1);
                }
                fresh.insert(rel, text);
            }
            Err(e) if e.expected => eprintln!("skipped: {}", e.msg),
            Err(e) => {
                eprintln!("error: {}", e.msg);
                failed += 1;
                kept_rels.push(rel);
            }
        }
    }
    let old = read_manifest(&out_dir);
    // pictures a diagram's `svgFile:` names are never the export's to remove, whatever the manifest says
    let protected: Vec<PathBuf> = elements
        .iter()
        .filter_map(|e| {
            let sf = e.frontmatter.svg_file.as_deref()?;
            Some(normalize(&Path::new(&e.file_path).parent()?.join(sf.trim())))
        })
        .collect();
    let stale: Vec<&String> = old
        .iter()
        .filter(|r| r.ends_with(&format!(".{ext}")) && !fresh.contains_key(*r) && !kept_rels.contains(r) && safe_rel(r))
        .filter(|r| !protected.contains(&normalize(&out_dir.join(r))))
        .collect();
    let mut manifest: std::collections::BTreeSet<String> = old.iter().filter(|r| !stale.contains(r)).cloned().collect();
    manifest.extend(fresh.keys().cloned());
    let new_manifest = manifest_text(&manifest);
    let regenerate = {
        let rest: Vec<&str> = args.iter().map(String::as_str).filter(|a| *a != "--check").collect();
        format!("`syscribe -m {} diagram export {}`", model_root.display(), rest.join(" "))
    };
    if check {
        let mut problems = failed;
        for (rel, text) in &fresh {
            let path = out_dir.join(rel);
            match std::fs::read_to_string(&path) {
                Ok(old) if old == *text => {}
                Ok(_) => {
                    eprintln!("differs: {}", path.display());
                    problems += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!("missing: {}", path.display());
                    problems += 1;
                }
                Err(e) => {
                    eprintln!("unreadable: {}: {e}", path.display());
                    problems += 1;
                }
            }
        }
        for r in &stale {
            let p = out_dir.join(r);
            if p.symlink_metadata().is_ok() {
                eprintln!("left over: {} (no diagram renders it any more)", p.display());
                problems += 1;
            }
        }
        if std::fs::read_to_string(out_dir.join(MANIFEST)).ok().as_deref() != Some(new_manifest.as_str()) && failed == 0 {
            eprintln!("differs: {}", out_dir.join(MANIFEST).display());
            problems += 1;
        }
        if problems > 0 {
            eprintln!("{problems} problem(s) under {}; regenerate with {regenerate}", out_dir.display());
            return 1;
        }
        eprintln!("{} diagram file(s) up to date under {}", fresh.len(), out_dir.display());
        return 0;
    }
    for (rel, text) in &fresh {
        let path = out_dir.join(rel);
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("error: cannot create {}: {e}", parent.display());
                return 1;
            }
        }
        // rewrite only what changed, so an unchanged file keeps its timestamp
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("error: cannot write {}: {e}", path.display());
                return 1;
            }
        }
    }
    if failed > 0 {
        // a diagram that failed to render keeps its last good file, and the manifest keeps naming it
        eprintln!("{failed} diagram(s) failed to render; wrote {} file(s) under {} and removed nothing", fresh.len(), out_dir.display());
        let kept: std::collections::BTreeSet<String> = old.iter().cloned().chain(fresh.keys().cloned()).collect();
        let _ = write_manifest(&out_dir, &manifest_text(&kept));
        return 1;
    }
    for r in &stale {
        let p = out_dir.join(r);
        // a plain file the export wrote, inside the directory: never a link, nor a path through one that leads out
        let inside = match (p.parent().and_then(|d| d.canonicalize().ok()), out_dir.canonicalize()) {
            (Some(d), Ok(o)) => d.starts_with(o),
            _ => false,
        };
        match p.symlink_metadata() {
            Ok(m) if m.file_type().is_file() && inside => {
                if let Err(e) = std::fs::remove_file(&p) {
                    eprintln!("error: cannot remove {}: {e}", p.display());
                    return 1;
                }
                eprintln!("removed {}", p.display());
            }
            _ => {}
        }
    }
    if let Err(e) = write_manifest(&out_dir, &new_manifest) {
        eprintln!("error: cannot write {}: {e}", out_dir.join(MANIFEST).display());
        return 1;
    }
    eprintln!("wrote {} diagram file(s) under {}", fresh.len(), out_dir.display());
    0
}

/// The relative paths a previous `--all` run recorded in the output directory (none when there is no manifest).
fn read_manifest(out_dir: &Path) -> Vec<String> {
    std::fs::read_to_string(out_dir.join(MANIFEST))
        .map(|t| t.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(String::from).collect())
        .unwrap_or_default()
}

fn manifest_text(entries: &std::collections::BTreeSet<String>) -> String {
    let mut t = String::from("# Written by `syscribe diagram export --all`: the files it generated here. Do not edit.\n");
    for e in entries {
        t.push_str(e);
        t.push('\n');
    }
    t
}

fn write_manifest(out_dir: &Path, text: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let path = out_dir.join(MANIFEST);
    if std::fs::read_to_string(&path).ok().as_deref() == Some(text) {
        return Ok(());
    }
    std::fs::write(path, text)
}

/// A manifest entry that stays inside the output directory: relative, with no `..`.
fn safe_rel(rel: &str) -> bool {
    let p = Path::new(rel);
    !p.is_absolute() && p.components().all(|c| matches!(c, std::path::Component::Normal(_)))
}

/// `p` made absolute against the working directory, with `.` and `..` resolved lexically (no file need exist).
fn normalize(p: &Path) -> PathBuf {
    let abs = std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// The path that leads from directory `from` to `to`, both absolute and normalized.
fn relative_to(to: &Path, from: &Path) -> PathBuf {
    let (t, f): (Vec<_>, Vec<_>) = (to.components().collect(), from.components().collect());
    let common = t.iter().zip(&f).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in common..f.len() {
        out.push("..");
    }
    for c in &t[common..] {
        out.push(c);
    }
    out
}

/// Print the export, or write it to `out_file`; the exit code.
fn emit(result: Result<String, String>, out_file: Option<&str>) -> i32 {
    match result {
        Ok(text) => match out_file {
            Some(path) => {
                let p = Path::new(path);
                if let Some(parent) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("error: cannot create {}: {e}", parent.display());
                        return 1;
                    }
                }
                if let Err(e) = std::fs::write(p, &text) {
                    eprintln!("error: cannot write {}: {e}", p.display());
                    return 1;
                }
                eprintln!("wrote {}", p.display());
                0
            }
            None => {
                print!("{text}");
                0
            }
        },
        Err(msg) => {
            eprintln!("error: {msg}");
            1
        }
    }
}
