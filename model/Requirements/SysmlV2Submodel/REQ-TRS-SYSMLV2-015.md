---
type: Requirement
id: REQ-TRS-SYSMLV2-015
name: "A connect endpoint's genuinely two-segment chain that falls back to head-only (the tail isn't a locally-redeclared feature) raises W542, identifying the dropped segment, instead of silently truncating"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-013]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - connectivity
---

A `connect` endpoint whose genuinely two-segment dotted chain (`a.fooProvider`, no further `.`)
falls back to `REQ-TRS-SYSMLV2-010`'s head-only qualification — because the tail isn't a feature
`REQ-TRS-SYSMLV2-013`'s local redeclaration lookahead can verify — shall raise a `W542` finding
identifying the endpoint text, the dropped tail segment, and the head-only edge it was truncated
to. A chain that resolves via `REQ-TRS-SYSMLV2-013`'s redeclaration lookahead, a bare (undotted)
endpoint, and a three-or-more-segment chain (`REQ-TRS-SYSMLV2-013`'s own separate, deliberately
unwarned fallback) shall all raise no `W542`.

## Rationale

`REQ-TRS-SYSMLV2-013`'s local, AST-lookahead-only resolution deliberately covers only the case
where a `.sysml` author explicitly redeclares the referenced feature on the usage itself — the
overwhelmingly common case (a feature *inherited* from the head's type, never redeclared on the
usage) still silently truncates, exactly as `REQ-TRS-SYSMLV2-010` originally specified. Filed as
issue #104 after using `REQ-TRS-SYSMLV2-013` against a real, multi-file CarOS architecture
submodel: every ordinary `part carDisplayService : Services::CarDisplayService;` composition —
where `CarDisplayService` (the type) declares the interface, never redeclared on the usage — hit
the silent-truncation path, with nothing in `validate` output to indicate a `.fooProvider`/
`.compositorControl`-shaped segment had been dropped rather than resolved. Full resolution through
an un-redeclared, type-inherited feature was considered (issue #104's proposed approach 1) and
rejected as this requirement's scope — it needs the full, cross-file element graph to look up the
head's *type* definition, which doesn't exist yet at the ingest-time, single-file AST-processing
point `REQ-TRS-SYSMLV2-013`'s resolution already runs at (the same reason a full resolver was
rejected for `REQ-TRS-SYSMLV2-013` itself). A warning identifying the truncation, per issue #104's
proposed approach 2, is the narrower, always-answerable fix: the same AST already in hand at
`qualify_connection_end`'s call site already knows exactly when this happens.

## Scope

- Fires only for a genuinely two-segment chain (`head.tail`, no further `.`) that falls back to
  head-only — the exact same condition `REQ-TRS-SYSMLV2-013`'s own lookahead already isolates. A
  three-or-more-segment chain stays silent, unaffected — `REQ-TRS-SYSMLV2-013`'s own, separately
  documented and deliberate scope limit, not this requirement's concern.
- Raised via `RawElement.derive_findings` (the same mechanism `W540`/`W541` already use), attached
  to the *owning* `part def`/`part`/`variant part` usage element — the same element
  `REQ-TRS-SYSMLV2-010`'s `connections:` lift writes onto — not onto the nested `Connection`
  element `REQ-TRS-SYSMLV2-012` separately synthesizes for the same `connection` usage.
- One `W542` per truncated endpoint. A binary `connect a to b;` clause can raise zero, one, or two;
  the n-ary `connect (a, b, c)` form can raise one per truncated end.
- Does not change the qualified qname a truncated endpoint resolves to — `Top::a`, exactly as
  `REQ-TRS-SYSMLV2-010`/`-013` already specify. This requirement only adds visibility, never
  changes the structural output.
- `W542` continues the dedicated SysMLv2 code range `REQ-TRS-SYSMLV2-006` established
  (`W540`/`W541`), distinct from the WASM-plugin family.

**Acceptance criteria:** `connect a.fooProvider to b.fooClient;` where neither `a` nor `b`
redeclares the referenced feature raises exactly two `W542` findings (one per endpoint), each
identifying its own dropped segment, alongside the unchanged `Top::a -> Top::b` edge; the same
clause where both `a` and `b` redeclare the feature (`REQ-TRS-SYSMLV2-013`'s already-working case)
raises no `W542`; a bare `connect a to b;` raises no `W542`; a three-segment chain
(`a.fooProvider.deep`) raises no `W542`.
