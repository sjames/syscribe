---
type: Requirement
id: REQ-TRS-PLUGIN-002
name: "[plugins.<alias>] config and the plugin's JSON envelope merge into the graph as first-class elements"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLUGIN-000]
breakdownAdr: Decisions::WasmPluginsADR
tags:
  - plugins
---

`.syscribe.toml` shall accept a `[plugins.<alias>]` table (`wasm` path, optional `timeout_ms` /
`memory_max_bytes`) naming the compiled plugin for a given `foreignFormat:` alias. The plugin's
`parse` export shall return a JSON envelope — a list of elements (`qname`, `type`, optional `id`
/ `name`, `doc`, and arbitrary extra frontmatter keys) plus a list of parse diagnostics — which
Syscribe merges into the element graph as ordinary `RawElement`s, qname-prefixed under the owning
package, resolvable by `derivedFrom:`/`satisfies:`/`verifies:`/`Allocation` and every other
cross-reference kind exactly like a hand-authored element.

## Rationale

`RawElement`/`Resolver`/`validate_with_config` are already origin-agnostic (proven by the
existing FMEA/TARA row-explosion passes in `walker.rs`, which synthesize sibling elements the same
way). Reusing that same injection point means zero special-casing anywhere in the resolver or
validator for plugin-originated elements.

## Scope

- An element whose `type:` doesn't name a recognised element type is dropped with `W534`, not a
  hard failure of the whole run.
- An element whose extra frontmatter fails to map onto `RawFrontmatter` is dropped with `W533`.
- A `type: Requirement` element with a valid `REQ-*`-shaped `id:` opts into the full native
  Requirement traceability rule set (breakdown-ADR, leaf-assignment, …) exactly as a hand-authored
  one would — no restriction against plugins emitting native-Requirement-typed elements.
