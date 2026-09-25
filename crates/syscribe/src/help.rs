//! Detailed per-command help (REQ-TRS-CLI-005). Mirrors `spec.rs`: each command's
//! man page is an embedded markdown file under `prompts/help/`, surfaced by
//! `syscribe help <command>` and `syscribe <command> --help`/`-h`.

/// The command registry, grouped for the `--help` / `help` indexes: `(group title,
/// [(command name, embedded man page)])`. One entry per dispatchable command; a
/// missing `prompts/help/*.md` is a compile error, so this doubles as a build-time
/// coverage guard. Both indexes are generated from this table (and each page's
/// `## SYNOPSIS`), so neither can omit a command.
const HELP: &[(&str, &[(&str, &str)])] = &[
    (
        "Validation & reporting",
        &[
            ("report", include_str!("../../../prompts/help/report.md")),
            ("validate", include_str!("../../../prompts/help/validate.md")),
            ("audit", include_str!("../../../prompts/help/audit.md")),
            ("lint-docs", include_str!("../../../prompts/help/lint-docs.md")),
        ],
    ),
    (
        "Large-model overview & search",
        &[
            ("stats", include_str!("../../../prompts/help/stats.md")),
            ("digest", include_str!("../../../prompts/help/digest.md")),
            ("search-text", include_str!("../../../prompts/help/search-text.md")),
            ("summarize", include_str!("../../../prompts/help/summarize.md")),
            ("topics", include_str!("../../../prompts/help/topics.md")),
            ("clusters", include_str!("../../../prompts/help/clusters.md")),
        ],
    ),
    (
        "Browsing",
        &[
            ("show", include_str!("../../../prompts/help/show.md")),
            ("ls", include_str!("../../../prompts/help/ls.md")),
            ("tree", include_str!("../../../prompts/help/tree.md")),
            ("find", include_str!("../../../prompts/help/find.md")),
            ("extref", include_str!("../../../prompts/help/extref.md")),
            ("list", include_str!("../../../prompts/help/list.md")),
            ("types", include_str!("../../../prompts/help/types.md")),
            ("untyped", include_str!("../../../prompts/help/untyped.md")),
            ("connectivity", include_str!("../../../prompts/help/connectivity.md")),
        ],
    ),
    (
        "Export",
        &[
            ("export", include_str!("../../../prompts/help/export.md")),
            ("export-html", include_str!("../../../prompts/help/export-html.md")),
            ("export-reqif", include_str!("../../../prompts/help/export-reqif.md")),
            ("sbom", include_str!("../../../prompts/help/sbom.md")),
        ],
    ),
    (
        "Traceability & coverage",
        &[
            ("trace", include_str!("../../../prompts/help/trace.md")),
            ("why", include_str!("../../../prompts/help/why.md")),
            ("who-verifies", include_str!("../../../prompts/help/who-verifies.md")),
            ("links", include_str!("../../../prompts/help/links.md")),
            ("follow", include_str!("../../../prompts/help/follow.md")),
            ("link-types", include_str!("../../../prompts/help/link-types.md")),
            ("refs", include_str!("../../../prompts/help/refs.md")),
            ("impact", include_str!("../../../prompts/help/impact.md")),
            ("n2", include_str!("../../../prompts/help/n2.md")),
            ("matrix", include_str!("../../../prompts/help/matrix.md")),
            ("verification-depth", include_str!("../../../prompts/help/verification-depth.md")),
            ("testplan", include_str!("../../../prompts/help/testplan.md")),
        ],
    ),
    (
        "Suspect links & release baselines",
        &[
            ("suspect", include_str!("../../../prompts/help/suspect.md")),
            ("baseline", include_str!("../../../prompts/help/baseline.md")),
        ],
    ),
    (
        "Safety & security analysis",
        &[
            ("metrics", include_str!("../../../prompts/help/metrics.md")),
            ("cyber-risk", include_str!("../../../prompts/help/cyber-risk.md")),
            ("co-analysis", include_str!("../../../prompts/help/co-analysis.md")),
            ("safety-case", include_str!("../../../prompts/help/safety-case.md")),
            ("fmea", include_str!("../../../prompts/help/fmea.md")),
            ("fault-tree", include_str!("../../../prompts/help/fault-tree.md")),
            ("behavioral-coverage", include_str!("../../../prompts/help/behavioral-coverage.md")),
            ("zones", include_str!("../../../prompts/help/zones.md")),
            ("conduits", include_str!("../../../prompts/help/conduits.md")),
        ],
    ),
    (
        "Reviews",
        &[
            ("reviews", include_str!("../../../prompts/help/reviews.md")),
            ("review", include_str!("../../../prompts/help/review.md")),
        ],
    ),
    (
        "MagicGrid",
        &[
            ("magicgrid", include_str!("../../../prompts/help/magicgrid.md")),
            ("trade-study", include_str!("../../../prompts/help/trade-study.md")),
        ],
    ),
    (
        "Product lines (variability)",
        &[
            ("feature-check", include_str!("../../../prompts/help/feature-check.md")),
            ("features", include_str!("../../../prompts/help/features.md")),
            ("feature", include_str!("../../../prompts/help/feature.md")),
            ("why-active", include_str!("../../../prompts/help/why-active.md")),
            ("configure", include_str!("../../../prompts/help/configure.md")),
            ("build-config", include_str!("../../../prompts/help/build-config.md")),
            ("diff", include_str!("../../../prompts/help/diff.md")),
            ("applies-when", include_str!("../../../prompts/help/applies-when.md")),
        ],
    ),
    (
        "Authoring helpers",
        &[
            ("template", include_str!("../../../prompts/help/template.md")),
            ("next-id", include_str!("../../../prompts/help/next-id.md")),
            ("check-ref", include_str!("../../../prompts/help/check-ref.md")),
            ("path-for", include_str!("../../../prompts/help/path-for.md")),
            ("move", include_str!("../../../prompts/help/move.md")),
            ("set", include_str!("../../../prompts/help/set.md")),
            ("scaffold-gherkin", include_str!("../../../prompts/help/scaffold-gherkin.md")),
            ("ingest-results", include_str!("../../../prompts/help/ingest-results.md")),
        ],
    ),
    (
        "Planning items (multi-agent coordination)",
        &[
            ("claim", include_str!("../../../prompts/help/claim.md")),
            ("release", include_str!("../../../prompts/help/release.md")),
        ],
    ),
    (
        "Diagrams",
        &[
            ("render", include_str!("../../../prompts/help/render.md")),
            ("diagram", include_str!("../../../prompts/help/diagram.md")),
            ("plantuml", include_str!("../../../prompts/help/plantuml.md")),
        ],
    ),
    (
        "Multi-repo & foreign-source ingestion",
        &[
            ("repos", include_str!("../../../prompts/help/repos.md")),
            ("plugins", include_str!("../../../prompts/help/plugins.md")),
            ("annotations", include_str!("../../../prompts/help/annotations.md")),
        ],
    ),
    (
        "Editor, agent & extension integration",
        &[
            ("mcp", include_str!("../../../prompts/help/mcp.md")),
            ("lsp", include_str!("../../../prompts/help/lsp.md")),
            ("scripts", include_str!("../../../prompts/help/scripts.md")),
        ],
    ),
    (
        "Reference",
        &[
            ("spec", include_str!("../../../prompts/help/spec.md")),
            ("help", include_str!("../../../prompts/help/help.md")),
        ],
    ),
];

/// Every `(command name, man page)` in registry order.
fn entries() -> impl Iterator<Item = (&'static str, &'static str)> {
    HELP.iter().flat_map(|(_, cmds)| cmds.iter().copied())
}

/// The command registry: `(name, one-line summary)` for every command that has a man
/// page. This is the single source of truth the clap router derives its subcommands
/// from (REQ-TRS-CLI-008), so the router cannot drift from the help pages.
pub fn commands() -> impl Iterator<Item = (&'static str, &'static str)> {
    entries().map(|(name, body)| (name, summary(body)))
}

/// The man page for a command, if one exists.
pub fn page(cmd: &str) -> Option<&'static str> {
    entries().find(|(name, _)| *name == cmd).map(|(_, body)| body)
}

/// True if `cmd` is a command we have help for (used to recognise `<cmd> --help`).
pub fn is_command(cmd: &str) -> bool {
    entries().any(|(name, _)| name == cmd)
}

/// One-line summary extracted from a page's H1 (`# name — summary`).
fn summary(body: &str) -> &str {
    let first = body.lines().next().unwrap_or("");
    // Split on the em dash used in every H1; fall back to the whole line.
    first.split_once('—').map(|(_, s)| s.trim()).unwrap_or(first.trim_start_matches("# ").trim())
}

/// The non-blank lines of a page's `## SYNOPSIS` section (code fences dropped),
/// verbatim. Every page has one (guarded by a test), so the generated index always
/// carries each command's current synopsis.
pub fn synopsis(body: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in body.lines() {
        if line.starts_with("## ") {
            if inside {
                break;
            }
            inside = line.trim() == "## SYNOPSIS";
            continue;
        }
        if inside && !line.trim().is_empty() && !line.trim_start().starts_with("```") {
            out.push(line);
        }
    }
    out
}

/// One synopsis line as shown in the `--help` index: the leading
/// `syscribe -m <root> ` is dropped (the index states it once), continuation lines
/// are re-indented under their invocation.
fn index_synopsis_line(line: &str) -> String {
    let t = line.trim();
    for prefix in ["syscribe -m <root> ", "syscribe -m <model> "] {
        if let Some(rest) = t.strip_prefix(prefix) {
            return format!("      {rest}");
        }
    }
    if t.starts_with("syscribe") {
        return format!("      {t}");
    }
    format!("          {t}")
}

/// The top-level `syscribe --help` text. The command list is generated from the
/// registry (group, one-line summary, and each page's `## SYNOPSIS`), so it cannot
/// omit or misdescribe a command; only the cross-cutting sections are hand-written.
pub fn usage_text() -> String {
    let mut s = String::from(USAGE_HEAD);
    for (group, cmds) in HELP {
        s.push('\n');
        s.push_str(group);
        s.push_str(":\n");
        for (name, body) in *cmds {
            s.push_str(&format!("  {:<20} {}\n", name, summary(body)));
            for line in synopsis(body) {
                s.push_str(&index_synopsis_line(line));
                s.push('\n');
            }
        }
    }
    s.push_str(USAGE_TAIL);
    s
}

const USAGE_HEAD: &str = "\
Usage: syscribe [-m <root>] <command> [args...]

Model root (priority order):
  -m / --model <path>            Explicit flag
  SYSCRIBE_MODEL=<path>          Environment variable
  .syscribe.toml                 Auto-discovered by walking up from the current dir
  model/                         Default fallback

Commands (synopses omit the leading `syscribe -m <root>`; run `syscribe help <command>`
or `syscribe <command> --help` for the full page with every option and example):
";

const USAGE_TAIL: &str = "
Configuration lens (§9 projection; with no feature model only a stored Configuration is accepted):
  --config <CONF|features>       On validate/list/export and most read commands: project onto a
                                 configuration (stored id/qname or ad-hoc 'Features::A,Features::B').
                                 validate --config certifies the variant and flags escaping refs
                                 (E226 structural / W019 traceability).
  validate --all-configs         Validate every stored Configuration; gates apply per variant.

Exit codes (validate; other gating commands document theirs on their help page):
  0                              No errors and no gate failures
  1                              One or more Error-severity findings, or a usage error (undefined
                                 --profile, unresolvable --config, malformed flag value)
  2                              Warnings tripped a gate (--deny / --max-warnings / --warnings-as-errors / --profile)
                                 — the same in every mode (--config, --all-configs per variant; 1 > 2 > 0)

Usage errors (all commands): an unknown option on the commands that check theirs, an option
value outside its documented set (e.g. impact --direction, --format), or a non-integer count
(e.g. n2 --depth) prints a message on stderr, nothing on stdout, and exits 1.

Options:
  -m, --model <path>             Model root directory
  --agent-instructions [topic]   Print the LLM authoring prompt; topic 'magicgrid' teaches MagicGrid modeling
  --version, -V                  Print the tool version (also `syscribe [-m <root>] version`)
  --help, -h                     Show this help

Examples (against the bundled model/ demo):
  syscribe -m model/ validate
  syscribe -m model/ validate --json
  syscribe -m model/ validate --file model/UAV/Avionics/FlightController.md
  syscribe -m model/ list PartDef
  syscribe -m model/ list PortDef UAV::Avionics
  syscribe -m model/ matrix
  syscribe -m model/ path-for UAV::Avionics::FlightController
  syscribe -m model/ check-ref Interfaces::TelemetryPortDef
  syscribe -m model/ next-id REQ-UAV-FC
  syscribe -m model/ template Requirement
  syscribe -m model/ find FlightController
  syscribe -m model/ show UAV::Avionics::FlightController
  syscribe -m model/ tree UAV
  syscribe -m model/ trace REQ-UAV-FC-001
  syscribe -m model/ who-verifies REQ-UAV-SAFE-001
  syscribe -m model/ refs Interfaces::TelemetryPortDef
  SYSCRIBE_MODEL=model/ syscribe validate

Detailed help: `syscribe help <command>` or `syscribe <command> --help` (e.g. `syscribe help audit`).
";

/// `syscribe help [<command>]`.
pub fn cmd_help(arg: Option<&str>) {
    match arg {
        None => print_index(),
        Some(cmd) => match page(cmd) {
            Some(body) => print!("{}", body),
            None => {
                eprintln!("No detailed help for command: {cmd}");
                eprintln!();
                print_index_to(&mut std::io::stderr());
                std::process::exit(1);
            }
        },
    }
}

fn print_index() {
    print_index_to(&mut std::io::stdout());
}

fn print_index_to(w: &mut dyn std::io::Write) {
    let _ = writeln!(w, "syscribe — detailed command help");
    let _ = writeln!(w);
    let _ = writeln!(w, "Run `syscribe help <command>` or `syscribe <command> --help` for a full page.");
    for (group, cmds) in HELP {
        let _ = writeln!(w);
        let _ = writeln!(w, "{group}:");
        for (name, body) in *cmds {
            let _ = writeln!(w, "  {:<20} {}", name, summary(body));
        }
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "See also: `syscribe spec` (format reference), `syscribe --agent-instructions` (LLM prompt).");
}

#[cfg(test)]
mod index_tests {
    use super::*;

    /// Every registered command has a `## SYNOPSIS` section whose first line is a
    /// `syscribe` invocation naming it, so the generated `--help` index shows a real
    /// synopsis for each.
    #[test]
    fn every_page_has_a_synopsis_invoking_its_command() {
        for (name, body) in entries() {
            let syn = synopsis(body);
            assert!(!syn.is_empty(), "prompts/help/{name}.md has no `## SYNOPSIS` section");
            let first = syn[0].trim();
            assert!(first.starts_with("syscribe"), "{name}: SYNOPSIS must start with a `syscribe` invocation: {first}");
            assert!(
                syn.iter().any(|l| l.split(|c: char| c.is_whitespace() || c == '[' || c == ']').any(|t| t == name)),
                "{name}: SYNOPSIS never names the command"
            );
        }
    }

    /// The `--help` index and the `help` index both list every registered command.
    #[test]
    fn both_indexes_list_every_command() {
        let usage = usage_text();
        let mut idx = Vec::new();
        print_index_to(&mut idx);
        let idx = String::from_utf8(idx).unwrap();
        for (name, _) in commands() {
            let row = format!("\n  {name:<20} ");
            assert!(usage.contains(&row), "`syscribe --help` omits {name}");
            assert!(idx.contains(&row), "`syscribe help` omits {name}");
        }
    }

    /// The user-facing CLI reference (`docs/cli/index.md`) covers every registered
    /// command: either a heading naming it as a code span (`` ## … (`stats`) ``) or a
    /// `syscribe [-m <root>] <command>` invocation in the page.
    #[test]
    fn cli_reference_covers_every_command() {
        let doc = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/cli/index.md"),
        )
        .expect("read docs/cli/index.md");
        let headings: Vec<&str> = doc.lines().filter(|l| l.starts_with('#')).collect();
        let invoked = regex::Regex::new(r"syscribe(?:\s+(?:-m|--model)\s+\S+)?\s+([a-z][a-z0-9-]*)").unwrap();
        let invoked: std::collections::HashSet<&str> =
            invoked.captures_iter(&doc).filter_map(|c| c.get(1)).map(|m| m.as_str()).collect();
        let missing: Vec<&str> = commands()
            .map(|(n, _)| n)
            .filter(|n| !invoked.contains(n) && !headings.iter().any(|h| h.contains(&format!("`{n}`"))))
            .collect();
        assert!(missing.is_empty(), "docs/cli/index.md has no section or invocation for: {missing:?}");
    }

    /// Each command appears in exactly one group.
    #[test]
    fn registry_has_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for (name, _) in entries() {
            assert!(seen.insert(name), "{name} registered twice");
        }
    }
}

#[cfg(test)]
mod prompt_syntax_tests {
    use super::*;
    use regex::Regex;
    use std::sync::LazyLock;

    /// Every prompt an LLM agent is handed via `--agent-instructions`.
    const PROMPTS: &[(&str, &str)] = &[
        ("create-model.md", include_str!("../../../prompts/create-model.md")),
        ("create-magicgrid-model.md", include_str!("../../../prompts/create-magicgrid-model.md")),
    ];

    /// Subcommands whose positional argument genuinely *is* a filesystem path, so a
    /// directory there is not a misplaced model root (`lint-docs docs/`,
    /// `render model/Diagrams/X.md`, `ingest-results <file>`).
    const PATH_TAKING: &[&str] = &["lint-docs", "render", "ingest-results"];

    /// Known offenders, matched by (repo-relative file, exact line text); the line
    /// number is informational. Empty: every documented invocation passes the model
    /// root with `-m`. Add an entry only for a deliberate, commented exception.
    const KNOWN_OFFENDERS: &[(&str, usize, &str)] = &[];

    fn repo_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// Any relative directory path (`model/`, `my_model/`, `examples/x/`).
    fn is_dir_path(tok: &str) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_.~][A-Za-z0-9_.~/-]*/$").unwrap());
        RE.is_match(tok)
    }

    /// A model-root-looking directory: `model/`, `model_auto/`, `./model/`,
    /// `examples/<...>/` (e.g. `examples/foo/model/`).
    fn is_model_root(tok: &str) -> bool {
        let t = tok.strip_prefix("./").unwrap_or(tok);
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^(model|model_[A-Za-z0-9_]+|examples/[A-Za-z0-9_./-]+)/$").unwrap());
        RE.is_match(t)
    }

    /// The whitespace-separated arguments of every `syscribe …` /
    /// `cargo run --package syscribe[-server] … -- …` invocation on `line`, each cut
    /// at the end of the command (backtick, pipe, `;`, `&`, `<`, quote, comment).
    fn invocations(line: &str) -> Vec<Vec<&str>> {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"(?:^|[^A-Za-z0-9_.-])(?:syscribe|cargo\s+run\s+(?:[^\s|;&`<]+\s+)*?(?:--package|-p)\s+syscribe(?:-server)?(?:\s+[^\s|;&`<]+)*?\s+--)(?:\s|$)",
            )
            .unwrap()
        });
        RE.find_iter(line)
            .map(|m| {
                let rest = &line[m.end()..];
                let end = rest.find(['`', '|', ';', '&', '<', '"', '\'', '#', '>']).unwrap_or(rest.len());
                rest[..end].split_whitespace().filter(|t| *t != "\\").collect()
            })
            .collect()
    }

    /// The offending model-root argument of one invocation, if any. `full` also
    /// checks positionals after the subcommand (for agent-facing prompts); otherwise
    /// only a directory passed directly as the first argument is flagged.
    fn offending_arg<'a>(args: &[&'a str], full: bool) -> Option<&'a str> {
        let mut sub: Option<&str> = None;
        let mut i = 0;
        while i < args.len() {
            let a = args[i];
            if a.starts_with('-') {
                // `-m <root>` / `--model <root>` (and, after the subcommand, any
                // `--flag <value>`) consume the next token as a value, not a positional.
                let takes_value = !a.contains('=')
                    && (a == "-m" || a == "--model" || sub.is_some())
                    && args.get(i + 1).is_some_and(|n| !n.starts_with('-'));
                i += if takes_value { 2 } else { 1 };
                continue;
            }
            match sub {
                None => {
                    if is_dir_path(a) {
                        return Some(a); // `syscribe model/ …` — positional root
                    }
                    if !full || PATH_TAKING.contains(&a) {
                        return None;
                    }
                    sub = Some(a);
                }
                Some(_) => {
                    if is_model_root(a) {
                        return Some(a); // `syscribe diagram list model/`
                    }
                }
            }
            i += 1;
        }
        None
    }

    /// Scan `text` (from `file`, repo-relative) and return one message per offending
    /// line not on the temporary allowlist.
    fn scan(file: &str, text: &str, full: bool) -> Vec<String> {
        scan_raw(file, text, full, true)
    }

    fn scan_raw(file: &str, text: &str, full: bool, allowlist: bool) -> Vec<String> {
        let mut out = Vec::new();
        for (i, line) in text.lines().enumerate() {
            for args in invocations(line) {
                if let Some(bad) = offending_arg(&args, full) {
                    if allowlist && KNOWN_OFFENDERS.iter().any(|(f, _, l)| *f == file && *l == line) {
                        continue;
                    }
                    out.push(format!(
                        "{file}:{}: model root '{bad}' passed positionally — use `-m {bad}`: {}",
                        i + 1,
                        line.trim()
                    ));
                }
            }
        }
        out
    }

    fn read_rel(rel: &str) -> String {
        std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
    }

    /// Every `.md` file under `dir` (repo-relative), sorted, as repo-relative paths.
    fn md_files(dir: &str) -> Vec<String> {
        let root = repo_root();
        let mut files: Vec<String> = walkdir::WalkDir::new(root.join(dir))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|x| x == "md"))
            .filter_map(|e| e.path().strip_prefix(&root).ok().map(|p| p.to_string_lossy().replace('\\', "/")))
            .collect();
        files.sort();
        files
    }

    /// The model root is only ever passed with `-m`/`--model`; the router rejects a
    /// positional path (`syscribe model/ show X` → "unrecognized subcommand 'model/'",
    /// `syscribe diagram list model/` → "unexpected argument"), so an agent copying
    /// such an example fails on its first command. Covers every agent-facing prompt
    /// (`prompts/**/*.md`: the `--agent-instructions` prompts, `spec` topics and
    /// `help` pages) and `cargo run --package syscribe[-server] -- <path>/`.
    #[test]
    fn prompts_never_pass_the_model_root_positionally() {
        let mut bad = Vec::new();
        for f in md_files("prompts") {
            bad.extend(scan(&f, &read_rel(&f), true));
        }
        assert!(bad.is_empty(), "positional model root in prompts:\n{}", bad.join("\n"));
    }

    /// The same guard, for the direct forms only (`syscribe <path>/`,
    /// `cargo run --package syscribe[-server] -- <path>/`), over the user docs.
    /// `docs/releases/` is excluded: release notes quote the old, wrong syntax
    /// when describing what a release fixed.
    #[test]
    fn docs_never_pass_the_model_root_positionally() {
        let mut files: Vec<String> =
            md_files("docs").into_iter().filter(|f| !f.starts_with("docs/releases/")).collect();
        files.push("README.md".into());
        files.push("overrides/home.html".into());
        let mut bad = Vec::new();
        for f in &files {
            bad.extend(scan(f, &read_rel(f), false));
        }
        assert!(bad.is_empty(), "positional model root in docs:\n{}", bad.join("\n"));
    }

    /// Every temporary-allowlist entry is a line the guard really flags (in the
    /// scan mode its file is checked with), so the allowlist only ever hides
    /// genuine offenders.
    #[test]
    fn known_offenders_are_real_offenders() {
        for (file, line_no, line) in KNOWN_OFFENDERS {
            let full = file.starts_with("prompts/");
            assert!(
                !scan_raw(file, line, full, false).is_empty(),
                "{file}:{line_no}: allowlisted line is not flagged by the guard: {line}"
            );
            assert!(scan(file, line, full).is_empty(), "{file}:{line_no}: allowlist entry not honoured");
        }
    }

    /// The guard itself: it catches every known-bad form and passes the good ones.
    #[test]
    fn positional_root_guard_catches_known_forms() {
        let bad = [
            "syscribe model/ show X",
            "./target/debug/syscribe model_auto/",
            "syscribe diagram list model/",
            "syscribe diagram list model/ --type PartDef",
            "syscribe diagram measure model/ \\",
            "syscribe diagram compose model/ my-arch.layout.json \\",
            "syscribe -m model/ diagram list examples/foo/model/",
            "cargo run --package syscribe -- model/",
            "cargo run --package syscribe -- model/ > reports/validation.md",
            "cargo run -p syscribe-server -- model/",
            "cargo run --release --package syscribe-server -- model_sil/",
            "<code>cargo run --package syscribe-server -- model/</code>",
            "`syscribe model/ validate`",
        ];
        for l in bad {
            assert!(!scan("t.md", l, true).is_empty(), "not caught: {l}");
        }
        let good = [
            "syscribe -m model/ show X",
            "syscribe --model model_auto/ validate",
            "syscribe -m model/ diagram list",
            "syscribe -m model/ diagram compose my.layout.json --output model/Views/X.svg",
            "syscribe -m model/ lint-docs docs/",
            "syscribe -m model/ render model/Diagrams/SystemBDD.md",
            "cargo run --package syscribe -- -m model/",
            "cargo run --package syscribe-server -- -m model/",
            "the .syscribe/results.json sidecar under model/",
            "syscribe-model/ crate",
        ];
        for l in good {
            assert!(scan("t.md", l, true).is_empty(), "false positive: {l}");
        }
        // Direct-only mode (docs): a post-subcommand root is out of scope.
        assert!(scan("t.md", "syscribe diagram list model/", false).is_empty());
        assert!(!scan("t.md", "cargo run --package syscribe -- model/", false).is_empty());
    }

    /// Every `syscribe -m <root> <command>` example names a real subcommand.
    #[test]
    fn prompt_examples_name_real_subcommands() {
        let known: std::collections::HashSet<&str> =
            commands().map(|(n, _)| n).chain(["help", "spec", "version"]).collect();
        let re = regex::Regex::new(r"syscribe\s+(?:-m|--model)\s+\S+\s+([a-z][a-z-]*)\b").unwrap();
        for (name, text) in PROMPTS {
            for (i, line) in text.lines().enumerate() {
                for c in re.captures_iter(line) {
                    assert!(known.contains(&c[1]), "{name}:{}: unknown subcommand '{}'", i + 1, &c[1]);
                }
            }
        }
    }
}
