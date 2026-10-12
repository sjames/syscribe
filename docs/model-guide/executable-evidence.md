# Executable evidence

A TestCase whose `subject:` is a state machine, an executable action or a composite, and whose body holds Gherkin
scenarios, runs: `simulate` executes it in-process. Two reports turn those runs into verification evidence (GH #333,
#344), and they share one engine, so their numbers agree:

* `requirement-coverage` answers *which requirements are verified by something that runs, and how well*;
* `exec-coverage` answers *which architecture elements are modelled, and what exercises them*.

The full rules are in spec §12.12.

## Claiming requirements per scenario

A scenario verifies the requirements of its TestCase's `verifies:`. When a TestCase verifies several requirements,
tag each scenario with the ones it is evidence for:

```gherkin
Feature: Grid connection and protection chain (US)

  @verifies REQ-GTI-SYS-030
  Scenario: insulation below the limit prevents connection until it recovers
    ...

  @invariant @verifies REQ-GTI-SYS-010,REQ-GTI-SYS-038
  Scenario: a dead grid at power-up is never energised
    ...
```

`@verifies(REQ-A, REQ-B)` and `@verifies:REQ-A` work too; a tag before `Feature:` applies to every scenario, and a
`# verifies: REQ-A` comment line before a step adds to the scenario's claims (in `Background:`, to every scenario's).
Tags on an `Examples:` table apply to its rows. An untagged scenario, or one whose every claim is invalid, verifies
every requirement of its TestCase. Tags only narrow: a reference outside the TestCase's `verifies:` is ignored with `W994`,
and when every scenario is tagged but none claims one of the `verifies:` requirements, that requirement has no
evidence from this TestCase (`W995`).

## requirement-coverage

    syscribe -m model_gti requirement-coverage --config CONF-GTI-US-M3-001

grades every requirement:

| Verdict | Meaning |
|---|---|
| `executable` | passing scenarios exercise an architecture element that satisfies it |
| `executable-weak` | such scenarios exist, but one fails, or coverage or mutation score is below a threshold you gave |
| `descriptive-only` | only TestCases that do not run verify it |
| `unverified` | nothing verifies it |
| `not-applicable` | verified by inspection or analysis, or a parent whose children all have executable evidence or need none |

For the US configuration of the inverter, 11 of the 63 applicable requirements are `executable`; the protection
scenarios carry `REQ-GTI-SYS-007`, `-010`, `-030`, `-038`, `-039`, `REQ-GTI-SW-001` and `REQ-GTI-HW-008`/`-009`, and
the MPPT scenarios `REQ-GTI-SYS-022`/`-023` and `REQ-GTI-SW-006`. `REQ-GTI-SYS-037` (the 10 µs overcurrent trip) is
`descriptive-only`: the parts that model the power stage name it in `abstracts:`, so only the bench TestCases verify it.

Name requirements to see the detail: each scenario that is evidence, the coverage those scenarios give of every
machine that carries the requirement (merged over that requirement's scenarios only), mutation scores, satisfiers
and descriptive tests:

    syscribe -m model_gti requirement-coverage REQ-GTI-SYS-030 --config CONF-GTI-US-M3-001

A scenario that claims a requirement but whose subject exercises nothing that satisfies it is listed as *claimed but
not exercised* and does not count. A parent requirement with no executable evidence of its own needs none when all its
children have it (or need none); otherwise it keeps its own verdict and the report names the children that lack it.
The inverter's stakeholder requirements are `descriptive-only` for that reason.

### Thresholds and mutation scores

`--min-states`, `--min-transitions`, `--min-guards` and `--min-score` (percentages) make evidence below them
`executable-weak`; a requirement for which no state-machine coverage is measured (an action subject) is weak under any
of them, since they cannot be checked. A score belongs to a machine and the subject whose scenarios judged it; a
machine used by several parts (the two relays) has one score summed over them. `mutate <subject> --record` keeps the
scores in `.syscribe/mutation.json`, one per machine version (a machine projected onto two Configurations keeps two);
a score whose machine changed since, or that cannot be checked because the machine does not lower, is marked stale.
`--mutate` computes the scores on the spot instead.

    syscribe -m model_gti mutate TC-GTI-PROT-US-001 --config CONF-GTI-US-M3-001 --record
    syscribe -m model_gti requirement-coverage --config CONF-GTI-US-M3-001 --min-score 80

## exec-coverage

    syscribe -m model_gti exec-coverage --config CONF-GTI-US-M3-001

lists every software and hardware `Part`/`PartDef`: executable itself (`System::Software::Supervisor` carries its
machine), modelled by executable parts (`System::Software::ProtectionManager` by `GridMonitorPart`), or not modelled;
how many scenarios exercise it and how many passed; the `models:` correspondence findings that name it; mutation scores; and its
requirements by verdict, keeping apart executable evidence that exercised another element satisfying the same requirement. Seven of the inverter's 23 elements are modelled.

## Gates

| Gate | Fails (exit 2) when |
|---|---|
| `requirement-coverage --min-executable <pct>` | fewer of the applicable requirements are `executable` |
| `requirement-coverage --require <REQ>` | that requirement is not `executable` |
| `exec-coverage --min-modelled <pct>` | fewer of the elements are modelled |
| `exec-coverage --require <element>` | that element is not modelled, or no passing scenario exercises it |

The MCP server exposes the same data read-only as `requirement_coverage` and `exec_coverage`.

## Limits of this version

* Scenarios that use plants (`plants:` on the TestCase) are listed as not run.
* A requirement no architecture element satisfies is exercised by any subject, and every product part's machine
  counts toward its coverage.
* `matrix`, the safety case and the coverage tree do not read these verdicts yet.
