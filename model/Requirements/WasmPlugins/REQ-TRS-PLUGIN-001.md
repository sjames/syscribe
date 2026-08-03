---
type: Requirement
id: REQ-TRS-PLUGIN-001
name: "A package declares itself foreign via foreignFormat:, excluding its subtree from native parsing"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLUGIN-000]
breakdownAdr: Decisions::WasmPluginsADR
tags:
  - plugins
---

A package `_index.md` shall accept an optional `foreignFormat: <alias>` field. When present,
every other file under that directory is excluded from native `.md`/YAML-frontmatter parsing;
the package's `_index.md` itself remains a normal native element (so it still has a `name`,
documentation body, and a place in the containment tree).

## Rationale

The directory *is* the namespace boundary (`CLAUDE.md`'s "Directory / Namespace Convention"), so
marking a subtree foreign at the package level — rather than per-file — keeps the boundary exactly
where model authors already expect namespace boundaries to be, and gives the plugin a natural,
collision-resistant qname root to synthesize elements under (`<package-qname>::<element-qname>`).

## Scope

- `foreignFormat:`'s value must resolve to a `[plugins.<alias>]` entry in `.syscribe.toml`
  (`REQ-TRS-PLUGIN-002`); an unresolved alias is `E532`.
- Nested packages inside a foreign directory are not treated as separate native packages — the
  whole subtree is plugin-owned.
