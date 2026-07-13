---
type: Requirement
id: REQ-TRS-BL-010
name: "Baseline output locations are configurable via [baselines] in .syscribe.toml"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - config
---

Where `baseline create` writes the `Baseline` element and the manifest shall be
configurable, so a project can adopt its own release-evidence layout once rather than on
every invocation.

- A `[baselines]` table in `<model-root>/.syscribe.toml` shall accept:
  - `element_dir` — the directory (relative to the **model root**, or absolute) that
    receives the `Baseline` element file. Default: `Baselines`. The element must remain
    **within the model tree** so the walker indexes it and drift-checking finds it; a
    configured `element_dir` that escapes the model root shall be rejected with a clear
    error and nothing written.
  - `manifest_dir` — the directory (relative to the **git root**, or absolute) that receives
    the JSON manifest. Default: `baselines`. The manifest is a non-model artifact and may
    live anywhere.
- The absence of the table (or of either key) preserves the current defaults, so existing
  models are unaffected.
- The manifest location shall stay self-recording: the element's `seal.manifest` shall store
  the manifest path such that `baseline verify` / drift-checking resolve it regardless of the
  configured directory (git-root-relative when the manifest is under the git root, else
  absolute).
- A CLI override is out of scope for v1; configuration is the single mechanism, consistent
  with the other `.syscribe.toml` sections (`[ids]`, `[links]`, `[repos]`, …).
