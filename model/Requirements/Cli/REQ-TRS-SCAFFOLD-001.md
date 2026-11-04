---
type: Requirement
id: REQ-TRS-SCAFFOLD-001
name: "new scaffolds a feature, a Configuration or a parameter that validates the first time"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe new feature|configuration|parameter` and the MCP tool `scaffold` shall create what an author would otherwise get wrong (GH #427):

- **feature** — a `FeatureDef` under a package or a group feature, with a free `FEAT-*` id derived from the name (extended with the names above it when taken), `mandatory:` and the `groupKind:` of its children; a name that is not a basic name, a malformed or taken id, an existing file and a leaf parent are refused;
- **configuration** — a `Configuration` with a valid, unused `CONF-*` id, `status: draft`, the base's `featureModel:`, `derivedFrom:` and a `bindingStage:` that is a real stage not earlier than the base's; `--select` seeds `features:`;
- **parameter** — appended to the feature's `parameters:` with type, unit, range, enumeration, default and a binding window whose stages exist and are in order; a default outside its range or enumeration, a duplicate name and a malformed range are refused.

Each is previewed through the guarded-write engine (`--dry-run`; the MCP tool defaults to preview) and committed only when it adds no error. Created files add no finding to the example models, existing lines of an extended file are unchanged, and the report has the published `scaffold` schema.
