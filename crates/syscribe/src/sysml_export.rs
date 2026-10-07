//! `syscribe export-sysml` (`REQ-TRS-SYSMLV2-037`, `ADR-SYS-SYSMLV2-002`): one-way
//! export of the model as SysML v2 textual notation. Formatting lives in
//! `syscribe_model::sysmlv2::export`; this module only parses arguments and
//! writes the result. Never modifies the model.

use std::path::Path;

use syscribe_model::element::RawElement;
use syscribe_model::sysmlv2::export::export_sysml;

/// Run the command; returns the process exit code.
pub fn cmd_export_sysml(elements: &[RawElement], args: &[String]) -> i32 {
    let mut scope: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out" | "--output" => match it.next() {
                Some(v) => out = Some(v.as_str()),
                None => {
                    eprintln!("error: {a} needs a value (a file or directory)");
                    return 2;
                }
            },
            s if s.starts_with("--") => {
                eprintln!("error: unknown option '{s}' for export-sysml");
                return 2;
            }
            s => scope = Some(s),
        }
    }

    let export = match export_sysml(elements, scope) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    match out {
        None => print!("{}", export.text),
        Some(path) => {
            let p = Path::new(path);
            let as_dir = path.ends_with('/') || path.ends_with('\\') || p.is_dir();
            let result = if as_dir {
                std::fs::create_dir_all(p).and_then(|_| {
                    for (name, text) in &export.parts {
                        // The top-level name is a qname segment: keep file names portable.
                        let stem: String = name
                            .chars()
                            .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
                            .collect();
                        std::fs::write(p.join(format!("{stem}.sysml")), text)?;
                    }
                    Ok(())
                })
            } else {
                if let Some(parent) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::write(p, &export.text)
            };
            if let Err(e) = result {
                eprintln!("error: could not write '{path}': {e}");
                return 1;
            }
        }
    }
    eprintln!("export-sysml: {}", export.report.summary_line());
    0
}
