---
type: Requirement
id: REQ-TRS-SYSMLV2-031
name: "A syscribe sysml command reports each SysMLv2 submodel's parsed files, ingested element counts, unmapped-construct counts and W540-W543 findings"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-030]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - cli
---

`syscribe -m <root> sysml [--json]` shall inspect the SysMLv2 submodels of the model without
modifying it. For every package declaring `sysmlSubmodel: true` it shall report the `.sysml`/`.kerml`
files parsed (and whether each parsed), the number of ingested elements per element kind, the
parsed-but-unmapped construct counts per kind (the data behind `W543`) and the `W540`-`W543`
findings raised for that submodel. `--json` shall emit the same data as a machine-readable document.
A model with no `sysmlSubmodel: true` package shall yield an explanatory message (or an empty
`submodels` array under `--json`) and exit status 0.

The data gathering shall live in `syscribe-model` (`sysmlv2::report`) so other front ends reuse it
(`REQ-TRS-SYSMLV2-032`); the CLI only formats it.
