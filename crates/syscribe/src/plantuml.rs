use std::path::{Path, PathBuf};

use syscribe_model::config::PlantumlConfig;
use syscribe_model::element::RawElement;

/// Entry point for `syscribe -m <root> plantuml [<qname>] [--output <file|-]> [--dry-run]`.
///
/// Batch (no qname): generates `.puml` companion files for every `Diagram` element
/// that has `pumlMode: companion` set (REQ-TRS-PUML-010).
///
/// Single (qname given): generates the `.puml` for that one element regardless of
/// whether `pumlMode` is set (REQ-TRS-PUML-011).
pub fn cmd_plantuml(elements: &[RawElement], args: &[String], cfg: &PlantumlConfig) {
    // `render` is a sub-subcommand — hand off before any other parsing.
    if args.first().map(|s| s.as_str()) == Some("render") {
        cmd_render(elements, &args[1..], cfg);
        return;
    }

    let dry_run = args.iter().any(|a| a == "--dry-run");

    // --output / -o  (single-element mode only)
    let output: Option<&str> = {
        let mut out = None;
        let mut i = 0;
        while i < args.len() {
            if (args[i] == "--output" || args[i] == "-o") && i + 1 < args.len() {
                out = Some(args[i + 1].as_str());
                break;
            }
            if let Some(val) = args[i].strip_prefix("--output=") {
                out = Some(val);
                break;
            }
            i += 1;
        }
        out
    };

    // First non-flag positional argument
    let qname: Option<&str> = {
        let mut found = None;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--output" || args[i] == "-o" {
                i += 2;
                continue;
            }
            if args[i].starts_with("--") || (args[i].starts_with('-') && args[i].len() == 2) {
                i += 1;
                continue;
            }
            found = Some(args[i].as_str());
            break;
        }
        found
    };

    if output.is_some() && qname.is_none() {
        eprintln!("error: --output requires a <qname> argument");
        std::process::exit(1);
    }

    match qname {
        Some(q) => cmd_single(elements, q, output, dry_run, cfg),
        None => cmd_batch(elements, dry_run, cfg),
    }
}

// ── Single element ────────────────────────────────────────────────────────────

fn cmd_single(elements: &[RawElement], qname: &str, output: Option<&str>, dry_run: bool, cfg: &PlantumlConfig) {
    let elem = match elements.iter().find(|e| e.qualified_name == qname) {
        Some(e) => e,
        None => {
            eprintln!("error: element '{}' not found", qname);
            std::process::exit(1);
        }
    };

    let puml = match syscribe_model::plantuml::render_plantuml(elem, elements, Some(cfg)) {
        Some(s) => s,
        None => {
            let kind = elem.frontmatter.diagram_kind.as_deref().unwrap_or("(none)");
            eprintln!(
                "warn: skipping '{}' — diagramKind '{}' has no PlantUML mapping",
                qname, kind
            );
            std::process::exit(1);
        }
    };

    match output {
        Some("-") => print!("{}", puml),
        Some(path) => write_file(path, &puml, dry_run),
        None => {
            let path = companion_puml_path(elem);
            write_file(path.to_string_lossy().as_ref(), &puml, dry_run);
        }
    }
}

// ── Batch ─────────────────────────────────────────────────────────────────────

fn cmd_batch(elements: &[RawElement], dry_run: bool, cfg: &PlantumlConfig) {
    let companions: Vec<&RawElement> = elements
        .iter()
        .filter(|e| e.frontmatter.puml_mode.as_deref() == Some("companion"))
        .collect();

    if companions.is_empty() {
        println!("No diagrams with `pumlMode: companion` found.");
        return;
    }

    let mut written = 0usize;
    let mut skipped = 0usize;

    for elem in companions {
        let q = &elem.qualified_name;
        match syscribe_model::plantuml::render_plantuml(elem, elements, Some(cfg)) {
            None => {
                let kind = elem.frontmatter.diagram_kind.as_deref().unwrap_or("(none)");
                eprintln!(
                    "warn: skipping '{}' — diagramKind '{}' has no PlantUML mapping",
                    q, kind
                );
                skipped += 1;
            }
            Some(puml) => {
                let path = companion_puml_path(elem);
                write_file(path.to_string_lossy().as_ref(), &puml, dry_run);
                written += 1;
            }
        }
    }

    if !dry_run {
        println!("{} file(s) written, {} skipped.", written, skipped);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Resolve the output `.puml` path from the element's `pumlFile:` field or the
/// default `<stem>.puml` alongside the `.md` file (REQ-TRS-PUML-001).
fn companion_puml_path(elem: &RawElement) -> PathBuf {
    let md_path = Path::new(&elem.file_path);
    let dir = md_path.parent().unwrap_or(Path::new("."));
    match &elem.frontmatter.puml_file {
        Some(pf) => dir.join(pf.trim_start_matches("./")),
        None => md_path.with_extension("puml"),
    }
}

fn write_file(path: &str, content: &str, dry_run: bool) {
    if dry_run {
        println!("{}", path);
        return;
    }
    if let Err(e) = std::fs::write(path, content) {
        eprintln!("error writing '{}': {}", path, e);
        std::process::exit(1);
    }
}

// ── plantuml render ───────────────────────────────────────────────────────────

/// Invoke PlantUML to render `.puml` companion files to SVG (REQ-TRS-PUML-050).
fn cmd_render(elements: &[RawElement], args: &[String], cfg: &PlantumlConfig) {
    let dry_run = args.iter().any(|a| a == "--dry-run");

    // --jar flag (highest priority per REQ-TRS-PUML-051)
    let jar_flag: Option<PathBuf> = {
        let mut found = None;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--jar" && i + 1 < args.len() {
                found = Some(PathBuf::from(&args[i + 1]));
                break;
            }
            if let Some(val) = args[i].strip_prefix("--jar=") {
                found = Some(PathBuf::from(val));
                break;
            }
            i += 1;
        }
        found
    };

    let invocation = resolve_plantuml(jar_flag.as_deref(), cfg);

    let companions: Vec<PathBuf> = elements
        .iter()
        .filter(|e| e.frontmatter.puml_mode.as_deref() == Some("companion"))
        .map(companion_puml_path)
        .collect();

    if companions.is_empty() {
        println!("No diagrams with `pumlMode: companion` found.");
        return;
    }

    if dry_run {
        for p in &companions {
            println!("{}", p.display());
        }
        return;
    }

    let inv = match invocation {
        Some(i) => i,
        None => {
            eprintln!("error: PlantUML not found. Provide it via one of:");
            eprintln!("  --jar <path>              path to plantuml.jar");
            eprintln!("  [plantuml] jar in .syscribe.toml");
            eprintln!("  PLANTUML_JAR env variable  path to plantuml.jar");
            eprintln!("  plantuml on PATH           binary or wrapper script");
            std::process::exit(1);
        }
    };

    let mut rendered = 0usize;
    let mut failed = 0usize;

    for puml_path in &companions {
        if !puml_path.exists() {
            eprintln!(
                "warn: skipping '{}' — .puml file not found (run `syscribe plantuml` first)",
                puml_path.display()
            );
            continue;
        }

        let result = inv.run(puml_path);
        match result {
            Ok(output) if output.status.success() => {
                rendered += 1;
            }
            Ok(output) => {
                eprintln!("error rendering '{}':", puml_path.display());
                if !output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                failed += 1;
            }
            Err(e) => {
                eprintln!("error invoking PlantUML for '{}': {}", puml_path.display(), e);
                failed += 1;
            }
        }
    }

    println!("{} rendered, {} failed.", rendered, failed);
    if failed > 0 {
        std::process::exit(1);
    }
}

// ── PlantUML tool resolution (REQ-TRS-PUML-051) ──────────────────────────────

enum PlantUmlInvocation {
    Jar(PathBuf),    // java -jar <path> -tsvg
    Binary(String),  // <cmd> -tsvg
}

impl PlantUmlInvocation {
    fn run(&self, puml: &Path) -> std::io::Result<std::process::Output> {
        match self {
            Self::Jar(jar) => std::process::Command::new("java")
                .args(["-jar", jar.to_str().unwrap_or(""), "-tsvg"])
                .arg(puml)
                .output(),
            Self::Binary(cmd) => std::process::Command::new(cmd)
                .arg("-tsvg")
                .arg(puml)
                .output(),
        }
    }
}

fn resolve_plantuml(
    jar_flag: Option<&Path>,
    cfg: &PlantumlConfig,
) -> Option<PlantUmlInvocation> {
    // 1. --jar flag
    if let Some(j) = jar_flag {
        return Some(PlantUmlInvocation::Jar(j.to_path_buf()));
    }
    // 2. [plantuml] jar in .syscribe.toml
    if let Some(ref j) = cfg.jar {
        return Some(PlantUmlInvocation::Jar(j.clone()));
    }
    // 3. PLANTUML_JAR env variable
    if let Ok(j) = std::env::var("PLANTUML_JAR") {
        if !j.is_empty() {
            return Some(PlantUmlInvocation::Jar(PathBuf::from(j)));
        }
    }
    // 4. `plantuml` on PATH
    if which_plantuml() {
        return Some(PlantUmlInvocation::Binary("plantuml".to_string()));
    }
    None
}

fn which_plantuml() -> bool {
    std::process::Command::new("plantuml")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}
