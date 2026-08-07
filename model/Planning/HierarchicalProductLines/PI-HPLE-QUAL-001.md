---
type: PlanningItem
id: PI-HPLE-QUAL-001
name: "Qual test cases TC-TRS-HPLE-* wired into the qual harness"
status: done
itemType: task
parent: PI-HPLE-001
evidence:
  - path: "repo:qual/tests/tc/TC-TRS-HPLE-001.sh"
  - path: "repo:qual/tests/tc/TC-TRS-HPLE-002.sh"
  - path: "repo:qual/tests/tc/TC-TRS-HPLE-003.sh"
  - path: "repo:qual/tests/tc/TC-TRS-HPLE-004.sh"
  - path: "repo:qual/tests/tc/TC-TRS-HPLE-005.sh"
tags:
  - variability
  - multi-repo
---

Formal `qual/` coverage for `REQ-TRS-HPLE-001..005`, following the exact convention established
for `TC-TRS-SYSMLV2-*`/`TC-TRS-PLANITEM-*`: `qual/Requirements/`, `qual/TestCases/`,
`qual/fixtures/`, `qual/tests/tc/`. Full unfiltered `qual/tests/run_qual.sh` must stay green.

Five `TC-TRS-HPLE-001..005`, one per requirement, each fixture set mirroring
`TC-TRS-TYPE-021`'s own multi-repo pattern (a shared `peer/` dir plus per-scenario consolidator
subdirectories, each with its own `.syscribe.toml` `[repos]` entry): `-001` exercises the
`subConfigurations:` resolution + peer-validity gate (E516/E517/E518); `-002` the transitive
`parameterBindings:` resolution (clean resolve, E222 for a genuinely unreachable target); `-003`
cross-tier binding legality (E519, E523 naming the nearer tier); `-004` the opt-in `W513`
completeness warning, including `--deny` actually gating the exit code; `-005` `bindTo:` isolation,
via `feature-check` specifically (the only command that runs the `bindTo:`/`E202` check), with a
positive control plus both leak directions.

Full unfiltered `qual/tests/run_qual.sh --no-build`: **265 total, 265 passed, 0 failed, 6 skipped**
(the 6 skips are pre-existing, unrelated to HPLE — `TC-TRS-OUT-021..023`/`TC-TRS-SEARCH-001..003`
have no script yet). `TVR.md`/`results.ndjson` regenerated in the same batch.
