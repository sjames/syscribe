# CLI Reference

The `syscribe` binary is a command-line tool for validating, browsing, and querying Syscribe models. All examples on this page use the **Engine ECU** demo model (`model_auto/`).

## Pointing at a model

Every command needs to know where the model root is. Three ways to specify it, in priority order:

```bash
# Flag (short or long)
syscribe -m model_auto/ validate
syscribe --model model_auto/ validate

# Environment variable — useful in scripts and CI
export SYSCRIBE_MODEL=model_auto/
syscribe validate

# Default — if none of the above, looks for model/ in the current directory
syscribe validate
```

---

## Validation

### Full report

Running with no subcommand prints the full 10-section Markdown validation report — element inventory, requirements matrix, traceability summary, and findings.

```
$ syscribe -m model_auto/

...

Warnings (1):

| Code | File | Message |
|---|---|---|
| W803 | model_auto/Security/VR-ENG-002.md | VulnerabilityReport has status: open — ensure it is being tracked and mitigated |
```

### Findings only

`validate` prints just the errors and warnings table — useful for quick iteration.

```
$ syscribe -m model_auto/ validate

Warnings (1):

| Code | File | Message |
|---|---|---|
| W803 | model_auto/Security/VR-ENG-002.md | VulnerabilityReport has status: open — ensure it is being tracked and mitigated |
```

### Scoped to a single file

Useful when editing one file and you want instant feedback without re-validating the whole model.

```
$ syscribe -m model_auto/ validate --file model_auto/System/Software/SafetyMonitor.md

0 errors, 0 warnings — model is valid.
```

### Machine-readable output

`--json` emits a JSON array of findings — suitable for editor integrations and CI scripts.

```
$ syscribe -m model_auto/ validate --json

[
  {
    "code": "W803",
    "file": "model_auto/Security/VR-ENG-002.md",
    "message": "VulnerabilityReport has status: open — ensure it is being tracked and mitigated",
    ...
  }
]
```

---

## Browsing the model

### List root packages

```
$ syscribe -m model_auto/ ls

# (root)

| Name | Qualified Name | Type |
|---|---|---|
| Allocations | Allocations | Package |
| Decisions   | Decisions   | Package |
| Requirements | Requirements | Package |
| Safety      | Safety      | Package |
| Security    | Security    | Package |
| System      | System      | Package |
| Vehicle     | Vehicle     | Package |
| Verification | Verification | Package |
```

### List a namespace

```
$ syscribe -m model_auto/ ls System::Software

# System::Software

| Name | Qualified Name | Type |
|---|---|---|
| BootSequence           | System::Software::BootSequence           | ActionDef |
| CANSecurityModule      | System::Software::CANSecurityModule      | PartDef   |
| DiagnosticSecurityLayer | System::Software::DiagnosticSecurityLayer | PartDef  |
| EngineStallMonitor     | System::Software::EngineStallMonitor     | PartDef   |
| FuelControl            | System::Software::FuelControl            | PartDef   |
| SafetyMonitor          | System::Software::SafetyMonitor          | PartDef   |
| SecureBootManager      | System::Software::SecureBootManager      | PartDef   |
| ThrottleControl        | System::Software::ThrottleControl        | PartDef   |
```

### Recursive tree

```
$ syscribe -m model_auto/ tree System::Software

System::Software
├── BootSequence [ActionDef]
├── CANSecurityModule [PartDef]
├── DiagnosticSecurityLayer [PartDef]
├── EngineStallMonitor [PartDef]
├── FuelControl [PartDef]
├── SafetyMonitor [PartDef]
├── SecureBootManager [PartDef]
└── ThrottleControl [PartDef]
    └── canOut [Port]
```

### Show an element

`show` prints an element's fields, features, and documentation body.

```
$ syscribe -m model_auto/ show System::Software::SafetyMonitor

# System::Software::SafetyMonitor

| Field | Value |
|---|---|
| type    | PartDef  |
| file    | model_auto/System/Software/SafetyMonitor.md |
| domain  | software |
| ASIL    | D        |
| satisfies | REQ-ENG-SAFE-001, REQ-ENG-SAFE-004, REQ-ENG-SAFE-005 |

## Features

| Name | Type | Multiplicity |
|---|---|---|
| monitorCycleMs           | ScalarValues::Integer | 1 |
| tpsDivergenceThresholdPct | ScalarValues::Real   | 1 |

## Documentation

Safety monitoring software component (ASIL D). Supervises all safety-relevant
inputs and function outputs...
```

---

## Searching

### Fuzzy search

`find` searches element names, IDs, and documentation bodies. Results are ranked by relevance.

```
$ syscribe -m model_auto/ find throttle

# Search: `throttle`

| Score | Qualified Name | Type | Excerpt |
|---|---|---|---|
| 65 | System::Actuators::ThrottleActuator | PartDef | DC motor-driven electronic throttle body... |
| 65 | System::Sensors::ThrottlePositionSensor | PartDef | Dual-track potentiometric throttle position sensor... |
| 65 | System::Software::ThrottleControl | PartDef | Electronic throttle control (ETC) software component... |
| 50 | Requirements::Safety::REQ-ENG-SAFE-005 | Requirement | Throttle close command shall be verified by position feedback... |
| 50 | Safety::FMEA::FMEA-ENG-001::FM-ENG-001 | FMEAEntry | Throttle plate stuck open at >20 % position |
| ...  | | | |
```

### List elements by type

```
$ syscribe -m model_auto/ list Requirement

# Requirement elements (14)

| Qualified Name | Name / ID |
|---|---|
| Requirements::Safety::REQ-ENG-SAFE-001 | Safety monitor shall detect all safety faults within 100 ms |
| Requirements::Safety::REQ-ENG-SAFE-002 | Hardware watchdog shall reset ECU within 30 ms of software failure |
| ...
```

Scope to a namespace with an optional second argument:

```
$ syscribe -m model_auto/ list PartDef System::Software

# PartDef elements in `System::Software` (7)

| Qualified Name | Name / ID |
|---|---|
| System::Software::CANSecurityModule | CAN Security Module |
| System::Software::SafetyMonitor     | Safety Monitor      |
| ...
```

Filter by free-text `tags:` with `--tag` (repeatable; orthogonal to the feature model):

```
$ syscribe -m model/ list Requirement --tag smoke
```

---

## Variability (product lines)

The variability dimension is **opt-in**: it stays dormant unless the model declares a `FeatureDef` and links something to it. See the [validation rules](../validation/rules.md) (E209, W015) and §9 of the format spec.

`appliesWhen:` conditions any element (including a `TestCase`) on a boolean expression over `FeatureDef` qualified names — `and` / `or` / `not` / parentheses, with a bare QName or a list (AND) also accepted:

```yaml
appliesWhen: "Features::CortexM and Features::Mpu"
appliesWhen: "not Features::Smp"
```

A `TestCase` *runs in* a `Configuration` iff its `appliesWhen:` is satisfied by that configuration's `features:` selections (no `appliesWhen:` ⇒ runs everywhere). There is no `runsIn` field.

### Coverage matrix

`matrix` emits a Requirement × Configuration grid. Columns are the model's `Configuration` elements; cells are covered (`✓`), gap (`✗`), or N/A (`—`, requirement not active in that variant):

```
$ syscribe -m model/ matrix
$ syscribe -m model/ matrix --json            # structured grid (schemaVersion, columns, rows)
$ syscribe -m model/ matrix --tag safety      # filter rows by tag
```

With no feature model present, `matrix` prints a notice and falls back to a flat requirement/testcase view (exit 0).

`refs <CONF-id>` additionally lists the `TestCase`s that run in a given configuration.

### Feature-model check

`feature-check` runs holistic feature-model validation that is deliberately kept out of the per-element `validate` pass — `requires`/`excludes` resolution and satisfaction, dead/always-on optional features, `derivedFrom:` cycles, `bindTo:` propagation ranges, and `parameterConstraints` paths (see the [validation rules](../validation/rules.md)):

```
$ syscribe -m model/ feature-check
$ syscribe -m model/ feature-check --json
```

Exit code is `0` when there are no errors and `1` otherwise; with no `FeatureDef` present it prints a notice and exits `0`.

Add `--deep` for SAT-backed whole-configuration-space analysis (over a propositional encoding of the feature model — deterministic; uses batsat, a pure-Rust CDCL solver, in-process):

```
$ syscribe -m model/ feature-check --deep
$ syscribe -m model/ feature-check --deep --json
```

`--deep` detects **void** models (`E223`), **dead** features (`E224`), **false-optional** features (`W018`), and **invalid configurations** under full group/cardinality semantics (`E225`), reports **core** features, and explains each unsatisfiability with a conflict set. It covers the Boolean feature layer only (parameter satisfiability is out of scope) and skips with a notice on models above a feature-count guard.

---

## Traceability

### Full trace for a requirement

`trace` shows a requirement's parents, breakdown ADR, safety goal, satisfying architecture elements, and covering test cases in one view.

```
$ syscribe -m model_auto/ trace REQ-ENG-SAFE-001

# Trace: REQ-ENG-SAFE-001

Title:  Safety monitor shall detect all safety faults within 100 ms
Status: approved · domain: software · ASIL: D

## Parents (derivedFrom)
- REQ-ENG-SAFE-000 — Engine ECU shall prevent safety hazards identified in HARA

## Breakdown ADR
- ADR-ENG-SAFE-001 — ASIL D decomposition for engine safety requirement (accepted)

## Safety Goal (derivedFromSafetyGoal)
- SG-ENG-001 — Prevent unintended engine acceleration (ASIL D)

## Satisfied by
- System::Software::SafetyMonitor [PartDef, software]

## Verified by
- TC-ENG-SAFE-002 — HIL — TPS dual-track divergence triggers safe state (L5)
```

### What does this component satisfy?

```
$ syscribe -m model_auto/ why System::Software::SafetyMonitor

# Why: System::Software::SafetyMonitor

## Satisfied requirements

| ID | Title | ASIL |
|---|---|---|
| REQ-ENG-SAFE-001 | Safety monitor shall detect all safety faults within 100 ms | D |
| REQ-ENG-SAFE-004 | Rev limiter shall cut fuel and retard ignition above 6500 rpm | A |
| REQ-ENG-SAFE-005 | Throttle close command shall be verified by position feedback | A |

## Verification coverage

| TC ID | Level | Covers |
|---|---|---|
| TC-ENG-SAFE-002 | L5 | REQ-ENG-SAFE-001 |
| TC-ENG-SAFE-005 | L5 | REQ-ENG-SAFE-004 |
| TC-ENG-SAFE-006 | L5 | REQ-ENG-SAFE-005 |
```

### Which test cases cover a requirement?

```
$ syscribe -m model_auto/ who-verifies REQ-ENG-SAFE-001

# Verification: REQ-ENG-SAFE-001

| TC ID | Level | Gherkin Scenarios | Status |
|---|---|---|---|
| TC-ENG-SAFE-002 | L5 | 1 | active |
```

### All relationships on an element

`links` shows every outbound and inbound relationship — useful for impact analysis before editing a file.

```
$ syscribe -m model_auto/ links System::Software::SafetyMonitor

## Outbound relationships

| Relationship | Target | Type |
|---|---|---|
| satisfies | REQ-ENG-SAFE-001 | Requirement |
| satisfies | REQ-ENG-SAFE-004 | Requirement |
| satisfies | REQ-ENG-SAFE-005 | Requirement |

## Inbound relationships

| Source | Relationship | Type |
|---|---|---|
| Safety::FMEA::FMEA-ENG-001::FM-ENG-006 | subject | FMEAEntry |
| Safety::FMEA::FMEA-ENG-001::FM-ENG-010 | subject | FMEAEntry |
| Vehicle::PowertrainECU::softwareImage::safetyMonitor | typedBy | Part |
```

### What references this element?

```
$ syscribe -m model_auto/ refs System::Software::SafetyMonitor

# References to: System::Software::SafetyMonitor

| Source | Relationship | Type |
|---|---|---|
| Safety::FMEA::FMEA-ENG-001::FM-ENG-006 | subject | FMEAEntry |
| Safety::FMEA::FMEA-ENG-001::FM-ENG-010 | subject | FMEAEntry |
| Vehicle::PowertrainECU::softwareImage::safetyMonitor | typedBy | Part |
```

---

## Authoring helpers

### Resolve a cross-reference

Verify that a qualified name or stable ID is resolvable before writing it into a `derivedFrom:` or `satisfies:` field.

```
$ syscribe -m model_auto/ check-ref System::Software::ThrottleControl

resolved  System::Software::ThrottleControl
type      PartDef
file      model_auto/System/Software/ThrottleControl.md
```

### Find the file path for an element

```
$ syscribe -m model_auto/ path-for REQ-ENG-SAFE-001

model_auto/Requirements/Safety/REQ-ENG-SAFE-001.md
```

### Get the next available ID

`next-id` scans the model for all IDs matching a prefix and returns the next unused one — prevents ID collisions when adding new elements.

```
$ syscribe -m model_auto/ next-id REQ-ENG-SAFE

REQ-ENG-SAFE-006
```

### Print a frontmatter template

`template` prints a ready-to-fill skeleton for any element type. Pipe it directly into a new file.

```
$ syscribe -m model_auto/ template Requirement

---
type: Requirement
id: REQ-PREFIX-001
title: "The system shall ..."
status: draft
reqDomain: system
verificationMethod: test
# silLevel: 1
# asilLevel: A
# derivedFrom:
#   - REQ-PARENT-001
# breakdownAdr: ADR-XXX-001
# derivedFromSafetyGoal: SG-PREFIX-001
---

The system shall ...

## Rationale

Why this requirement exists.
```

Create a new requirement file in one step:

```bash
syscribe -m model_auto/ template Requirement \
  > model_auto/Requirements/Safety/REQ-ENG-SAFE-006.md
```

---

## Format spec browser

The `spec` subcommand gives you the full Syscribe format specification without leaving the terminal. No model root needed.

```bash
syscribe spec                  # table of contents
syscribe spec types            # all element types and their schemas
syscribe spec fields           # complete frontmatter field reference
syscribe spec validation       # all validation rule codes (E001–W808)
syscribe spec traceability     # traceability rules R-001–R-007
syscribe spec safety           # safety/security analysis elements
```

---

## LLM authoring prompt

Print the prompt that instructs an LLM to produce valid Syscribe models:

```bash
syscribe --agent-instructions
```

Pipe it directly into your LLM tool of choice:

```bash
syscribe --agent-instructions | llm "Create a brake-by-wire model for ISO 26262 ASIL D"
```

---

## Use by agents and MCP servers

The CLI is designed to be driven by an agent (Claude Code, a CI script, or an MCP server tool) without any modification. Every command writes structured output to stdout and exits cleanly.

### Setting the model root from the environment

Agents that launch subprocesses should set `SYSCRIBE_MODEL` once rather than repeating the flag on every command:

```bash
export SYSCRIBE_MODEL=model_auto/
syscribe validate --json   # no -m needed
syscribe show System::Software::SafetyMonitor
syscribe next-id REQ-ENG-SAFE
```

### Machine-readable validation output

`validate --json` emits a JSON array, one object per finding. An agent reads this directly without parsing Markdown tables.

```json
[
  {
    "code": "W803",
    "severity": "warning",
    "file": "model_auto/Security/VR-ENG-002.md",
    "message": "VulnerabilityReport has status: open — ensure it is being tracked and mitigated"
  }
]
```

An empty array means the model is valid. A non-zero exit code is only set when the command itself fails (bad path, parse error) — a model with warnings still exits 0.

### The incremental authoring loop

The recommended workflow for an agent authoring a new model is to write files in batches and validate after each one. The validator output tells the agent exactly what to fix before moving on.

```
agent writes Batch 4 — Requirements
  → syscribe -m model_auto/ validate --json
  → reads [{"code":"E310","file":"...","message":"missing breakdownAdr"}]
  → fixes the file in the same turn
  → syscribe -m model_auto/ validate --json
  → reads []   ← clean; move to Batch 5
```

See the [LLM Workflow guide](../model-guide/llm-workflow.md) for the full eight-batch sequence and per-batch error checklist.

### Commands useful to agents

| Command | What it returns | When to use |
|---|---|---|
| `validate --json` | JSON findings array | After every batch of writes |
| `validate --file <path>` | Scoped findings | After editing a single file |
| `check-ref <qname\|id>` | Resolved type and file, or error | Before writing a cross-reference |
| `next-id <prefix>` | Next unused stable ID | Before writing a new REQ-*, TC-*, ADR-* |
| `show <qname>` | Full element fields and doc | To read an element before modifying it |
| `trace <req-id>` | Parents, ADR, safety goal, satisfiers, test cases | Impact analysis before changing a requirement |
| `links <qname>` | All outbound and inbound relationships | Impact analysis before changing an element |
| `path-for <qname\|id>` | Absolute file path | To open or overwrite the file for an element |
| `list <type> [scope]` | All elements of a type, optionally scoped | To enumerate IDs in use before authoring |
| `--agent-instructions` | Full generation prompt | System prompt for a model-authoring session |

### Exposing syscribe as an MCP tool

Any MCP server that can execute shell commands can expose syscribe as a set of tools. The simplest pattern is one tool per command group, using `SYSCRIBE_MODEL` to avoid passing the path on every call:

```json
{
  "tools": [
    {
      "name": "syscribe_validate",
      "description": "Validate the Syscribe model and return a JSON array of findings. Empty array means valid.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "file": { "type": "string", "description": "Optional: scope validation to a single file path" }
        }
      }
    },
    {
      "name": "syscribe_show",
      "description": "Show all fields and documentation for a model element.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "qname": { "type": "string", "description": "Qualified name or stable ID (e.g. System::Software::SafetyMonitor or REQ-ENG-SAFE-001)" }
        },
        "required": ["qname"]
      }
    },
    {
      "name": "syscribe_trace",
      "description": "Full traceability slice for a requirement: parents, ADR, safety goal, satisfiers, test cases.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "id": { "type": "string", "description": "Stable requirement ID (e.g. REQ-ENG-SAFE-001)" }
        },
        "required": ["id"]
      }
    },
    {
      "name": "syscribe_next_id",
      "description": "Return the next unused stable ID for a given prefix.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "prefix": { "type": "string", "description": "ID prefix, e.g. REQ-ENG-SAFE or TC-ENG-SEC" }
        },
        "required": ["prefix"]
      }
    }
  ]
}
```

Each tool handler runs the corresponding `syscribe` command, captures stdout, and returns it as the tool result. The agent calls `syscribe_validate` after every write and only proceeds when it returns `[]`.
