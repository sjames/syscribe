---
type: Requirement
id: REQ-TRS-SYSMLV2-014
name: "A doc-comment directive syntax lifts the same fixed @Syscribe* fields onto interface def/port def/connection def, since these three body grammars have no MetadataAnnotation slot to carry the real @Name{...} form"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-008]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - safety
---

A SysMLv2 `interface def`, `port def`, or `connection def` shall be able to lift the same fixed
set of fields `REQ-TRS-SYSMLV2-008` establishes for `part def`/`part`
(`shortName:`/`implementedBy:`, and — where meaningful — `domain:`/`asilLevel:`/`silLevel:`/
`plLevel:`), via a **structured directive line inside the element's `doc /* ... */` comment**,
recognized alongside the ordinary doc-text lift (`REQ-TRS-SYSMLV2-009`):

```
doc /*
  @SyscribeShortName: power-if
  @SyscribeImplementedBy: aidl/interfaces/car/power/IPowerInterface.aidl
*/
```

| Directive | Field(s) lifted |
|---|---|
| `@SyscribeShortName: <value>` | `shortName:` |
| `@SyscribeImplementedBy: <path>` | `implementedBy:` (single path) |
| `@SyscribeDomain: <value>` | `domain:` |
| `@SyscribeIntegrity: <key>=<value>[, <key>=<value>...]` (keys `asil`/`sil`/`pl`) | `asilLevel:`/`silLevel:`/`plLevel:` |

A recognized directive line is stripped from the text that lands in the synthesized element's
`doc:` field — it is metadata, not documentation prose, exactly as a real `@SyscribeDomain
{ value = '...'; }` annotation on a `part def`/`part` never appears in that element's `doc:`
either. An unrecognized `@Something: ...` line, or a directive name that doesn't match one of the
four above, is left in the doc text untouched (it's prose, not a defect). A later directive line
for the same field overrides an earlier one in the same doc comment, matching the
last-annotation-wins behavior `REQ-TRS-SYSMLV2-008` already establishes for repeated
`@Syscribe*` annotations.

## Rationale

`REQ-TRS-SYSMLV2-008`'s `@Name { field = value; }` mechanism depends on a real, structurally
parsed `MetadataAnnotation` AST node — and confirmed by direct inspection of the vendored
`sysml-v2-parser` source (both the pinned 0.53.0 and the latest 0.54.0), `InterfaceDefBodyElement`,
`PortDefBodyElement`, and `ConnectionDefBodyElement` carry **no such variant at all** (issue #100).
This is a genuine upstream grammar gap, not a missing `ingest.rs` dispatch arm: `@SyscribeImplementedBy
{ path = '...'; }` inside an `interface def { }` is a hard parse error (`W541`), confirmed
empirically, because the grammar production for these three body kinds has nowhere to put it.
`sysml-v2-parser` is a plain crates.io version dependency (`ADR-SYS-SYSMLV2-001` sub-decision 2),
not a vendored/forked local copy this repository can extend, so the real `@Name{...}` form cannot
be widened to these three element kinds without an upstream parser change.

The pain this leaves unaddressed — `implementedBy:` un-set, so no `W023` disk-check and no
`sbom`/`export-reqif` pickup — is real (documented directly in issue #100's own Motivation
section, from a live `.aidl`-mirrored `interface def` conversion). A doc-comment-embedded
directive is a **working, testable, real substitute** that delivers the same downstream effect
(`implementedBy:`/`shortName:`/etc. populated on the synthesized element exactly like a
hand-authored one, driving `W023` and everything else unchanged) without reversing
`ADR-SYS-SYSMLV2-001` sub-decision 2's deliberate choice not to vendor the parser. It is
explicitly **not** presented as equivalent syntax to `@Name { field = value; }` — it is a
different, doc-comment-based spelling, used only where the real annotation form is grammatically
unreachable.

## Scope

- Scoped to exactly the three element kinds `part def`/`part` don't already cover and issue #100
  named: `interface def`, `port def`, `connection def`. Not extended to their usage counterparts
  (`interface`/`port`/`connection` usages) in this phase — out of scope per issue #100's own
  acceptance criteria, which only names the three `def` forms.
- The directive line must appear inside a `doc /* ... */` block already reachable by
  `REQ-TRS-SYSMLV2-009`'s doc-lift dispatch for that element kind — no new comment-scanning
  machinery, no change to what counts as a `doc` block.
- If real `MetadataAnnotation` coverage for these three body kinds is ever added upstream (see
  issue #100's own suggested path 1), this doc-comment mechanism is not superseded or removed —
  it simply becomes a second, always-available spelling alongside the real annotation form, the
  same way `id`/qname are both valid cross-reference targets today.
- Malformed directive content (e.g. `@SyscribeIntegrity: sil=999`) is not specially rejected
  here — the resulting `silLevel: 999` lands on the synthesized frontmatter exactly like a
  hand-authored one and is caught downstream by the existing range check, mirroring
  `REQ-TRS-SYSMLV2-008`'s own posture for the real annotation form.

**Acceptance criteria:** an `interface def`/`port def`/`connection def` whose `doc /* ... */`
comment contains `@SyscribeImplementedBy: <path>` validates with `implementedBy: [<path>]` on the
synthesized element and triggers `W023` under the same conditions a hand-authored element with
that value would; the directive line does not appear in the synthesized `doc:` text; an element
whose `doc` comment contains no recognized directive behaves exactly as `REQ-TRS-SYSMLV2-009`
already specifies (no regression).
