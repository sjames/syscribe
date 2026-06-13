---
id: REQ-TRS-OUT-019
type: Requirement
name: Tool shall generate a CycloneDX / SPDX SBOM from implementedBy links (sbom command)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only **`sbom`** command (§18, GH #66) that generates a
Software Bill of Materials from the `implementedBy:` links on `Part`/`PartDef` elements (and,
with `--include-tests`, from `TestCase.sourceFile:`).

**`syscribe sbom [--format cyclonedx|spdx] [--config <CONF>] [--output <file>] [--include-tests] [--scope <qname>]`**

- An `implementedBy:` value matching `<registry>:<package>@<version>[#path]` (registries
  `crates.io` · `npm` · `pypi` · `maven` · `nuget` · `github`) **shall** be emitted as an
  external **package** component with a PURL (`pkg:<eco>/<package>@<version>`); any other
  value (incl. a `repo:` link) **shall** be a local **file** component.
- A locally-derived component **shall** carry references back to the requirement(s) the
  implementing part `satisfies:` — CycloneDX `externalReferences` (`syscribe://<REQ-id>`) and
  SPDX `GENERATED_FROM` relationships.
- **`--format cyclonedx`** (default) **shall** emit CycloneDX 1.6 JSON
  (`bomFormat`/`specVersion: "1.6"`, a `serialNumber` urn:uuid, `metadata` with tool +
  timestamp, and `components`). **`--format spdx`** **shall** emit SPDX 2.3 JSON
  (`spdxVersion: "SPDX-2.3"`, `packages`, and `DESCRIBES`/`CONTAINS`/`GENERATED_FROM`
  relationships).
- `--scope` restricts to a namespace subtree; `--config` projects to a `Configuration`
  first; `--output` writes to a file instead of stdout; `--include-tests` adds TestCase
  source components.

**Source:** §18 (SBOM Generation), GH #66. Read-only; no new element types or rules.

**Acceptance criteria:**

- A part `implementedBy:` a `crates.io:`/`npm:` URI yields a package component with the
  correct PURL; a local path yields a file component.
- A local component links back to the satisfied requirement (CycloneDX `externalReferences`
  / SPDX `GENERATED_FROM`).
- `--format cyclonedx` is valid CycloneDX 1.6 JSON; `--format spdx` is valid SPDX 2.3 JSON.
- `--include-tests` adds TestCase source components; `--output` writes a file.
