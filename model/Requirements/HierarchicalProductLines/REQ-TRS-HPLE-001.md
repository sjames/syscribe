---
type: Requirement
id: REQ-TRS-HPLE-001
name: "subConfigurations: names the peer Configurations a Configuration consolidates, each of which must resolve and be internally valid"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-HPLE-000]
breakdownAdr: Decisions::HierarchicalProductLineADR
tags:
  - variability
  - multi-repo
---

A `Configuration` shall accept an optional `subConfigurations: <Configuration qname-or-id | list>`
field naming one or more other `Configuration` elements — reachable locally or, in the common case,
via the existing `repoImports:` mounting of a lower-tier product-line repo into the local qname
namespace. Each named `Configuration` shall resolve to a real `Configuration` element, and that
element shall itself be internally valid (SAT-satisfiable and clean per `feature-check --deep`/the
equivalent of `validate --config` run against wherever it actually lives).

## Rationale

Reusing ordinary qname resolution — rather than inventing a new cross-repo addressing scheme —
follows directly from `repoImports:` already mounting a peer's elements into the local namespace and
already resolving cross-repo references by searching the local model first, then each loaded repo
in declaration order (§14.4). Requiring the named `Configuration` to be independently valid before
it can be consolidated is what makes "consolidation of *configured* lower level models" a real
guarantee rather than a name that happens to resolve to something broken.

**Correction found during implementation scoping** (the ADR's Decision 1 originally overstated
this): existing multi-repo composition's qname reach is real but shallow — `LoadedRepo` indexes a
peer into two flat `HashSet<String>`s (qnames, stable ids) purely for existence-checking, exactly
as `ADR-SYS-PLUGIN-001` documents ("it never builds real graph nodes for peer content"). That is
sufficient for confirming a `subConfigurations:` entry's qname *exists*, but not for reading the
peer `Configuration`'s actual selected features or their parameter metadata (`isFixed`, `range`,
`isRequired`) — which `REQ-TRS-HPLE-002`'s cross-tier `parameterBindings:` validation genuinely
needs. Resolving *that* requires actually loading and parsing the referenced peer's relevant
elements (at minimum, the named `Configuration` and the `FeatureDef`s it selects), not merely
checking that a name exists — the same shape of upgrade `ADR-SYS-PLUGIN-001` made for foreign-format
content, applied here to a peer repo's own native elements instead. This requirement's "SAT-clean"
check is exactly the forcing function for that loading step: confirming a peer `Configuration` is
internally valid already requires reading its real feature/parameter structure, not just its name.

## Scope

- `subConfigurations:` is valid at any tier. It is naturally empty or absent at a leaf tier (one
  with no lower-tier product lines to consolidate) — this is not a separately validated or
  separately declared concept; "leaf" falls out structurally exactly as it already does elsewhere
  in this codebase.
- An entry that doesn't resolve to any element is a dangling-reference finding, following the
  established error-code family for unresolved multi-repo cross-references (`E512`-adjacent).
- An entry that resolves to something that isn't a `Configuration` is a validation error.
- An entry that resolves to a real `Configuration` which is itself SAT-void or otherwise invalid is
  a validation error — checked empirically (not assumed symmetric with any other resolution check)
  against however this codebase's existing per-`Configuration` validity check is actually invoked,
  per the lesson learned building the SysMLv2 and PlanningItem features: verify the check applies
  before relying on it, don't assume.
- Whether the consolidating `Configuration` supplies any parameter values into the consolidated
  subtree is `REQ-TRS-HPLE-002`, not this requirement.
