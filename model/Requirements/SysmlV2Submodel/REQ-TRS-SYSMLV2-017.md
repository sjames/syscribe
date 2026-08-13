---
type: Requirement
id: REQ-TRS-SYSMLV2-017
name: "A cross-package, SysMLv2-authored typedBy:/supertype: reference resolves through the referencing element's enclosing-package scope chain for W007's usage tracking and for graph.rs's TypedBy edge, not only for W600's suppression check"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-016]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - validation
---

`W007`'s "defined but never used as a supertype or type" usage tracking, and `graph.rs`'s
`TypedBy` edge, shall resolve `typedBy:`/`supertype:` by searching outward through the
referencing element's own enclosing-package scope chain — the same `Resolver::resolve_scoped_ref`
widening `REQ-TRS-SYSMLV2-016` already made for `W600`'s suppression check — so a `*Def` used
exclusively via a cross-package, SysMLv2-authored reference (e.g. `Documented`, referenced only as
`part x : Services::Documented;` from a different package `System`) counts as real usage and
produces a real graph edge, not only an already-fully-qualified reference from the model root.

## Rationale

`REQ-TRS-SYSMLV2-016`'s own Scope bullet named this exact gap and deliberately left it open:
`W007`'s usage-tracking lookup and `graph.rs`'s `TypedBy` edge share `W600`'s original root
cause — a package-relative `typedBy:`/`supertype:` value only resolves via `Resolver::resolve_ref`
(exact qname/stable-ID/display-name match) when the relative text happens to already equal the
target's real full qname, which fails the moment `.sysml` content spans more than one package.
Issue #107 confirmed the resulting false-positive rate directly against the same live
CarOS/`sabaton-caros` conversion `REQ-TRS-SYSMLV2-016` used: 35 of 36 `W007` warnings in that
submodel were false positives, all sharing this one cause (only the top-of-hierarchy `CarOS`
element was a genuine unused type). `graph.rs`'s `TypedBy` edge shares the identical bug — before
this requirement, `idx.get(s)` did an exact-index lookup with no fallback at all (not even the
plain `resolve_ref`'s id/display-name matching), so `connectivity`/`n2`/`impact` silently showed no
edge whatsoever for a package-relative `typedBy:`.

## Scope

- Widens the top-level `supertype:`/`typedBy:` lookup and the nested `typedBy:` lookup (inside
  `features:`/`connections:`/`flowConnections:`/`bindingConnections:`/`successionConnections:`/
  `performs:`, including the nested `ports:` sub-key) in `W007`'s usage-tracking pass to
  `resolve_scoped_ref(elements, <referencing element's own qname>, r)`, in place of the plain
  `resolve_ref`. `exhibitsStates:` is unaffected — it is never synthesized by SysMLv2 ingestion, so
  it is always written fully qualified from the model root by this format's own hand-authored
  convention, and stays on `resolve_ref`.
- Widens `graph.rs`'s `TypedBy` edge construction the same way, replacing its previous bare
  `idx.get(s)` exact-qname lookup with `resolve_scoped_ref`. This is a strict widening of what
  `connectivity`/`n2`/`impact` traverse: an edge that previously silently failed to appear for a
  package-relative reference now appears, correctly; an edge that already resolved (same-package,
  fully-qualified, stable-ID, or display-name) is unaffected.
- The `Supertype` edge in `graph.rs` and the `mutate::guard`'s dangling-reference check (`EREF`,
  gating MCP guarded writes) are **not** widened here — each was named as a candidate by
  `REQ-TRS-SYSMLV2-016`'s Scope bullet but is out of this requirement's filed scope (issue #107's
  own acceptance criteria are `W007`-scoped, with `graph.rs`'s `TypedBy` edge as an explicit
  in-scope extra; the issue's own "Limitations" section allows narrowing to `W007` alone if the
  edge widening's blast radius warrants it — assessed here and judged safe to include, since it
  only ever adds an edge that reflects a real, previously-missed reference). Widening the
  `Supertype` edge or the `mutate::guard` dangling check is left for a future issue.
- The already-correct same-package usage-tracking case, and the "genuinely unused" case (nothing
  anywhere references the `*Def` as `supertype:`/`typedBy:`, cross-package or otherwise), are
  unaffected — no regression.

**Acceptance criteria:** a `*Def` referenced exclusively via a cross-package, SysMLv2-authored
`typedBy:`/`supertype:` no longer raises `W007`, and the reference is a real, `connectivity`-visible
`TypedBy` graph edge; a genuinely unused `*Def` still raises `W007`; same-package resolution is
unaffected.
