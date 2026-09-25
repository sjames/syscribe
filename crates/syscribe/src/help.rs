//! Detailed per-command help (REQ-TRS-CLI-005). Mirrors `spec.rs`: each command's
//! man page is an embedded markdown file under `prompts/help/`, surfaced by
//! `syscribe help <command>` and `syscribe <command> --help`/`-h`.

/// (command name, embedded man page). One entry per dispatchable command; a
/// missing `prompts/help/*.md` is a compile error, so this doubles as a
/// build-time coverage guard. Ordered by group for the index.
const HELP: &[(&str, &str)] = &[
    // Core
    ("validate", include_str!("../../../prompts/help/validate.md")),
    ("report", include_str!("../../../prompts/help/report.md")),
    ("audit", include_str!("../../../prompts/help/audit.md")),
    ("stats", include_str!("../../../prompts/help/stats.md")),
    ("digest", include_str!("../../../prompts/help/digest.md")),
    ("search-text", include_str!("../../../prompts/help/search-text.md")),
    ("summarize", include_str!("../../../prompts/help/summarize.md")),
    ("topics", include_str!("../../../prompts/help/topics.md")),
    ("clusters", include_str!("../../../prompts/help/clusters.md")),
    // Browsing
    ("show", include_str!("../../../prompts/help/show.md")),
    ("ls", include_str!("../../../prompts/help/ls.md")),
    ("tree", include_str!("../../../prompts/help/tree.md")),
    ("find", include_str!("../../../prompts/help/find.md")),
    ("extref", include_str!("../../../prompts/help/extref.md")),
    ("list", include_str!("../../../prompts/help/list.md")),
    ("types", include_str!("../../../prompts/help/types.md")),
    ("untyped", include_str!("../../../prompts/help/untyped.md")),
    ("connectivity", include_str!("../../../prompts/help/connectivity.md")),
    ("export", include_str!("../../../prompts/help/export.md")),
    ("export-html", include_str!("../../../prompts/help/export-html.md")),
    // Traceability
    ("trace", include_str!("../../../prompts/help/trace.md")),
    ("why", include_str!("../../../prompts/help/why.md")),
    ("who-verifies", include_str!("../../../prompts/help/who-verifies.md")),
    ("links", include_str!("../../../prompts/help/links.md")),
    ("follow", include_str!("../../../prompts/help/follow.md")),
    ("link-types", include_str!("../../../prompts/help/link-types.md")),
    ("refs", include_str!("../../../prompts/help/refs.md")),
    ("suspect", include_str!("../../../prompts/help/suspect.md")),
    ("baseline", include_str!("../../../prompts/help/baseline.md")),
    ("matrix", include_str!("../../../prompts/help/matrix.md")),
    ("magicgrid", include_str!("../../../prompts/help/magicgrid.md")),
    ("trade-study", include_str!("../../../prompts/help/trade-study.md")),
    ("verification-depth", include_str!("../../../prompts/help/verification-depth.md")),
    ("testplan", include_str!("../../../prompts/help/testplan.md")),
    // Safety / security analysis
    ("metrics", include_str!("../../../prompts/help/metrics.md")),
    ("cyber-risk", include_str!("../../../prompts/help/cyber-risk.md")),
    ("co-analysis", include_str!("../../../prompts/help/co-analysis.md")),
    ("safety-case", include_str!("../../../prompts/help/safety-case.md")),
    ("behavioral-coverage", include_str!("../../../prompts/help/behavioral-coverage.md")),
    ("sbom", include_str!("../../../prompts/help/sbom.md")),
    ("export-reqif", include_str!("../../../prompts/help/export-reqif.md")),
    ("zones", include_str!("../../../prompts/help/zones.md")),
    ("conduits", include_str!("../../../prompts/help/conduits.md")),
    ("repos", include_str!("../../../prompts/help/repos.md")),
    ("plugins", include_str!("../../../prompts/help/plugins.md")),
    ("annotations", include_str!("../../../prompts/help/annotations.md")),
    ("impact", include_str!("../../../prompts/help/impact.md")),
    ("n2", include_str!("../../../prompts/help/n2.md")),
    ("reviews", include_str!("../../../prompts/help/reviews.md")),
    ("review", include_str!("../../../prompts/help/review.md")),
    ("fmea", include_str!("../../../prompts/help/fmea.md")),
    ("fault-tree", include_str!("../../../prompts/help/fault-tree.md")),
    ("lint-docs", include_str!("../../../prompts/help/lint-docs.md")),
    // Product lines
    ("feature-check", include_str!("../../../prompts/help/feature-check.md")),
    ("features", include_str!("../../../prompts/help/features.md")),
    ("feature", include_str!("../../../prompts/help/feature.md")),
    ("why-active", include_str!("../../../prompts/help/why-active.md")),
    ("configure", include_str!("../../../prompts/help/configure.md")),
    ("build-config", include_str!("../../../prompts/help/build-config.md")),
    ("diff", include_str!("../../../prompts/help/diff.md")),
    // Authoring helpers
    ("template", include_str!("../../../prompts/help/template.md")),
    ("next-id", include_str!("../../../prompts/help/next-id.md")),
    ("check-ref", include_str!("../../../prompts/help/check-ref.md")),
    ("path-for", include_str!("../../../prompts/help/path-for.md")),
    ("move", include_str!("../../../prompts/help/move.md")),
    ("set", include_str!("../../../prompts/help/set.md")),
    ("claim", include_str!("../../../prompts/help/claim.md")),
    ("release", include_str!("../../../prompts/help/release.md")),
    ("mcp", include_str!("../../../prompts/help/mcp.md")),
    ("lsp", include_str!("../../../prompts/help/lsp.md")),
    ("applies-when", include_str!("../../../prompts/help/applies-when.md")),
    ("scaffold-gherkin", include_str!("../../../prompts/help/scaffold-gherkin.md")),
    ("ingest-results", include_str!("../../../prompts/help/ingest-results.md")),
    ("plantuml", include_str!("../../../prompts/help/plantuml.md")),
    ("render", include_str!("../../../prompts/help/render.md")),
    ("diagram", include_str!("../../../prompts/help/diagram.md")),
    ("scripts", include_str!("../../../prompts/help/scripts.md")),
    ("spec", include_str!("../../../prompts/help/spec.md")),
    ("help", include_str!("../../../prompts/help/help.md")),
];

/// The command registry: `(name, one-line summary)` for every command that has a man
/// page. This is the single source of truth the clap router derives its subcommands
/// from (REQ-TRS-CLI-008), so the router cannot drift from the help pages.
pub fn commands() -> impl Iterator<Item = (&'static str, &'static str)> {
    HELP.iter().map(|(name, body)| (*name, summary(body)))
}

/// The man page for a command, if one exists.
pub fn page(cmd: &str) -> Option<&'static str> {
    HELP.iter().find(|(name, _)| *name == cmd).map(|(_, body)| *body)
}

/// True if `cmd` is a command we have help for (used to recognise `<cmd> --help`).
pub fn is_command(cmd: &str) -> bool {
    HELP.iter().any(|(name, _)| *name == cmd)
}

/// One-line summary extracted from a page's H1 (`# name — summary`).
fn summary(body: &str) -> &str {
    let first = body.lines().next().unwrap_or("");
    // Split on the em dash used in every H1; fall back to the whole line.
    first.split_once('—').map(|(_, s)| s.trim()).unwrap_or(first.trim_start_matches("# ").trim())
}

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
    let _ = writeln!(w);
    for (name, body) in HELP {
        let _ = writeln!(w, "  {:<20} {}", name, summary(body));
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "See also: `syscribe spec` (format reference), `syscribe --agent-instructions` (LLM prompt).");
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
