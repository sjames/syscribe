# User-Defined Link Types Example: Brake-by-Wire

A small, standalone brake-by-wire model demonstrating **user-defined link
types** (`ADR-SYS-LINKTYPE-001`, `REQ-TRS-LINKTYPE-000` through `-012`,
spec §12.10). Three project-specific relationships are declared once in
`model/.syscribe.toml` and authored under `links:` on ordinary elements:

| Link type | Kind | What it shows |
|---|---|---|
| `mitigates` (inverse `mitigatedBy`) | free-standing | Own constraints only: `sourceTypes = ["PartDef"]`, `targetTypes = ["Requirement"]`. No built-in rule applies, so a software monitor may mitigate a hardware requirement. |
| `partiallySatisfies` (inverse `partiallySatisfiedBy`) | `extends = "satisfies"` | A named variant of `satisfies` with `relax = ["E313"]` (cross-domain allowed) and `coverage = false` (no credit in `satisfiedBy`). |
| `conflictsWith` (no inverse) | free-standing, symmetric | Requirement to requirement; `acyclic = false` so two requirements naming each other is legal; `suspect = false` opts it out of suspect-link detection. |

It is a separate model root from this repository's own `model/`, so running
validation here never affects that model's baseline. A second, deliberately
broken model root (`error-demo/model/`) shows the link-type checks firing.

## Running it

```bash
cargo build --workspace   # once, if you haven't already

# Main example: 0 errors
./target/debug/syscribe -m examples/link-types/model validate

# The project's link-type vocabulary (what an LLM agent should run first)
./target/debug/syscribe -m examples/link-types/model link-types
./target/debug/syscribe -m examples/link-types/model link-types --json

# Follow a custom link forward, via its inverse, and with --reverse
./target/debug/syscribe -m examples/link-types/model follow Architecture::WatchdogMonitor mitigates
./target/debug/syscribe -m examples/link-types/model follow REQ-BRK-003 mitigatedBy
./target/debug/syscribe -m examples/link-types/model follow REQ-BRK-003 mitigates --reverse

# A relaxed satisfies variant, as JSON
./target/debug/syscribe -m examples/link-types/model follow REQ-BRK-003 partiallySatisfiedBy --format json

# Transitive traversal terminates on the conflictsWith 2-cycle
./target/debug/syscribe -m examples/link-types/model follow REQ-BRK-004 conflictsWith --transitive

# Built-in links and reverse indices work with follow too
./target/debug/syscribe -m examples/link-types/model follow REQ-BRK-001 derivedChildren --format dot
./target/debug/syscribe -m examples/link-types/model follow REQ-BRK-003 satisfiedBy

# Existing relationship commands include custom links
./target/debug/syscribe -m examples/link-types/model links REQ-BRK-003
./target/debug/syscribe -m examples/link-types/model refs REQ-BRK-003
./target/debug/syscribe -m examples/link-types/model trace REQ-BRK-003
./target/debug/syscribe -m examples/link-types/model impact REQ-BRK-003 --direction both --kinds mitigates,partiallySatisfies

# The authoring prompt, with a "Project link types" section appended
./target/debug/syscribe -m examples/link-types/model --agent-instructions | tail -40

# Deliberate error demonstration
./target/debug/syscribe -m examples/link-types/error-demo/model validate
```

The architecture `PartDef`s are never used as a `supertype`/`typedBy`, so the
main model also carries the usual `W007` "defined but never used" advisories;
they are unrelated to link types.

## File layout

```
examples/link-types/
  README.md                         This file
  model/                            Main example: 0 errors
    .syscribe.toml                  [linkTypes.mitigates], [linkTypes.partiallySatisfies],
                                    [linkTypes.conflictsWith]
    _index.md                       Root package
    Decisions/
      ADR-BRK-001.md                breakdownAdr for the three leaf requirements
    Requirements/
      REQ-BRK-001.md                system parent (stakeholder)
      REQ-BRK-002.md                software leaf; links: conflictsWith: REQ-BRK-004
      REQ-BRK-003.md                hardware leaf; target of mitigates and partiallySatisfies
      REQ-BRK-004.md                software leaf; links: conflictsWith: REQ-BRK-002
    Architecture/
      BrakeController.md            software; satisfies REQ-BRK-002,
                                    links: partiallySatisfies: REQ-BRK-003 (E313 relaxed)
      HydraulicModulator.md         hardware; satisfies REQ-BRK-003
      RegenBlender.md               software; satisfies REQ-BRK-004
      WatchdogMonitor.md            software; links: mitigates: [REQ-BRK-002, REQ-BRK-003]
      PressureReliefValve.md        hardware; links: mitigates: REQ-BRK-003
    Tests/
      TC-BRK-001.md                 L4 integration test for the parent (W305)
      TC-BRK-002..004.md            one test per leaf requirement (W002)
  error-demo/model/                 Deliberately broken companion
```

## What to look at

**`partiallySatisfies` relaxes exactly one rule, for itself only.**
`BrakeController` is `domain: software`; `REQ-BRK-003` is `reqDomain:
hardware`. A plain `satisfies:` would be `E313`. Because `partiallySatisfies`
declares `extends = "satisfies"` and `relax = ["E313"]`, that instance is
exempt, while every other `satisfies` rule (for example `E312`, no
assignment to a parent requirement) still applies to it. `coverage = false`
keeps it out of `REQ-BRK-003`'s `satisfiedBy`, so `HydraulicModulator` remains
the single satisfier: `follow REQ-BRK-003 satisfiedBy` returns only it, while
`follow REQ-BRK-003 partiallySatisfiedBy` returns `BrakeController`. If you
deleted `HydraulicModulator`'s `satisfies:`, `REQ-BRK-003` would raise `W300`,
because partial contributions earn no coverage credit.

**`mitigates` is free-standing.** It has no `extends`, so no built-in rule
applies to it; only its declared `sourceTypes`/`targetTypes`/`cardinality`
do. `WatchdogMonitor` (software) mitigates both a software and a hardware
requirement with no domain check involved. To require every `PartDef` to
mitigate something, you would set `cardinality = "1..*"`; every non-draft
`PartDef` without a `mitigates` link would then raise `W631`.

**`conflictsWith` is symmetric and informational.** `REQ-BRK-002` and
`REQ-BRK-004` name each other. The 2-cycle is legal because the type is
`acyclic = false`; with `acyclic = true` it would be `E636`. `suspect = false`
keeps these links out of `suspect list` and `W090`, since a trade-off
relationship does not become stale when either requirement's wording is
edited. `mitigates` and `partiallySatisfies` keep the default `suspect = true`:
baseline them with `suspect accept --all-unbaselined`, and editing
`REQ-BRK-003` afterwards flags them as `W090`.

## Deliberate error demonstration

`error-demo/model/` is expected to fail validation. Each finding is
intentional:

| File | Finding | Why |
|---|---|---|
| `.syscribe.toml` | `W630` | `looselySatisfies` relaxes `E104`, which is not relaxable for a `satisfies` variant; the whole entry is ignored. |
| `Architecture/TypoLink.md` | `E630` | `mitigate` is not declared. The message lists the declared types (`mitigates`, `partiallySatisfies`), so the typo corrects itself. |
| `Architecture/UsesIgnored.md` | `E630` | Uses `looselySatisfies`, whose declaration was rejected under `W630`. |
| `Architecture/WrongTarget.md` | `E634` | `mitigates` targets a `PartDef`, but `targetTypes = ["Requirement"]`. |
| `Architecture/StrictCrossDomain.md` | `E313` | The built-in `satisfies:` is never relaxed. The `partiallySatisfies` link to the same requirement on the same element raises nothing. |

`REQ-ERR-LT-001` is a draft orphan, so the demo also raises `W005`, and the
unused `PartDef`s raise `W007`; neither relates to link types.
