# Syscribe Format Specification — Reference Index

Markdown-SysML: a SysMLv2-compatible systems modeling format. Each `.md` file is one
model element; YAML frontmatter declares the type and metadata; the Markdown body is
the element's documentation. Directory path encodes namespace ownership.

## Sections

| Command | Contents |
|---|---|
| `syscribe spec types` | Element type inventory, SysML keyword mapping, native type schemas |
| `syscribe spec fields` | Complete frontmatter field reference (all ~90 fields) |
| `syscribe spec namespace` | Directory conventions, cross-reference syntax, multiplicity rules |
| `syscribe spec validation` | All validation rule codes (E001–E941, W001–W905; PLE E2xx + projection E226/E227, W010–W022) |
| `syscribe spec traceability` | Traceability rules R-001–R-007 |
| `syscribe spec safety` | Safety/security analysis elements: HARA, TARA, FTA, FMEA |

**Variability / product lines (§9, opt-in).** `FeatureDef` + `Configuration` + `appliesWhen:` model a product line (the 150% model). Tools: `matrix` (Requirement × Configuration coverage), `feature-check` / `feature-check --deep` (holistic + SAT-backed analysis), `configure` (assisted configuration), and the `--config` projection lens (`validate`/`list`/`export --config`, `validate --all-configs`, `diff`). Dormant — and unchanged — when no `FeatureDef` is present. See `spec fields` and `spec validation`.

## Core rules (memorise these)

1. Model root directory → root namespace. Each subdirectory → one `package`.
2. Qualified name = `::` path from root, e.g. `UAV::Avionics::FlightController`.
3. `_index.md` represents the containing directory's package; it has no own QName segment.
4. Cross-references use qualified names or stable IDs (`REQ-*`, `TC-*`, `ADR-*`, etc.).
5. OSLC link direction: the downstream/derived artifact always holds the reference field.
6. Reverse indices (`verifiedBy`, `derivedChildren`, `satisfiedBy`) are computed by the
   parser — **never written to disk**.

## ID pattern quick reference

| Element type | Pattern | Example |
|---|---|---|
| `Requirement` | `REQ(-[A-Z0-9]{2,12})+-[0-9]{3}` | `REQ-UAV-FC-001` |
| `TestCase` | `TC(-[A-Z0-9]{2,12})+-[0-9]{3}` | `TC-SCHED-001` |
| `ADR` | `ADR(-[A-Z0-9]{2,12})+-[0-9]{3}` | `ADR-SW-SCHED-001` |
| `HazardousEvent` | `HE-*` | `HE-BRAKE-001` |
| `SafetyGoal` | `SG-*` | `SG-BRAKE-001` |
| `DamageScenario` | `DS-*` | `DS-001` |
| `ThreatScenario` | `TS-*` | `TS-001` |
| `CybersecurityGoal` | `CSG-*` | `CSG-001` |
| `SecurityControl` | `SC-*` | `SC-001` |
| `VulnerabilityReport` | `VR-*` | `VR-001` |
| `FaultTree` | `FT-*` | `FT-BRAKE-001` |
| `FaultTreeGate` | `FTG-*` | `FTG-001` |
| `FaultTreeEvent` | `FTE-*` | `FTE-001` |
| `FMEASheet` | `FMEA-*` | `FMEA-BRAKE-001` |
| `TARASheet` | `TARA-*` | `TARA-001` |
| `Configuration` (PLE) | `CONF-*` | `CONF-HEX-001` |

## Status values

| Element | Allowed statuses |
|---|---|
| `Requirement` | `draft` · `review` · `approved` · `implemented` · `verified` |
| `TestCase` | `draft` · `active` · `retired` |
| `ADR` | `proposed` · `accepted` · `deprecated` · `superseded` |
| Safety/security (HE, SG, DS, TS, CSG, SC, VR, FT, FMEA) | `draft` · `review` · `approved` |
| Other elements | no mandated status field |
