# Part III — Architecture

`GUIDE · PART III — ARCHITECTURE`


### Without MagicGrid — Lightweight Architecture Viewpoint

#### 3.1 PartDef: the architecture building block

A `PartDef` is a named system component — a SysML Block expressed as a Markdown file.

```yaml
---
type: PartDef
name: KernelScheduler
domain: software
asilLevel: D
satisfies:
  - REQ-SCHED-001
  - REQ-SCHED-002
---

The priority-preemptive round-robin scheduler. Maintains one ready queue per
priority level. Context-switch cost is O(1) in the number of threads.
```

The `satisfies:` field closes the traceability chain from requirement to architecture.
The `why` command shows what a component satisfies:

```bash
syscribe -m model why SabatonRt::Software::KernelScheduler
```

#### 3.2 Architecture Decision Records (ADRs)

ADRs capture *why* a design choice was made. They are required when a requirement is derived
(`breakdownAdr:` mandatory — E310 if absent). They serve the same function as the "rationale"
column in a DOORS attribute scheme, but with a full prose record.

```yaml
---
type: ADR
id: ADR-SCHED-001
name: "Priority-preemptive scheduler with O(1) ready queue"
status: accepted
---

## Context

Multiple safety requirements (REQ-SCHED-001..004) demand deterministic scheduling
under bounded WCET. A priority-sorted bitmap approach gives O(1) enqueue, dequeue,
and highest-priority lookup.

## Decision

Use a 32-level bitmap-indexed ready queue (one bit per priority level), with a
circular list at each level for round-robin within a priority.

## Consequences

- Positive: O(1) scheduling decision; WCET bounded independently of thread count.
- Negative: Priority inversion possible without explicit inheritance; documented
  as AOU-KERNEL-001.
```

**For auditors**: the ADR is the documented rationale. Every architectural decision that
allocates a requirement to a component gets an ADR. `list ADR` shows all decisions; `show
ADR-SCHED-001` shows the full record.

#### 3.3 Diagrams

Diagrams are referenced artifacts — SVG or image files linked from `Diagram` elements.
They appear in `syscribe show` output and are included in generated reports.

```yaml
---
type: Diagram
name: SchedulerDataFlow
diagramFile: ../docs/diagrams/scheduler-data-flow.svg
elements:
  - SabatonRt::Software::KernelScheduler
  - SabatonRt::Software::TCB
---

Context diagram showing data flow between the scheduler ready queue and the
TCB pool on a context switch.
```

#### 3.4 Allocations

Record what implements what using `allocatedTo:` (lightweight) or a standalone `Allocation`
element (when you need documented rationale):

```yaml
# on the source element (lightweight)
allocatedTo: Hardware::CortexM33Core

# or as a standalone element when rationale matters
---
type: Allocation
name: SchedulerToCore0
allocatedFrom: SabatonRt::Software::KernelScheduler
allocatedTo: Hardware::CortexM33Core
---
Scheduler pinned to Core 0 (CORE_BOOT) for AMP isolation (REQ-AMP-001).
```

---

### With MagicGrid — Full MBSE Methodology

MagicGrid is a structured MBSE method from No Magic / Cameo that organises the model as a
**3 × 4 grid**: rows are abstraction levels (Problem black box B, Problem white box W,
Solution S); columns are SysML pillars (1 Requirements, 2 Behavior, 3 Structure, 4 Parameters).

syscribe implements MagicGrid as an optional overlay — the same element types, with
`mg_*` keys in `custom_fields:`. Enable it once:

```toml
# .syscribe.toml
[profiles.magicgrid]
magicgrid = true
promote = ["W307"]   # use cases must refine a stakeholder need
```

#### 3.5 The grid and what goes in each cell

```
                 1 Requirements      2 Behavior          3 Structure         4 Parameters
B (Problem       B1 Stakeholder      B2 Use Cases        B3 System Context   B4 Measures of
  Black Box)        Needs               (actors)            (SoI + externals)   Effectiveness (MoE)
W (Problem       W1 System           W2 Functional       W3 Logical          W4 Measures of
  White Box)        Requirements        Analysis            Architecture        Performance (MoP)
S (Solution)     (component reqs)    (component behav.)  S3 Physical         S4 Design
                                                            Components          Parameters
```

**Build order**: B1 → B2 → B3 → B4 → W1 → W2 → W3 → W4 → S3 → S4.
The `magicgrid` command shows which cells are populated and which are empty.

#### 3.6 B1 — Stakeholder needs

These are the top-level "the system shall" statements that capture what stakeholders want —
*before* technical decisions. In a safety product they often map to HARA hazardous events
and regulatory demands.

```yaml
---
type: Requirement
id: REQ-STAKE-001
name: "Vehicle shall not enter an unsafe state due to scheduler failure"
status: approved
requirementKind: stakeholder
custom_fields:
  mg_cell: B1
---

Stakeholder need: a scheduler defect **shall not** propagate to a vehicular hazard.
Derived from HARA hazardous event HE-KERNEL-001.
```

#### 3.7 B2 — Use cases

Use cases capture *what the system does for each actor*, not how. Every use case must refine
a B1 stakeholder need (W307 if it does not).

```yaml
---
type: UseCaseDef
name: ScheduleHighPriorityTask
refines:
  - REQ-STAKE-001
actors:
  - SabatonRt::Actors::SafetyMonitorThread
custom_fields:
  mg_cell: B2
---

**Actor**: Safety monitor thread (external to the kernel)  
**Primary flow**: Thread becomes ready → kernel scheduler selects it as highest priority →
context switch completes within bounded WCET → safety monitor executes.
```

#### 3.8 B3 — System context

Define the system of interest (SoI) and its external actors. Exactly one element gets
`mg_soi: true`; the rest are `mg_external: true`.

```yaml
---
type: PartDef
name: SabatonRtKernel
custom_fields:
  mg_soi: true
  mg_cell: B3
---
```

```yaml
---
type: PartDef
name: ApplicationThread
custom_fields:
  mg_external: true
  mg_cell: B3
---
```

#### 3.9 B4 — Measures of Effectiveness

MoEs are the *measurable* qualities the stakeholder cares about. They become the criteria
in a trade study and drive W4 Measures of Performance.

```yaml
---
type: CalculationDef
name: SchedulingLatency
custom_fields:
  mg_cell: B4
  mg_moe: true
  mg_moe_measures: REQ-STAKE-001
  mg_moe_unit: microseconds
  mg_moe_direction: minimize
  mg_moe_threshold: 100
  mg_moe_objective: 10
  mg_moe_weight: 3.0   # relative importance in trade study
---

Context-switch latency from the moment the highest-priority thread becomes ready
to the moment it begins execution.
```

#### 3.10 W1 — System requirements

System requirements are derived from B1 needs (with an ADR for rationale). They live at W1
and carry the integrity level.

```yaml
---
type: Requirement
id: REQ-SCHED-001
name: "Scheduler shall select highest-priority ready thread in O(1)"
status: approved
asilLevel: D
derivedFrom: [REQ-STAKE-001]
breakdownAdr: Decisions::ADR-SCHED-001
custom_fields:
  mg_cell: W1
---
```

#### 3.11 W2 — Functional analysis

W2 ActionDefs describe the functions the system performs to satisfy W1 requirements. Each
must be allocated to a W3 logical subsystem (MG081 if not).

```yaml
---
type: ActionDef
name: SelectNextThread
refines: [REQ-SCHED-001]
allocatedTo: Architecture::Logical::Scheduler
custom_fields:
  mg_cell: W2
---
```

#### 3.12 W3 / S3 — Logical and physical architecture

W3 logical subsystems are technology-agnostic; S3 physical components are the actual
implementation. The allocation chain W2→W3→S3 is checked by the validator.

```yaml
# W3 logical subsystem
---
type: PartDef
name: Scheduler
custom_fields:
  mg_cell: W3
  mg_layer: logical
allocatedTo: Architecture::Physical::CortexM33Core
---

# S3 physical component
---
type: PartDef
name: CortexM33Core
domain: hardware
custom_fields:
  mg_cell: S3
  mg_layer: physical
---
```

#### 3.13 W4 / S4 — Measures of Performance and design parameters

W4 MoPs refine the B4 MoEs with concrete engineering targets. They drive the quantitative
acceptance criteria in tests.

```yaml
---
type: ConstraintDef
name: ContextSwitchWCET
custom_fields:
  mg_cell: W4
  mg_mop: true
  mg_mop_refines: Problem::BlackBox::SchedulingLatency
  mg_mop_unit: microseconds
---

Context switch latency **shall** be ≤ 8 µs at 120 MHz on Cortex-M33 with
no FPU state and a 4-region MPU domain.
```

#### 3.14 Trade study

When you have multiple design alternatives, model them as `Configuration`s with
`parameterBindings:` matching the W4/S4 MoP names. `syscribe trade-study` scores each
alternative against the B4 MoE weights and identifies the winner:

```bash
syscribe -m model trade-study
```

#### 3.15 MagicGrid inspection commands

```bash
syscribe -m model magicgrid              # cell population report (B/W/S × 1-4)
syscribe -m model magicgrid --audit      # readiness + PASS/FAIL verdict
syscribe -m model matrix --allocations   # function→logical→physical allocation matrix
syscribe -m model trade-study            # MoE-weighted alternative scoring
syscribe -m model validate --profile magicgrid   # full gate with MG-code checks
```

---
