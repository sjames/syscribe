---
id: REQ-TRS-CLI-007
type: Requirement
name: Tool shall report its version via --version, -V, and a version subcommand
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** report its own version so users and CI can confirm which build is
installed. The version is the `syscribe` crate's package version (`CARGO_PKG_VERSION`).

The tool **shall**:

- print its version when invoked as `syscribe --version`, `syscribe -V`, **or**
  `syscribe version`;
- emit the version to **stdout** in the form `syscribe <semver>` (the binary name, a
  space, then the semantic version — e.g. `syscribe 0.25.0`), followed by a newline;
- **exit 0** for all three spellings;
- work **without** a model directory — version reporting **shall** be handled before any
  model resolution, so it succeeds from any working directory regardless of whether a
  model or `.syscribe.toml` is present;
- be discoverable: `--version`/`-V` and the `version` command **shall** appear in the
  top-level help/command index.
