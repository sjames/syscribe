---
type: Package
name: Requirements
---

Normative requirements for the Syscribe tool. Grouped by functional area:

| Prefix | Area |
|---|---|
| `REQ-TRS-PARSE` | File discovery and frontmatter extraction (§11.1–11.2) |
| `REQ-TRS-QNAME` | Qualified name derivation (§11.3) |
| `REQ-TRS-XREF` | Cross-reference resolution (§11.5–11.6) |
| `REQ-TRS-ELEM` | Element type handling (§2, §11.4) |
| `REQ-TRS-ID` | ID scheme validation (§8.11, §8.12, §8.15) |
| `REQ-TRS-VAL` | Validation rule enforcement (§11.12) |
| `REQ-TRS-TRACE` | Traceability rules (§12) |
| `REQ-TRS-IMPL` | Architecture↔code implementation linkage: `implementedBy` field, `W023` path-exists, discoverability (GH #13) |
| `REQ-TRS-CONF` | Configuration / `appliesWhen` validation (§9, §11.12) |
| `REQ-TRS-VAR` | Variability: opt-in dormancy, `appliesWhen` (element- and transitive package-level), matrix, per-config coverage (§9) |
| `REQ-TRS-PARAM` | FeatureDef parameter binding validation, range syntax, and `parameterConstraints` evaluation (`E221`/`W025`, §9.7; GH #14) |
| `REQ-TRS-FM` | Explicit feature-model validation command (`feature-check`, §9); feature-model schema fields including the `mandatory:` membership field (`ADR-FM-003`) |
| `REQ-TRS-FMA` | Solver-backed feature-model analysis: `feature-check --deep`, cores, `configure`, variant count, diagnoses, DRAT proofs (`ADR-FM-002`) |
| `REQ-TRS-PROJ` | Configuration projection: the `--config` lens, per-variant validation, escaping refs, global guarantee, family checks (`ADR-PROJ-001`) |
| `REQ-TRS-PLAN` | Native `TestPlan` element: schema, config binding, computed membership, demonstrated goals, the `testplan` command and the `--plan` lens (GH #38) |
| `REQ-TRS-DISC` | Product-line feature discoverability: `features`, `feature`, `matrix --features`, `list --feature`, `why-active`, orphan-feature `W024` |
| `REQ-TRS-TAG` | Generic tag filtering (orthogonal to variability) |
| `REQ-TRS-CFLD` | User-defined `custom_fields:` frontmatter: shape validation (`W041`), the `--where` query predicate, and read-only CLI/web rendering (GH #39) |
| `REQ-TRS-OUT` | Output and reporting |
| `REQ-TRS-CLI` | CLI interface |
