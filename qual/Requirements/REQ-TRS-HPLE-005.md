---
id: REQ-TRS-HPLE-005
type: Requirement
name: A lower-tier product-line model shall carry zero awareness of, or reference to, anything above it
status: draft
reqDomain: software
verificationMethod: test
---

A product-line model that may be consolidated by a higher tier **shall** require no authoring
change, no field, and no foreknowledge of that consolidation to function as a lower tier. A
`FeatureDef` parameter that needs an externally-supplied value **shall** declare this exactly as
it already would in an ordinary, single-model `Configuration` (`isRequired: true`, no `default`),
with no field naming, or capable of naming, whoever eventually supplies that value. In particular,
`bindTo:` (the pre-existing component→system parameter-propagation mechanism, scoped to one model)
**shall not** be usable to reach across a `subConfigurations:` repo boundary — a `bindTo:` target
**shall** continue to resolve purely within its own model.

**Source:** `REQ-TRS-HPLE-005` (product model), `ADR-SYS-HPLE-001`.

**Acceptance criteria:** a lower tier's `bindTo:` naming a dotted path that exists only in a
separate, higher tier's model never causes that higher tier's own validation to raise a
propagation-range finding tied to it, and the lower tier's own independent validation is
unaffected by a sibling higher-tier model's bindings existing at all; the same `bindTo:` mechanism
does correctly raise a propagation-range finding when the match is genuinely local to one model
(positive control, confirming this is scoping, not simply the mechanism failing to work).
