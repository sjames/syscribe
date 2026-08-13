---
type: Requirement
id: REQ-TRS-SYSMLV2-016
name: "REQ-TRS-VAL-017's W600 typedBy: documentation fallback resolves a SysMLv2-authored, package-relative typedBy: reference, not only one that already equals the target's full model-root qname"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-VAL-017]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - validation
---

`REQ-TRS-VAL-017`'s `W600` suppression (a `Part` usage whose `typedBy:` target itself carries
documentation) shall resolve `typedBy:` by searching outward through the referencing element's own
enclosing-package scope chain, not only by an exact, already-fully-qualified match — so a
SysMLv2-authored `part x : Services::Documented;` written inside `package System { ... }`
(producing the literal, package-relative `typedBy: "Services::Documented"`, not the target's real
full qname `SysML2::Services::Documented`) suppresses `W600` on `x` exactly like the same reference
written inside `Documented`'s own package would.

## Rationale

`ingest.rs` performs no resolution of its own at parse time: a `typedBy:`/`supertype:` value on a
SysMLv2-synthesized element is the literal text a `.sysml` author wrote, which SysML v2's own
namespace-scoping rules let be relative to the referencing element's enclosing package — not
necessarily the from-model-root qualified form this format's hand-authored convention otherwise
always uses. `REQ-TRS-VAL-017`'s original implementation used the plain `Resolver::resolve_ref`
(exact qname / stable-ID / display-name match only), which only happens to succeed when the
relative reference text is coincidentally identical to the target's real full qname — true in a
flat, single-package `.sysml` file, false the moment `.sysml` content spans more than one package,
which is the ordinary shape of any real, multi-file architecture submodel (confirmed against a
live CarOS/`sabaton-caros` conversion: `W600` fired 15/15 times post-fix, none suppressed, because
every reference crossed a package boundary).

## Scope

- Adds `Resolver::resolve_scoped_ref(elements, from_qname, r)`: tries `r` prefixed by each
  enclosing scope of `from_qname` in turn, innermost first, down to the model root, then falls
  back to the existing `resolve_ref` (id / exact qname / display name) if no scoped candidate
  matches. A strict widening — every reference `resolve_ref` already resolved keeps resolving
  identically; only a previously-unresolved relative reference gains a new, correct match.
- Wired into `REQ-TRS-VAL-017`'s `W600` suppression check only. The same underlying gap — a
  package-relative `typedBy:`/`supertype:` value not resolving via the plain, unscoped
  `resolve_ref` — also affects the "defined but never used as a supertype or type" `W007` check,
  the dangling-`typedBy:` check, and `graph.rs`'s `TypedBy` edge (so `connectivity`/`n2`/`impact`
  don't currently traverse a package-relative `typedBy:` either). Widening those call sites to the
  same scoped resolution is explicitly **out of scope** here — each has its own blast radius worth
  assessing on its own, and this requirement's job is closing the specific, filed defect
  (`REQ-TRS-VAL-017`'s suppression not firing where it should), not auditing every `typedBy:`
  consumer in one pass.
  **Correction (`REQ-TRS-SYSMLV2-017`, issue #107):** the `W007` and `graph.rs`'s `TypedBy` edge
  gaps this bullet scoped out are now closed — `REQ-TRS-SYSMLV2-017` widened both to
  `resolve_scoped_ref`, confirmed against the same live CarOS/`sabaton-caros` submodel this
  requirement's own Rationale cites (35 of 36 `W007` warnings there were this exact false
  positive). The `mutate::guard` dangling-`typedBy:` check (`EREF`) remains open, tracked
  separately.
- The already-correct same-package suppression case, and the "target has no doc"/"typedBy: doesn't
  resolve at all" still-fires cases, are unaffected — no regression.
- `resolve_scoped_ref` is a general `Resolver` capability (documented and tested as such), not a
  one-off inline fix — reusable by a future widening of the other call sites named above without
  needing its own logic rewritten.

**Acceptance criteria:** `part x : Services::Documented;` (a package-relative reference, written
inside a different package than `Documented`'s own, `Documented` carrying a non-empty `doc`)
suppresses `W600` on `x`, matching the already-correct same-package case; the same reference to an
equally-undocumented target, or one that doesn't resolve via any scope, still raises `W600`
exactly as before.
