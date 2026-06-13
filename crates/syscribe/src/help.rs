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
    // Traceability
    ("trace", include_str!("../../../prompts/help/trace.md")),
    ("why", include_str!("../../../prompts/help/why.md")),
    ("who-verifies", include_str!("../../../prompts/help/who-verifies.md")),
    ("links", include_str!("../../../prompts/help/links.md")),
    ("refs", include_str!("../../../prompts/help/refs.md")),
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
    ("diff", include_str!("../../../prompts/help/diff.md")),
    // Authoring helpers
    ("template", include_str!("../../../prompts/help/template.md")),
    ("next-id", include_str!("../../../prompts/help/next-id.md")),
    ("check-ref", include_str!("../../../prompts/help/check-ref.md")),
    ("path-for", include_str!("../../../prompts/help/path-for.md")),
    ("move", include_str!("../../../prompts/help/move.md")),
    ("applies-when", include_str!("../../../prompts/help/applies-when.md")),
    ("scaffold-gherkin", include_str!("../../../prompts/help/scaffold-gherkin.md")),
    ("ingest-results", include_str!("../../../prompts/help/ingest-results.md")),
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
