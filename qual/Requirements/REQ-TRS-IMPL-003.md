---
id: REQ-TRS-IMPL-003
type: Requirement
name: Package-registry references in implementedBy shall be classified as external, exactly as the SBOM command recognises them, and shall not raise W023
status: draft
reqDomain: software
verificationMethod: test
---

`implementedBy:` (and every other field resolved with the `sourceFile` location semantics) **shall** classify a **package-registry reference** of the form `<registry>:<package>@<version>[#<path>]` as a remote/external location, not as a local path, where `<registry>` is one of the registry prefixes the `sbom` command maps to a Package URL ecosystem: `crates.io`, `npm`, `pypi`, `maven`, `nuget`, `github`. Both `<package>` and `<version>` **shall** be non-empty. The recognised prefixes and grammar **shall** come from one shared definition used by both the location classifier and `sbom`, so the two can never disagree.

Consequently a non-`draft` `Part`/`PartDef`/`Interface`/`InterfaceDef` whose `implementedBy:` lists only such references **shall not** raise `W023` (remote references are accepted as external, §12.8).

Classification **shall not** change for anything else: a genuine missing local path **shall** still raise `W023`; `repo:`/`model:` prefixed paths and `file://` URIs **shall** remain local; a Windows-drive-like value (`C:\src\lib.rs`, `C:/src`) **shall** remain local (a single letter is not a registry); an unknown prefix (`foo:bar@1`) and a registry prefix without `@<version>` (`crates.io:tokio`) **shall** remain local paths.

**Source:** GH #134 — `implementedBy: [crates.io:tokio@1.38.0, npm:lodash@4.17.21]` produced correct SBOM purls but `W023` "does not exist on disk", because `classify_source` treated only `scheme://` as remote.

**Acceptance criteria:** a non-draft `PartDef` with `implementedBy: [crates.io:tokio@1.38.0, npm:lodash@4.17.21, github:org/repo@v1]` raises no `W023`; a sibling with a missing local path still raises exactly one `W023`; `sbom` still emits `pkg:cargo/tokio@1.38.0` and `pkg:npm/lodash@4.17.21`; unit tests pin `C:\…`, `repo:`, `model:`, unknown-prefix and missing-version values as local.
