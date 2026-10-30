//! `residual --config <Configuration> [--json]` — the variability a Configuration leaves
//! open, grouped by the stage it can next be bound at and by that stage's binder
//! (`REQ-TRS-BT-016`, `ADR-SYS-BT-001`). Read-only.

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::residual::{residual, Residual};

/// Index of the Configuration `conf` names (id or qualified name).
pub fn find_configuration(elements: &[RawElement], conf: &str) -> Option<usize> {
    elements.iter().position(|e| {
        e.frontmatter.element_type.as_ref() == Some(&ElementType::Configuration)
            && (e.frontmatter.id.as_deref() == Some(conf) || e.qualified_name == conf)
    })
}

/// The residual of `conf`, or `None` when no such Configuration exists.
pub fn residual_for(elements: &[RawElement], conf: &str) -> Option<Residual> {
    find_configuration(elements, conf).map(|i| residual(elements, i))
}

fn print_text(r: &Residual) {
    let stage = r.stage.map_or("unstaged".to_string(), |s| format!("bindingStage {}", s.name()));
    println!("# Residual variability: {} ({stage})", r.configuration);
    println!();
    if r.groups.is_empty() {
        println!("Nothing is left open: every feature and parameter with a binding window is decided.");
    }
    for g in &r.groups {
        if g.closed {
            println!("## Closed — no later stage can bind these");
        } else {
            let st = g.stage.map_or("?".to_string(), |s| s.name());
            match &g.binder {
                Some(b) => println!("## Next bindable at {st} (binder: {b})"),
                None => println!("## Next bindable at {st}"),
            }
        }
        println!();
        println!("| Kind | Item | Window | Required |");
        println!("|---|---|---|---|");
        for i in &g.items {
            let req = match (i.required, i.has_default) {
                (true, true) => "yes (has default)",
                (true, false) => "yes",
                _ => "no",
            };
            println!("| {} | {} | {} | {} |", i.kind, i.key, i.window.describe(), req);
        }
        println!();
    }
    if !r.constraints.is_empty() {
        println!("## parameterConstraints");
        println!();
        println!("| Constraint | State | Still open |");
        println!("|---|---|---|");
        for c in &r.constraints {
            let open = if c.open.is_empty() { "—".to_string() } else { c.open.join(", ") };
            println!("| {} — `{}` | {} | {} |", c.id, c.expression, c.state.name(), open);
        }
    }
}

pub fn cmd_residual(elements: &[RawElement], conf: &str, json: bool) -> i32 {
    let Some(r) = residual_for(elements, conf) else {
        eprintln!("Configuration not found: {conf}");
        return 1;
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&r.to_json()).unwrap_or_default());
    } else {
        print_text(&r);
    }
    0
}
