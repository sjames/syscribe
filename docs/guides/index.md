# Practitioner Guide

`GUIDE · PRACTITIONER OVERVIEW`

A comprehensive how-to for applying syscribe across requirements, architecture, testing, automotive safety (ISO 26262), industrial safety (IEC 61508), and security (ISO/SAE 21434). Aimed at engineers experienced with traditional toolchains (DOORS, Polarion, Rhapsody, Cameo/MagicGrid, AUTOSAR).

## Contents

- [Introduction](introduction.md) — A new approach to system modeling; model-as-text; LLM integration; tool sceptic FAQ
- [Part I — Concepts and the Model-as-Code Shift](concepts.md) — What syscribe is; mental model shift; file layout; validation gate
- [Part II — Requirements](requirements.md) — Authoring requirements; hierarchy and derivation; status lifecycle; traceability links; integrity levels; multi-variant products
- [Part III — Architecture](architecture.md) — PartDefs; ADRs; diagrams; allocations; MagicGrid full MBSE methodology (B1–S4)
- [Part IV — Tests](tests.md) — Test pyramid; TestCase authoring; TestPlan; security test methods; coverage and gap analysis; ingesting test results
- [Part V — Safety: ISO 26262 (Automotive)](iso-26262.md) — Full workflow; HARA; SafetyGoal; FTA; FMEDA metrics; FMEA; safety case (GSN); AoU; confirmation measures
- [Part VI — Safety: IEC 61508 (Industrial / SIL)](iec-61508.md) — Differences from ISO 26262; risk graph parameters; SIL metric gating; tool qualification
- [Part VII — Security: ISO/SAE 21434](iso-21434.md) — TARA workflow; assets; damage scenarios; threat scenarios; attack trees; cybersecurity goals; security controls; vulnerability tracking
- [Part VIII — Integration and CI/CD](cicd.md) — Minimum CI gate; named profiles; safety-readiness dashboard; per-configuration gate; persisting evidence; external tool links; safe refactoring
- [Part IX — Software Architecture: Level 2 and Level 3](software-architecture.md) — Two-level distinction; subsystem decomposition; interface definitions; data flows; behavioral overview; ASIL allocation; WCET; error detection; L2→L3 traceability
- [Part X — Feature Modeling and the 150% Product Line Model](feature-modeling.md) — Feature model definition; configurations; appliesWhen; SAT integrity checks; variant diff; product line safety analysis; certification per variant
- [Appendix A — Traditional Tool Mapping](tool-mapping.md) — DOORS, Polarion, Cameo/MagicGrid, ASPICE, AUTOSAR
- [Appendix B — Validation Code Reference](../validation/rules.md) — Full list of E/W codes with triggers and descriptions
- [Appendix C — Quick Reference Card](quick-reference.md) — Element types by role; command cheat sheet; integrity level gating table
- [Appendix D — Scripting and Extensibility](scripting.md) — JSON output surface; full model graph export; test result ingestion; `.syscribe.toml`; diagram generation; code generation helpers; LLM integration; external tool integration
