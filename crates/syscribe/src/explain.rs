//! `explain <code>` — what a validation finding means and the command that fixes it
//! (GH #439). The CLI face of the MCP `explain_finding` tool; reads the embedded catalogue,
//! so it needs no model.

use crate::help;

/// `E106`, `w042`, `I010` — a validation code.
pub fn looks_like_code(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 4 && matches!(b[0].to_ascii_uppercase(), b'E' | b'W' | b'I') && b[1..].iter().all(u8::is_ascii_digit)
}

/// Commands named in backticks in `text` (`` `scaffold-gherkin --fix` `` → `scaffold-gherkin`), in order.
fn commands_in(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for span in text.split('`').skip(1).step_by(2) {
        let mut words = span.split_whitespace();
        let mut first = words.next().unwrap_or("");
        if first == "syscribe" {
            first = words.find(|w| !w.starts_with('-') && !w.ends_with('/')).unwrap_or("");
        }
        if help::is_command(first) && !out.iter().any(|c| c == first) {
            out.push(first.to_string());
        }
    }
    out
}

/// Print the explanation; returns the exit code (`1` for an unknown code).
pub fn cmd_explain(code: &str) -> i32 {
    let code = code.to_ascii_uppercase();
    match crate::mcp::explain_code(&code) {
        Some(text) => {
            let sev = match code.as_bytes()[0] {
                b'E' => "error",
                b'W' => "warning",
                _ => "info",
            };
            println!("{code} ({sev})\n");
            println!("{text}");
            let cmds = commands_in(&text);
            if !cmds.is_empty() {
                println!("\nRelated commands:");
                for c in &cmds {
                    let summary = help::commands().find(|(n, _)| n == c).map(|(_, s)| s).unwrap_or("");
                    println!("  syscribe help {c:<18} {summary}");
                }
            }
            0
        }
        None => {
            eprintln!("Unknown finding code: {code}. `syscribe spec validation` lists every code.");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_recognised() {
        assert!(looks_like_code("E106") && looks_like_code("w042") && looks_like_code("I010"));
        assert!(!looks_like_code("claim") && !looks_like_code("E1") && !looks_like_code("E10x"));
    }

    #[test]
    fn commands_are_pulled_from_backticks() {
        assert_eq!(commands_in("run `scaffold-gherkin --fix` or `syscribe set X`, not `nonsense`"), ["scaffold-gherkin", "set"]);
    }
}
