---
id: REQ-TRS-SPEC-001
type: Requirement
name: Discoverable Spec Reference Completeness
title: The syscribe spec reference shall document every element type and frontmatter field, including the safety/security analysis set
status: draft
reqDomain: software
verificationMethod: test
---

The tool embeds a discoverable specification reference, surfaced by `syscribe spec <section>` (`types`, `fields`, `safety`, `validation`, `namespace`, `traceability`, `toc`). Because an LLM/author can only use what this reference exposes, it **shall** stay complete with the implementation:

- `syscribe spec types` **shall** list every `ElementType` an author may write — including all Tier-2/Tier-4 safety and security types: `HazardousEvent`, `SafetyGoal`, `FaultTree`, `FaultTreeGate`, `FaultTreeEvent`, `FMEASheet`, **`FMEAEntry`**, `TARASheet`, `DamageScenario`, `ThreatScenario`, `CybersecurityGoal`, `SecurityControl`, `VulnerabilityReport`.
- `syscribe spec fields` **shall** document every frontmatter field the parser accepts — including the **safety analysis** fields (`severity`, `exposure`, `controllability`, `operationalSituation`, `consequence`, `freqExposure`, `avoidance`, `demandRate`, `safeState`, `ftti`, `hazardousEvents`, `topEvent`, `missionTime`, `gateType`, `inputs`, `eventKind`, `failureRate`, `probability`, `entries`, `failureMode`, `effect`, `cause`, `fmeaSeverity`, `occurrence`, `detection`, `rpn`, `recommendedAction`) and the **security analysis** fields (`damageTable`, `threatTable`, `goalTable`, `controlTable`, `damageSeverity`, `impactCategories`, `attackFeasibility`, `attackVector`, `damageScenarios`, `calLevel`, `securityProperty`, `threatScenarios`, `controlType`, `implementsGoals`, `cvssScore`, `cveId`, `affectedElements`, `mitigatedBy`).
- `syscribe spec safety` **shall** document each safety/security type's own fields, including `cveId`, `safeState`, and `ftti`.

The canonical, authoritative list of types and fields is the implementation (`crates/syscribe-model/src/element.rs`: the `ElementType` enum and `RawFrontmatter` struct). The discoverable reference is **non-conformant** if it omits a type or field the parser accepts.

**Source:** GH #27 — the safety/security analysis fields were implemented but absent from `syscribe spec fields`, so an author could not discover them.

**Acceptance criteria:** `syscribe spec types` contains `FMEAEntry`; `syscribe spec fields` contains every safety and security analysis field listed above; `syscribe spec safety` contains `cveId`, `safeState`, and `ftti`. A field accepted by the parser but absent from `syscribe spec fields` is a documentation defect.
