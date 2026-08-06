---
type: Requirement
id: REQ-TRS-SYSMLV2-001
name: "A package declares itself a SysMLv2 submodel via sysmlSubmodel: true, excluding its subtree from native parsing"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A package `_index.md` shall accept an optional `sysmlSubmodel: true` field. When present, every
`.sysml`/`.kerml` file anywhere in that directory's subtree — however nested — is parsed as native
SysML v2/KerML textual notation instead of Markdown+YAML frontmatter; the package's own `_index.md`
remains a normal native element (so it still has a `name`, documentation body, and a place in the
containment tree). No nested `_index.md` is expected or processed anywhere inside the marked
subtree.

## Rationale

The directory *is* the namespace boundary (`CLAUDE.md`'s "Directory / Namespace Convention"), so
marking a subtree this way at the package level keeps the boundary exactly where model authors
already expect namespace boundaries to be. A plain boolean is enough — unlike `foreignFormat:
<alias>`, which exists to let multiple named third-party plugin engines coexist, there is exactly
one built-in native engine here, so an alias indirection buys nothing.

## Scope

- Non-`.sysml`/`.kerml` files inside the subtree (a README, an exported diagram, …) are ignored by
  ingestion — ignored, not an error.
- A stray `_index.md` nested inside the marked subtree is a warning ("ignored — inside a
  sysmlSubmodel subtree"), not processed as a package.
- Hand-authored `.md` element files may coexist alongside `.sysml`/`.kerml` files in the same
  directory; both are parsed normally and contribute to the same package's namespace — this is not
  forbidden.
- SysML v2 namespace derivation inside the subtree (`REQ-TRS-SYSMLV2-002`) is independent of
  directory layout below the marked root; nested subdirectories inside the subtree carry no
  namespace meaning of their own.
