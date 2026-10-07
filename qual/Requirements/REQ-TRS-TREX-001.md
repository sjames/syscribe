---
id: REQ-TRS-TREX-001
type: Requirement
name: The trace document lists every requirement with identity, derivation, satisfaction, verification and a computed coverage block, every reference by full qualified name
status: verified
reqDomain: software
verificationMethod: test
---

The traceability export **shall** emit one JSON object `{ "version": 1, "modelRoot", "config":
null | {...}, "sort", "requirements": [...], "summary": {...} }`. Each entry of `requirements`
**shall** carry, in this order: `qname` (full qualified name), `id` (stable id, or null),
`name`, `type` (`Requirement` or `RequirementDef`), `status`, `reqClass`, `reqDomain`, `file`
(model-root-relative path); `derivedFrom` and `derivedChildren` (lists of `{qname, id}`),
`breakdownAdr` (`{qname, id, status}` or null); `satisfiedBy` (list of `{qname, id, type,
domain}`), `verifiedBy` (list of `{qname, id, testLevel, status, verdict}` where `verdict` is
the ingested result — `pass`/`fail`/`unknown` — or null without a results sidecar),
`refinedBy` (list of `{qname, id}`); and `coverage`: `{ "leaf", "satisfied", "verified",
"integrationVerified" }` computed by the same rules `W300`, `W002` and `W305` apply (a leaf is
satisfied when at least one satisfier exists; verified when at least one `active` TestCase
verifies it; integration-verified when an `active` L3/L4/L5 TestCase verifies it; a `retired`
TestCase is listed but never counts). `summary` **shall** give the counts of requirements,
leaves, satisfied, verified and integration-verified. Every reference **shall** be the
element's full qualified name; a dangling reference **shall** appear as `{ "qname": "<text as
written>", "unresolved": true }`, never dropped. Native `Requirement`s and SysML
`RequirementDef`/`Requirement` are all included.

**Source:** `REQ-TRS-TREX-001` (product model).

**Acceptance criteria:** on a model with a parent requirement (L4 TestCase, refined by a use
case), two derived children (one satisfied by a PartDef; one verified by an active L2 TestCase
and a retired one) and a requirement whose `derivedFrom` dangles: (a) the document and each
entry carry the fields in the normative order; (b) every reference is a full qualified name
with its id, including `breakdownAdr`'s status and the satisfier's type/domain; (c) `coverage`
reads `{false, false, true, true}` for the parent, `{true, true, false, false}` for the
satisfied child, `{true, false, true, false}` for the verified child and `summary` is `{4, 3,
1, 2, 1}`; (d) the dangling parent is `{qname: "REQ-TX-999", unresolved: true}`; (e) verdicts
come from the sidecar and are null without one.
