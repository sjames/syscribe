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
- `--run <id>` ranks a retained run by id (GH #471). A records-like ingest (`records`, `robot`, `cucumber`, a results plugin) made with `--run <id> --config <CONF>` keeps its slim records, measurements included, in the run's own file under `.syscribe/runs/` (`REQ-TRS-RUNHIST-001`), superseding the records with the same key (all of them under `--replace`); `trade-study --run <id>` reads each compared Configuration's records from that run in place of the sidecar. A Configuration the run holds nothing for has no evidence, never the sidecar's. An unknown id is refused listing the retained runs, as is a run with no records for any compared Configuration, saying why: a verdict-only run (re-ingest it with `--config`), a run whose records the cap dropped (`[results] keep_run_records`: raise it and re-ingest, or read an archived copy with `--results`), or a run whose file was deleted (restore it). `--run` and a `--results` directory both cover every compared Configuration and are refused together; `--results <Configuration>=<file>` overrides the run for that one. Neither flag applies to `TradeStudy` elements.
- A cell's `evidence` names the run (`origin: run`, `run`, `count`, `ingestedAtUnix`, `sources`) and lists `stale` reasons when a TestCase or requirement changed since the records were ingested (the check `W091` makes for the sidecar); this is information, not a knock-out.
- Not covered: a file outside the model (`--results`) is not sealed by it, and its records are stamped against the model as it is now, so a change since the run is not detected there (`W091` cannot apply).
