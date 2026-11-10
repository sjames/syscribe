---
type: Requirement
id: REQ-TRS-MOEMEAS-001
name: "A MoE can take its value from recorded measurements, and recorded failures knock a variant out"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
  - magicgrid
---

The MagicGrid `trade-study` shall be able to score a Configuration on the measurements recorded for it (`ingest-results --format records --config <CONF>`) as well as on its predicted value, and a recorded failure shall knock the Configuration out of the ranking whatever its design values predict (GH #467).

- A MoE names the measurement it reads with `mg_moe_measurement`, how several recorded values combine with `mg_moe_aggregate` (`mean`, `max`, `p95`, `last`; default `last`, the latest by finish time) and the unit its bounds are in with `mg_moe_unit` (absent, the unit of the latest recorded value). Each recorded value is converted to that unit; one that cannot be is reported, never read as a bare number. `mg_moe_source: measurement` scores the recorded value; the default `expression` keeps scoring the prediction.
- A cell is a knock-out (`FAIL`, never `WINNER`) when a record that carries the measurement has a `fail` verdict, or a recorded measurement breaks its own `limit`, whichever value is scored, in addition to the existing threshold rule. The reasons are listed under the table and in `violationReasons`.
- Predicted and measured values are shown side by side; a cell with nothing recorded is `n/a`, and a Configuration with no evaluable cell is never the winner. Only records ingested for that Configuration (`--config`) count, not unscoped ones.
- `--json` carries per cell `predicted`, `measured` (`value`, `unit`, `aggregate`, `recordCount`, the contributing `records` and the `evidence` they came from) and `violationReasons`. A malformed `mg_moe_source`, `mg_moe_measurement`, `mg_moe_aggregate` or `mg_moe_unit` is `MG033`.
- `--results <dir>` reads `<Configuration id>.json` for each compared Configuration, and `--results <Configuration>=<file>` one file, instead of the sidecar: a run kept outside the model (for instance a CI artefact) can be ranked, and an earlier run compared, without ingesting it. A file is the records format of `ingest-results --format records`, parsed, validated and joined to the model's TestCases exactly as at ingest (a record naming no TestCase is ignored and counted). It replaces the sidecar for the Configurations it covers and is never merged with it; a Configuration the directory has no file for has no evidence, and one not covered reads the sidecar. Nothing is written. Each cell's `evidence` names the file and its `blake3:` hash (or the sidecar ingest). A malformed argument or file is refused, exit 1.
- Not covered: the run history (`ingest-results --run`) keeps verdicts, not measurements, so a run is chosen by pointing `--results` at its files. A file outside the model is not sealed by it, and its records are stamped against the model as it is now, so a change to a TestCase or requirement since the run is not detected (`W091` cannot apply).
