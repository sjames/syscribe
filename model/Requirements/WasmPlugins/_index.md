---
type: Package
name: WasmPlugins
---

Requirements for foreign-format model ingestion via sandboxed WASM plugins: letting a directory
inside the model tree be authored in a different text-based modeling methodology and still
participate fully in Syscribe's traceability graph.

All requirements derive from `REQ-TRS-PLUGIN-000` and are governed by `ADR-SYS-PLUGIN-001`
(`Decisions::WasmPluginsADR`): marking a package foreign via `foreignFormat:`
(`REQ-TRS-PLUGIN-001`), the `[plugins.<alias>]` config and envelope merge into the graph as
first-class elements (`REQ-TRS-PLUGIN-002`), sandboxed execution with scoped host-function
filesystem access and no network (`REQ-TRS-PLUGIN-003`), an origin-agnostic duplicate-qname
diagnostic surfaced by this work (`REQ-TRS-PLUGIN-004`), graceful degradation on plugin failure
(`REQ-TRS-PLUGIN-005`), a `plugins run --dry-run` debug command (`REQ-TRS-PLUGIN-006`), and an
on-disk content-hash cache so an unchanged plugin/subtree pair skips re-invocation
(`REQ-TRS-PLUGIN-007`). Path-escape hardening beyond the cases already covered and write-protection
for plugin-owned elements in the mutate routes remain tracked as follow-on scope (see
`docs/model-guide/wasm-plugins.md`).
