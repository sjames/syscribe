---
type: Requirement
id: REQ-TRS-PLANCFG-001
name: "A TestPlan that names its Configurations is judged on their results only"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - testplan
  - results
---

A `TestPlan` that lists `configurations:` shall be judged on the results recorded for those Configurations, and on no other's (GH #528). Results are recorded per Configuration by `ingest-results --config` (`REQ-TRS-CFGRES-001`).

- Each listed Configuration is judged on its own effective results: the results recorded without a Configuration, overlaid with its own. A result recorded for a Configuration the plan does not list, pass or fail, shall not change the plan's verdict, its required-coverage gaps (`REQ-TRS-PLANCOV-002`), `W615` or `W617`.
- A member TestCase is judged in a Configuration only when it is active there. A member active in none of the plan's Configurations leaves the plan `incomplete`, as before.
- The plan verdict is the worst over its Configurations: `fail` over `incomplete` over `pass`. A Configuration with no passing result for a member leaves the plan `incomplete`, as a `runsOn:` rig does (`REQ-TRS-RESENV-001`).
- When a listed Configuration has results of its own, `testplan TP-X` shall name the Configuration of each gap (`<test> @ <rig> [axis=point] in <config> (…)`), and with several Configurations list the verdict of each; `--json` carries `verdictByConfiguration` and a `configuration` on each gap. `W615` and `W617` name the Configuration the same way. When none has results of its own they all read the same results, and the plan is judged once with no Configuration named.
- `audit`, the plan list and `trace <REQ>`'s required-coverage standing shall use the same verdict and gaps.
- A plan without `configurations:` shall be judged as before, on the worst over everything recorded.
