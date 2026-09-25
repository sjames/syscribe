//! CLI argument hygiene (REQ-TRS-CLI-009, GH #133).
//!
//! Most commands parse their own flags by hand (`rest.windows(2).find(..)`), which
//! silently ignores an unknown option and silently falls back to a default on an
//! invalid enumerated or numeric value. This module adds, without touching each
//! command's own parser:
//!
//! - a per-command **known-option table** checked before the model is loaded
//!   ([`check_known_options`]) — an unknown `-`/`--` token, or a value-taking
//!   option with no value, is a usage error;
//! - strict value helpers ([`enum_value`], [`usize_value`]) that turn an invalid
//!   value into a usage error naming the valid values.
//!
//! Commands absent from the table are not checked (they either have their own
//! strict parser — `lint-docs`, `follow`, `connectivity`, the clap `diagram`
//! family — or are not yet covered).

/// How an option consumes the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    /// A boolean switch (`--json`).
    Switch,
    /// Takes the next token as its value (`--format json`).
    Value,
    /// Takes a value either as the next token or inline (`--deny W1` / `--deny=W1`).
    ValueOrEq,
}

use Arity::{Switch, Value, ValueOrEq};

type Spec = &'static [(&'static str, Arity)];

/// The `--config` lens accepted (per REQ-TRS-PROJ-001) by the read commands.
const LENS: (&str, Arity) = ("--config", Value);

/// The documented options of each checked command. Keep in sync with the
/// command's own parser in `main.rs` and its help page in `prompts/help/`.
fn spec_for(cmd: &str) -> Option<Spec> {
    Some(match cmd {
        "validate" => &[
            ("--file", Value),
            ("--json", Switch),
            ("--deny", ValueOrEq),
            ("--max-warnings", ValueOrEq),
            ("--warnings-as-errors", Switch),
            ("--profile", Value),
            ("--results", Value),
            ("--format", Value),
            ("--fetch-remote", Switch),
            LENS,
            ("--all-configs", Switch),
        ],
        "list" => &[
            ("--tag", Value),
            LENS,
            ("--feature", Value),
            ("--metadata", Value),
            ("--status", Value),
            ("--sil", Value),
            ("--where", ValueOrEq),
            ("--has-wcet", Switch),
            ("--json", Switch),
        ],
        "show" => &[("--no-related", Switch)],
        "trace" => &[("--linked-only", Switch), LENS],
        "why" | "who-verifies" | "links" | "refs" => &[LENS],
        "tree" => &[],
        "extref" => &[("--json", Switch)],
        "impact" => &[
            ("--direction", Value),
            ("--depth", Value),
            ("--format", Value),
            ("--kinds", Value),
        ],
        // `--json` is a documented no-op spelling (export always emits JSON).
        "export" => &[("--ndjson", Switch), ("--json", Switch), LENS],
        "find" | "ls" => &[("--where", ValueOrEq)],
        "n2" => &[
            ("--depth", Value),
            ("--format", Value),
            ("--interfaces-only", Switch),
            ("--allocations", Switch),
        ],
        "behavioral-coverage" => &[
            ("--depth", Value),
            ("--format", Value),
            ("--uncovered-only", Switch),
            ("--include-planned", Switch),
        ],
        "sbom" => &[
            ("--format", Value),
            LENS,
            ("--scope", Value),
            ("--output", Value),
            ("--include-tests", Switch),
        ],
        "build-config" => &[
            LENS,
            ("--all-configs", Switch),
            ("--format", Value),
            ("--prefix", Value),
            ("--no-validate", Switch),
        ],
        "stats" => &[
            ("--json", Switch),
            ("--group-by", Value),
            ("--status", Value),
            ("--tag", Value),
            LENS,
            ("--package-top-n", Value),
            ("--where", ValueOrEq),
        ],
        "digest" => &[
            ("--json", Switch),
            ("--status", Value),
            ("--tag", Value),
            LENS,
            ("--limit", Value),
            ("--offset", Value),
            ("--where", ValueOrEq),
        ],
        "search-text" => &[
            ("--json", Switch),
            ("--type", Value),
            ("--status", Value),
            LENS,
            ("--limit", Value),
        ],
        "summarize" => &[
            ("--json", Switch),
            ("--no-cache", Switch),
            ("--scope", Value),
            LENS,
            ("--depth", Value),
        ],
        "topics" => &[("--json", Switch), ("--type", Value), LENS, ("--top", Value)],
        "clusters" => &[("--json", Switch), ("--type", Value), LENS, ("--k", Value)],
        "verification-depth" => &[
            ("--sil", Value),
            ("--status", Value),
            ("--json", Switch),
            ("--min-levels", Value),
            LENS,
            ("--plan", Value),
        ],
        _ => return None,
    })
}

/// Whether `cmd`'s options are checked by [`check_known_options`].
#[cfg(test)]
pub fn is_checked(cmd: &str) -> bool {
    spec_for(cmd).is_some()
}

/// Check `args` (the tokens after the command name) against `cmd`'s documented
/// options. `Ok(())` for an unchecked command. A token is an option when it starts
/// with `-` and is longer than one character; the token after a value-taking
/// option is its value (whatever it looks like). Positional arguments are not
/// checked.
pub fn check_known_options(cmd: &str, args: &[String]) -> Result<(), String> {
    let Some(spec) = spec_for(cmd) else { return Ok(()) };
    let known = || {
        let mut names: Vec<&str> = spec.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        if names.is_empty() { "(none)".to_string() } else { names.join(", ") }
    };
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a.len() < 2 || !a.starts_with('-') {
            i += 1;
            continue;
        }
        if let Some(&(name, arity)) = spec.iter().find(|(n, _)| *n == a) {
            if arity != Switch {
                if args.get(i + 1).is_none() {
                    return Err(format!("{cmd}: option '{name}' expects a value"));
                }
                i += 1;
            }
        } else if let Some((name, _)) = a.split_once('=') {
            match spec.iter().find(|(n, _)| *n == name) {
                Some((_, ValueOrEq)) => {}
                Some(_) => {
                    return Err(format!(
                        "{cmd}: option '{a}' — pass the value as a separate argument: {name} <value>"
                    ))
                }
                None => return Err(format!("{cmd}: unknown option '{a}'; valid options: {}", known())),
            }
        } else {
            return Err(format!("{cmd}: unknown option '{a}'; valid options: {}", known()));
        }
        i += 1;
    }
    Ok(())
}

/// The (first) value of `flag` in `args`, if given as `flag <value>` — the same
/// occurrence the commands' own `windows(2).find(..)` parsers read.
fn raw_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].as_str())
}

/// The value of an enumerated option: `default` when absent, the value when it is
/// one of `allowed`, else `Err` naming the valid values.
pub fn enum_value<'a>(
    cmd: &str,
    args: &'a [String],
    flag: &str,
    allowed: &[&'static str],
    default: &'static str,
) -> Result<&'a str, String> {
    match raw_value(args, flag) {
        None => Ok(default),
        Some(v) if allowed.contains(&v) => Ok(v),
        Some(v) => Err(format!(
            "{cmd}: invalid value '{v}' for {flag}; valid values: {}",
            allowed.join(", ")
        )),
    }
}

/// The value of a non-negative integer option: `None` when absent, else the parsed
/// value or `Err` naming the option.
pub fn usize_value(cmd: &str, args: &[String], flag: &str) -> Result<Option<usize>, String> {
    match raw_value(args, flag) {
        None => Ok(None),
        Some(v) => v.parse::<usize>().map(Some).map_err(|_| {
            format!("{cmd}: {flag} expects a non-negative integer, got '{v}'")
        }),
    }
}

/// Print a usage error and exit `1` (REQ-TRS-CLI-009).
pub fn usage_exit(msg: &str) -> ! {
    eprintln!("Error: {msg}");
    eprintln!("Run `syscribe help <command>` for the command's options.");
    std::process::exit(1);
}

/// `r` or a usage-error exit.
pub fn or_exit<T>(r: Result<T, String>) -> T {
    r.unwrap_or_else(|m| usage_exit(&m))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn unknown_options_are_rejected_and_known_ones_accepted() {
        assert!(check_known_options("impact", &s(&["X", "--direction", "both", "--depth", "-1"])).is_ok());
        assert!(check_known_options("impact", &s(&["X", "--bogus"])).unwrap_err().contains("--bogus"));
        assert!(check_known_options("impact", &s(&["X", "--depth"])).unwrap_err().contains("expects a value"));
        assert!(check_known_options("validate", &s(&["--deny=W001", "--max-warnings=3"])).is_ok());
        assert!(check_known_options("n2", &s(&["--format=json"])).unwrap_err().contains("separate argument"));
        assert!(check_known_options("tree", &s(&["A::B"])).is_ok());
        assert!(check_known_options("tree", &s(&["--x"])).is_err());
        // Unchecked commands pass through.
        assert!(!is_checked("follow"));
        assert!(check_known_options("follow", &s(&["--anything"])).is_ok());
    }

    /// Drift guard: every `--option` a checked command's help page documents in its
    /// SYNOPSIS or OPTIONS section is accepted by the known-option table, so the
    /// check can never reject a documented invocation.
    #[test]
    fn every_documented_option_of_a_checked_command_is_known() {
        for (cmd, _) in crate::help::commands() {
            let Some(spec) = spec_for(cmd) else { continue };
            let page = crate::help::page(cmd).unwrap_or("");
            let mut documented = page.split("## DESCRIPTION").next().unwrap_or("").to_string();
            if let Some(opts) = page.split("## OPTIONS").nth(1) {
                documented.push_str(opts.split("\n## ").next().unwrap_or(""));
            }
            for tok in documented.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
                let Some(rest) = tok.strip_prefix("--") else { continue };
                if rest.is_empty() || !rest.starts_with(|c: char| c.is_ascii_lowercase()) {
                    continue;
                }
                assert!(
                    spec.iter().any(|(n, _)| *n == tok),
                    "`{cmd}` documents {tok} but cliargs::spec_for does not accept it"
                );
            }
        }
    }

    /// The converse drift guard: every option a checked command accepts is
    /// documented somewhere on its help page, so no flag is accepted-but-undocumented.
    #[test]
    fn every_known_option_is_documented() {
        for (cmd, _) in crate::help::commands() {
            let Some(spec) = spec_for(cmd) else { continue };
            let page = crate::help::page(cmd).unwrap_or("");
            for (opt, _) in spec {
                let documented = page
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
                    .any(|tok| tok == *opt);
                assert!(documented, "`{cmd}` accepts {opt} but prompts/help/{cmd}.md never mentions it");
            }
        }
    }

    #[test]
    fn enum_and_integer_values_are_strict() {
        let a = s(&["--format", "xml", "--depth", "abc"]);
        assert!(enum_value("n2", &a, "--format", &["text", "json"], "text").unwrap_err().contains("text, json"));
        assert_eq!(enum_value("n2", &s(&[]), "--format", &["text"], "text").unwrap(), "text");
        assert!(usize_value("n2", &a, "--depth").unwrap_err().contains("--depth"));
        assert_eq!(usize_value("n2", &s(&["--depth", "3"]), "--depth").unwrap(), Some(3));
        assert_eq!(usize_value("n2", &s(&[]), "--depth").unwrap(), None);
    }
}
