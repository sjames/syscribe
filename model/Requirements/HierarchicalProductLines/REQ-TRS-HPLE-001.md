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
