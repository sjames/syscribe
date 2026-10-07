---
type: Requirement
id: REQ-TRS-SYSMLV2-030
name: "Ingestion of a SysMLv2 file reports, once per file, an advisory count of the parsed-but-unmapped construct kinds it dropped"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - validation
---

Ingestion is parse-broad, map-narrow (`REQ-TRS-SYSMLV2-007`): constructs the parser accepts but
that have no native Syscribe element are not synthesized. That drop shall no longer be silent.
For each `.sysml`/`.kerml` file in a `sysmlSubmodel: true` subtree that contains at least one such
construct, ingestion shall raise exactly one advisory warning `W543` naming the file and listing,
per construct kind, how many were dropped (for example `calc def x2, constraint def x1, doc x3`).
A file with no unmapped constructs raises nothing.

The finding is advisory only: it never aborts ingestion, never changes the synthesized elements,
and is draft-friendly (a non-error warning that can be left as is or gated with `--deny W543`).

## Scope

- Counted at package-body level, recursing through nested `package` declarations and root-level
  members: `calc def`, `constraint def`/`constraint`, `use case def`/`use case`, `metadata def`/
  `metadata`, `doc` (package-level documentation), `library package`, `namespace`, `alias`,
  `occurrence`/`individual`, `dependency`, `actor`, `satisfy` and the KerML declaration forms.
- Not counted: `import` and `comment` (pure namespace plumbing), and members nested inside a
  mapped definition's body (for example a requirement's `frame`/constraint body); those remain
  invisible and are a follow-up.
