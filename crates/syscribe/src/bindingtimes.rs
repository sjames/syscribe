//! `binding-times [--json]` — the effective binding-time vocabulary
//! (`REQ-TRS-BT-005`, `ADR-SYS-BT-001`). Read-only.

use syscribe_model::binding_time::BindingTimeVocab;

pub fn cmd_binding_times(vocab: &BindingTimeVocab, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(&vocab.to_json()).unwrap_or_default());
        return;
    }
    println!("# Binding times");
    println!();
    println!("| # | Stage | Binder | Description |");
    println!("|---|---|---|---|");
    for (i, s) in vocab.stages().iter().enumerate() {
        println!(
            "| {} | {} | {} | {} |",
            i + 1,
            s.name,
            s.binder.as_deref().unwrap_or("—"),
            s.description.as_deref().unwrap_or("—"),
        );
    }
    println!();
    if vocab.is_declared() && !vocab.defects().is_empty() {
        println!("Some `[bindingTimes]` entries in .syscribe.toml were ignored — run `validate` and fix the W750 findings.");
    } else if !vocab.is_declared() {
        println!("This is the built-in vocabulary. Refine it with a `[bindingTimes.<name>]` table in <model_root>/.syscribe.toml, e.g.:");
        println!();
        println!("    [bindingTimes.deploy]");
        println!("    description = \"installed on the target\"");
        println!("    binder = \"installer\"");
        println!("    after = \"compile\"");
        println!("    before = \"load\"");
    }
}
