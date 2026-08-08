---
type: Requirement
id: REQ-TRS-SYSMLV2-008
name: "A fixed set of @Syscribe* metadata annotations lift domain, integrity level, shortName, and implementedBy onto a SysMLv2 part def/part"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - traceability
---

A SysMLv2 `part def`/`part` shall be able to declare a fixed, named set of `@Syscribe*` metadata
annotations that lift onto the synthesized element's frontmatter fields exactly as if hand-authored
— the same shape `@SyscribeFeature { featureId = '...'; }` → `appliesWhen:` already establishes
(`REQ-TRS-SYSMLV2-005`):

| Annotation | Field(s) lifted | Existing validation reused |
|---|---|---|
| `@SyscribeDomain { value = '...'; }` | `domain:` | E303, E315, E313 |
| `@SyscribeIntegrity { asil = '...'; }` | `asilLevel:` | E010, E841–E843, W701, W808 |
| `@SyscribeIntegrity { sil = ...; }` | `silLevel:` | E009, W006 |
| `@SyscribeIntegrity { pl = '...'; }` | `plLevel:` | E837 |
| `@SyscribeShortName { value = '...'; }` | `shortName:` | — (display only) |
| `@SyscribeImplementedBy { path = '...'; }` | `implementedBy:` | W023 |

Once lifted, the field is indistinguishable from a hand-authored one to every downstream
consumer — the existing validation rules in the "reused" column apply unchanged, with no
SysMLv2-origin-aware branching anywhere in the validator.

## Rationale

`domain:` and `asilLevel:`/`silLevel:`/`plLevel:` are exactly the fields a safety-relevant
architecture needs most (E313 domain-compatibility checks on `satisfies:`, E841–E843 integrity
propagation, the `safety-case` report), and today they have no expressible form in real `.sysml`
text — a `sysmlSubmodel: true` package can carry structural elements but silently loses the
information that matters most for safety-critical use. `@Name { field = value; }` is already a
real, structurally parseable `MetadataAnnotation` AST node for any annotation name (confirmed by
the existing `syscribe_feature_id` lift), so this is missing mapper coverage, not a parser
limitation.

## Scope

- Fixed, named set — `@SyscribeDomain`, `@SyscribeIntegrity`, `@SyscribeShortName`,
  `@SyscribeImplementedBy` — matching `ADR-SYS-SYSMLV2-001` sub-decision 3's narrow-mapping
  philosophy. No generic "any `@Syscribe*` annotation → `custom_fields:`" passthrough; that was
  explicitly rejected for `@SyscribeFeature` and the same reasoning applies here.
- `@SyscribeIntegrity` may carry any of its three keys (`asil`/`sil`/`pl`); more than one present
  on the same annotation is not specially rejected here — the corresponding frontmatter fields are
  simply both written, and the pre-existing `silLevel`/`asilLevel` mutual-exclusion check (`W006`)
  fires on them exactly as it would for a hand-authored element carrying both. No new validation
  code is introduced by this requirement.
- Scoped to `part def`/`part` (including a `variant part` usage, which shares the same body
  shape) — the element kinds `domain:`/integrity levels/`shortName:`/`implementedBy:` are
  meaningful on. Not extended to `Requirement`/`Attribute`/`Port`/etc. in this phase.
- Still read-only, one-way ingestion, same as every other `sysmlSubmodel: true` mapping — no
  writer/round-trip.
- Does not address `doc:` extraction (tracked separately, `docs/model-guide/sysmlv2-submodel.md`
  §6) — an element carrying these annotations still gets `W600` until that lands independently.

**Acceptance criteria:** a `part def`/`part` with `@SyscribeDomain { value = 'software'; }` and
`@SyscribeIntegrity { asil = 'B'; }` validates with `domain: software`/`asilLevel: B` on the
synthesized element, exactly as a hand-authored equivalent would, and triggers the same
E313/E841–E843 findings a hand-authored element with those values would; a `part def`/`part` with
none of these annotations behaves exactly as it does today (no regression).
