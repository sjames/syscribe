---
type: Package
name: BrakeByWireDemo
---

Standalone example model demonstrating **user-defined link types**
(`ADR-SYS-LINKTYPE-001`, spec §12.10): three project-specific relationships
declared in `.syscribe.toml` — `mitigates`, `partiallySatisfies` (a relaxed
variant of `satisfies`) and `conflictsWith` — authored under `links:` on a
small brake-by-wire model. See `../README.md` for what each file demonstrates
and how to run `link-types` and `follow` against it.
