---
id: REQ-TRS-SYSMLV2-005
type: Requirement
name: A SysMLv2 variation point shall be able to target a Syscribe FeatureDef via a SyscribeFeature metadata annotation
status: draft
reqDomain: software
verificationMethod: test
---

A SysMLv2 `variation`/`variant` element **shall** be able to declare a `@SyscribeFeature {
featureId = '<FEAT-id>'; }` metadata annotation. The tool **shall** lift `featureId` into the
synthesized element's feature-model gate — the same field a native element's `appliesWhen:`
already populates — so the existing feature-model/SAT engine reasons about SysMLv2-authored
variation points identically to native ones, with **no changes to the solver**.

A `variation`/`variant` element with **no** `@SyscribeFeature` annotation **shall** be ingested
normally as a purely structural element — it simply does not participate in the feature-model
graph, exactly like a native element with no `appliesWhen:`.

An unresolvable `featureId` (no matching `FeatureDef`) **shall** be reported as a
dangling-reference finding — the same class already raised for any other unresolved feature
reference, not a new diagnostic.

**Source:** `REQ-TRS-SYSMLV2-005` (product model).

**Acceptance criteria:** a variant carrying `@SyscribeFeature { featureId = 'FEAT-X'; }` shows up
gated by that feature under `feature-check --deep`/`--config` projection exactly like a native
`appliesWhen: FEAT-X` element would (present under a configuration selecting `FEAT-X` true,
absent when false); a variant with no annotation is present under every configuration and raises
no feature-model finding; a `featureId` naming no real `FeatureDef` raises the existing
unresolved-feature-reference finding.
