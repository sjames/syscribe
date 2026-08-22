---
type: Requirement
id: REQ-TRS-SYSMLV2-025
name: "A SysMLv2 enum def/enum maps to the native EnumerationDef/Enumeration schema — values, supertype, typedBy"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

An `enum def` shall be synthesized into a native `EnumerationDef` element carrying `supertype:`/
`values:` (each entry `{name: ...}`). A named `enum` usage shall be synthesized into a native
`Enumeration` element carrying `typedBy:`/`doc`.

## Rationale

`ElementType::EnumerationDef`/`ElementType::Enumeration` already existed in the native schema,
exercised by two real hand-authored files (`model/Enumerations/{ArmStatus,FlightMode}.md`), but were
unreachable from SysMLv2 ingestion — unlike other deferred constructs, `REQ-TRS-SYSMLV2-007`'s fixed
mapped-kind list didn't even name enum as explicitly out of scope; it was simply absent. `EnumDef`/
`EnumerationUsage` are already reachable from all three dispatch enums this module cares about
(`PackageBodyElement`, `PartDefBodyElement`, `PartUsageBodyElement`) — no parser-level ceiling blocks
the base mapping, the same posture as `REQ-TRS-SYSMLV2-024`'s Flow mapping.

## Scope

- `EnumDef`/`EnumerationUsage` are two distinct AST structs — two conversion functions,
  `convert_enum_def`/`convert_enum_usage`.
- `EnumDef` has a **dedicated**, not shared, body type: `EnumerationBody::{Semicolon, Brace {
  values: Vec<EnumeratedValue> }}`. `EnumeratedValue` carries **only** a `name` — any inline body or
  `= expr` initializer on a literal is parsed and discarded by the vendored parser itself, so
  `values:` entries can only ever be `{name: ...}`, never the spec's optional `value:`/`valueKind:`/
  `unit:`/`metadata:` sub-fields (§8.5.2) — a real upstream ceiling, not a Syscribe choice.
- **`EnumerationBody` carries no `Doc` variant at all** — a first for this mapping series (every
  prior increment's body type had at least some path to a `Doc` variant, even an indirect one). A
  `doc /* ... */` written inside an `enum def` is structurally unparseable into anything this crate
  retains; `convert_enum_def` makes no `.with_doc(...)` call at all, and `doc` stays `""` exactly as
  it would for any element with no doc member.
- `EnumerationUsage.body` is exactly `AttributeBody` — the same shared type `AttributeDef`/
  `AttributeUsage`/`ItemDef` already use — so the existing `attribute_body_doc` helper is reused
  unchanged for `Enumeration`'s doc lift, no new helper needed. `EnumerationUsage.type_name` →
  `typed_by:`, matching every other `*Usage.type_name` → `typed_by:` convention in this module.
- `Enumeration` (the usage kind) has **no documented frontmatter schema of its own** anywhere in
  `spec/markdown-sysml-format.md` — it's only listed in the usage-type summary table as "usage of an
  EnumerationDef". `EnumerationUsage.multiplicity`/`.is_end` therefore have no obvious native field to
  land in and stay unmapped, the same class of descope as Flow's `payload.multiplicity`.
- **Out of scope, noted but not implemented here**: neither of spec §11.7's two `EnumerationDef`
  MUST-report validation rules ("`supertype:` resolves to another `EnumerationDef`", "`values:`
  absent") exist in `validator.rs` today, for any origin, hand-authored included. This is a
  pre-existing, general validator-compliance gap unrelated to SysMLv2 mapping specifically — flagged
  in the ADR addendum, not fixed as a side effect of this requirement.

**Acceptance criteria:** a package-wrapped `enum def` with both `enum`-prefixed and bare literals
synthesizes a real `EnumerationDef` with `values:` in source order, `supertype:` set from its `:>`
clause, and `doc` empty even when the source has a `doc /* ... */` member; a literal with an `=
expr` initializer keeps only its `name:`; a named `enum` usage synthesizes a real `Enumeration` with
`typedBy:` and lifted `doc`; both kinds are reachable at package level and nested inside a `part
def`/`part` usage body.
