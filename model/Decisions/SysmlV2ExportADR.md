---
type: ADR
id: ADR-SYS-SYSMLV2-002
name: "One-way export of native Syscribe elements to SysML v2 textual notation; ingestion stays read-only"
status: accepted
tags:
  - sysmlv2
  - interop
  - export
---

## Context

`ADR-SYS-SYSMLV2-001` decided that SysML v2/KerML submodels are ingested read-only: `.sysml`/`.kerml`
text is parsed in-process by `sysml-v2-parser` into ordinary `RawElement`s, and Syscribe has **no
writer** — nothing is ever serialized back into a `.sysml` file inside a submodel, and the
`sysml-v2-parser` crate itself has no serializer. That decision stands.

A different need has since appeared: teams that author in Syscribe's Markdown+YAML format want to
hand their structural model to SysML v2 tooling (spec42, the OMG pilot implementation, a review
tool, a colleague who reads textual SysML). The model already has a precedent for one-way,
export-only interchange: `export-reqif` (§21) writes ReqIF from native requirements and never reads
it back. The same posture fits SysML v2.

## Decision

Add a **one-way, export-only** writer, `syscribe -m <root> export-sysml [<package-qname>] [--out
<file|dir>]`, plus a read-only MCP tool `export_sysml`. It is a new, separate code path
(`syscribe_model::sysmlv2::export`) that reads the loaded element graph and renders SysML v2 textual
notation. It supersedes nothing in `ADR-SYS-SYSMLV2-001`.

Five sub-decisions, each with a rejected alternative:

1. **Ingestion stays read-only; the writer is export-only.** The writer never touches a
   `sysmlSubmodel: true` package's `.sysml` files, never edits the model, and `export-sysml` never
   becomes a sync or round-trip-editing channel. *Rejected:* a bidirectional bridge that writes
   edits back into `.sysml` files — it would need source-span-preserving rewriting, merge semantics
   for multi-file packages, and a stable mapping for every construct the ingester drops
   (`W543`); none of that is justified by the interchange need.
2. **Hand-rolled textual writer in `syscribe-model`, no new dependency.** The supported subset is
   small and regular (declaration header, `doc`, nested body), so a purpose-built renderer is
   cheaper and more predictable than adding a serializer crate that does not exist for
   `sysml-v2-parser`. The writer lives in `crates/syscribe-model/src/sysmlv2/export.rs` so the CLI
   and MCP front ends share it, exactly like `sysmlv2::report`.
3. **Directory/qname tree becomes nested `package`s; defs and usages nest by qname.** An element
   `A::B::C` is rendered inside `package A { package B { ... } }`; an element whose qname parent is
   a definition or usage (not a package) is rendered inside that parent's body. Missing
   intermediate segments become implicit packages. Every identity segment is emitted as a SysML
   identifier, quoted with single quotes when it is not a basic name or is a reserved word
   (`'REQ-TRS-001'`); this is what lets id-identified native elements keep their stable id as the
   name segment, and so lets qnames round-trip.
4. **A deliberately narrow v1 mapping, with explicit loss accounting.** Supported: `Package`;
   `PartDef`/`Part`; `PortDef`/`Port`; `AttributeDef`/`Attribute`; `ConnectionDef`/`Connection`;
   `InterfaceDef`/`Interface`; `ItemDef`/`Item`; `RequirementDef` and native `Requirement` (both as
   `requirement def`, native ones carrying their body as `doc /* */`); and header-plus-doc forms of
   `ActionDef`/`Action`, `StateDef`/`State`, `ConstraintDef`/`Constraint`,
   `CalculationDef`/`Calculation`. A definition's `supertype:` becomes `:>`, a usage's `typedBy:`
   becomes `:`, `multiplicity:` becomes `[n]`, an element's `satisfies:` becomes `satisfy <target>;`
   in its body. Every other element type (TestCase, ADR, PlanningItem, FeatureDef, safety/security
   records, ...) is **not** exported: the writer emits a `// skipped: <qname> (<type>)` line comment
   in the position it would have occupied and the report counts exported and skipped elements per
   type — the same "say what was dropped" spirit as ingestion's `W543`. Behavioural bodies
   (transitions, action steps, constraint expressions) are *not* emitted in v1.
   *Rejected:* silently omitting unsupported elements (hides loss), or failing the export on the
   first one (makes a mixed model unexportable).
5. **Verified by parse-back, not by a second source of truth.** The test strategy is to export a
   model, parse the output with the existing ingestion pipeline (`sysml-v2-parser`), and assert that
   the supported kinds and qnames come back (native `Requirement` returns as `RequirementDef`, the
   one documented non-identity mapping). Output is deterministic (sorted by qname, no timestamps),
   so exporting twice yields byte-identical text.

## Rationale

The ReqIF export proves Syscribe users want one-way interchange without taking on a sync problem.
Keeping the writer export-only preserves the invariant that Syscribe's Markdown model is the source
of truth and that a `.sysml` file inside a submodel is hand-authored input, never generated output.
Round-trip parity on the supported subset, checked by the real parser, bounds the risk of emitting
text no SysML tool accepts.

## Consequences

- A new module, CLI subcommand and read-only MCP tool; no change to ingestion, validation or any
  existing finding code.
- The exported text is a lossy projection (no behaviour, no traceability beyond `satisfy`, no
  Syscribe-only types). It is not intended to be re-imported as the same Syscribe elements.
- Re-importing exported text into a `sysmlSubmodel: true` package is possible and is exactly what
  the parse-back tests do, but it yields `RequirementDef`s, not native `Requirement`s.

## Addendum: export gap closure (REQ-TRS-SYSMLV2-049..052)

Sub-decision 4's "behavioural bodies are not emitted" narrows: usages now export `:>`/`:>>` and
multiplicity, inline attribute units export as literal-with-unit, `satisfies:` on non-part elements
becomes a real package-level `satisfy R by X;` and `verifies:` on requirement elements a real
`verify T;`, and constraint/calc elements export their parameters and expression text (the same
opaque text ingestion stores; unprintable placeholders become comments). Everything else in the
decision, including loss accounting and parse-back verification, is unchanged. `action`/`state`
bodies still export as header-plus-doc only.

## Addendum: behaviour export (REQ-TRS-SYSMLV2-056..058)

The last clause of the previous addendum ("`action`/`state` bodies still export as header-plus-doc
only") is superseded. `ActionDef`/`Action` and `StateDef`/`State` bodies now export in the exact
statement forms REQ-TRS-SYSMLV2-018/-019 ingestion reads: the writer is constrained to the grammar the
reader accepts, and each construct is covered by a parse-back test that re-ingests the text and compares
the resulting `subActions:`/`controlNodes:`/`successionConnections:` or `subStates:`/`transitions:`/
`entryAction:`/`doAction:`/`exitAction:` with the originals. Anything the reader would not reproduce
identically is written as a `//` comment rather than approximated: an unknown `kind:`, an
`AssignmentAction`/`IfAction`/`LoopAction`/`TerminateAction` whose name is not the one ingestion will
re-synthesize for that position (`assign_1`, `if_1`, `while_1`, …), and any guard, condition or effect
text that is a placeholder (`<conditional expression>`) or fails the expression parse check.
Successions and control nodes are emitted flat at the owning element's body, matching how ingestion
flattens them. Exported behaviour is still a one-way projection: nothing is read back as the same
Syscribe elements except by the explicit parse-back path.

## Addendum: named steps and dangling successions (REQ-TRS-SYSMLV2-061..063)

A control entry (`if`/`while`/`loop`/`for`/`assign`/`terminate`) whose name is not the synthesized
positional one now exports as `action <name> { <stmt> }`, the form ingestion reads back with that name
(REQ-TRS-SYSMLV2-060), instead of being commented out. A succession is exported only when each endpoint
that names an entry of the same body was itself exported; otherwise it is commented with the missing
endpoint named, so exported text never references a step the file does not contain. Compound units are
written as unit expressions (`[N*m]`), not quoted names (REQ-TRS-SYSMLV2-065).
