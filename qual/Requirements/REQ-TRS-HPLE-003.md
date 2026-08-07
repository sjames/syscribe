---
id: REQ-TRS-HPLE-003
type: Requirement
name: A parameterBindings entry reaching into a consolidated subtree shall target an actually-open parameter
status: draft
reqDomain: software
verificationMethod: test
---

A `parameterBindings:` entry reaching into a `subConfigurations:`-consolidated subtree **shall**
target a parameter that is genuinely open at the point it is bound: the parameter's owning
`FeatureDef` **shall** actually be selected by the descendant `Configuration` that owns it, and
the parameter **shall** not already be bound by that descendant's own `Configuration` or by a
nearer tier's `parameterBindings:` on the path down to it. Targeting an unselected feature's
parameter, or double-binding a parameter a nearer tier already supplies, **shall** each be
reported as a validation error.

**Source:** `REQ-TRS-HPLE-003` (product model), `ADR-SYS-HPLE-001`.

**Acceptance criteria:** a cross-tier binding targeting a parameter the owning tier does not
select is reported as an error naming the owning tier; a cross-tier binding double-supplying a
parameter a nearer tier already bound is reported as an error naming that nearer tier; a binding
targeting a genuinely open, selected parameter validates cleanly.
