---
id: REQ-TRS-XREF-007
type: Requirement
name: "Tool shall report unresolved supertype, typedBy, subsets, redefines and satisfies references"
status: draft
reqDomain: software
verificationMethod: test
---

The format specification requires a reference error for every cross-reference that fails to
resolve (§11.5 step 4, §11.7). `verifies:` (`E102`), `derivedFrom:` (`E103`) and
`allocatedTo:` (`E503`) already report; the structural fields `supertype:`, `typedBy:`,
`subsets:`, `redefines:` and the traceability field `satisfies:` did not — a typo such as
`supertype: Nope::Missing` validated with no finding at all (GH #125).

### Behaviour

The tool **shall** raise one error per unresolved reference, with a field-specific code:

| Code | Field |
|---|---|
| `E110` | `supertype:` |
| `E111` | `typedBy:` — element-level, and on inline `features:` entries |
| `E112` | `subsets:` |
| `E113` | `redefines:` |
| `E114` | `satisfies:` |

A reference **shall** count as resolved — and raise nothing — when it resolves by any of the
§11.5 forms the rest of the tool already accepts:

- a stable id, an absolute qualified name, or a display name (the ordinary resolver);
- a name relative to the referencing element's enclosing-package scope chain (the lookup
  `typedBy:` already uses for SysML v2-ingested elements, REQ-TRS-SYSMLV2-016/017), so
  SysML v2-submodel, stdio-plugin- and annotation-synthesized elements resolve as elsewhere;
- a `./name` sibling reference (§5.2);
- an `imports:` or `aliases:` declaration of the element or any enclosing package (§5.3,
  §5.4, §11.5 step 3c);
- an inline (non-file) feature of a resolvable owner — `Owner::feature`, or a bare
  `feature` of the referencing element's owner — including one inherited through the
  owner's `supertype:`/`typedBy:` chain;
- the SysML v2 standard library: the built-in `ScalarValues`/`Base` packages (an unknown
  member stays `W043`'s concern), the curated ISQ/SI recognition (REQ-TRS-LIB-003), a
  reference into any standard-library package (`ISQ::…`, `Parts::Part::…`, `Links::…`)
  whose top-level name the model does not itself declare, and the bare names of the
  well-known library root types (`Link`, `Part`, `Real`, …);
- an element exported by a loaded `[repos]` peer (§14.4).

The `typedBy:` of a SysML v2-ingested `allocation` usage is exempt from `E111`: ingestion does
not map `allocation def` (outside REQ-TRS-SYSMLV2-007's fixed kind set), so it can never resolve.

In a `[repos]`-configured model an unresolved reference **shall** be reported as `E512`
("cross-repo … reference resolves neither locally nor in any loaded repo") instead of
`E110`–`E114`, as `verifies:`/`derivedFrom:`/`satisfies:`/`allocatedTo:` already are — a
reference is never reported twice.

`E110`–`E114` **shall** take the REQ-TRS-XREF-006 model-root-name hint, and **shall** be
suppressed in the `validate --config` lens like the other resolution codes (a target pruned
from the variant is reported as `E226`/`W019`, not as dangling).

**Source:** GH #125 (v0.40.1 documentation/spec consistency review). Refines
[[REQ-TRS-XREF-005]] (§11.5, §11.7).

**Acceptance criteria:**

- A model with `supertype: Nope::Missing`, `typedBy: Nope::Def` (usage and inline feature),
  `subsets: [Nope::S]`, `redefines: Nope::R` and `satisfies: [REQ-NOPE-001]` raises
  `E110`, `E111` (×2), `E112`, `E113`, `E114` and exits non-zero.
- A model whose references resolve by relative scope, `./` sibling, import, alias, inline
  feature, standard-library name or stable id raises none of `E110`–`E114` and exits zero.
- `supertype: <RootName>::Lib::Base` raises `E110` carrying the root-name hint `Lib::Base`.
- In a `[repos]` model, a supertype resolving in the peer raises nothing; one resolving
  nowhere raises `E512` and no `E110`.
