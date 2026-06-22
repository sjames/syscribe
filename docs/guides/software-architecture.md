# Part IX — Software Architecture: Level 2 (High-Level Design) and Level 3 (Detailed Component Design)

`GUIDE · PART IX — SOFTWARE ARCHITECTURE`

### 9.1 The two-level distinction and the standards that require it

Functional safety standards mandate a two-level hierarchy for software design documentation.
The exact names differ between standards but the intent is the same:

| Level | IEC 61508-3 | ISO 26262-6 | Typical deliverable |
|---|---|---|---|
| **Level 2** | §7.4.3 Software Architectural Design | §7 Software Architectural Design | Software Architecture Description (SAD) |
| **Level 3** | §7.4.4 Software Unit Design | §8 Software Unit Design and Implementation | Software Detailed Design (SDD) / module specs |

**Level 2** answers: *what are the software subsystems, what data and control flows between them,
and how are requirements allocated to subsystems?*

**Level 3** answers: *what is the internal structure of each unit — data types, algorithms,
state machines, interface contracts, WCET, error handling?*

syscribe models both levels using the same element types (`PartDef`, `PortDef`, `ActionDef`,
`StateDef`, `AttributeDef`). What distinguishes the levels is the `domain: software`,
`mg_layer: logical` (L2) vs `mg_layer: physical` (L3) annotations, and the depth of
elaboration in the body prose and `features:` structure.

Without MagicGrid, you distinguish levels by directory structure and `supertype:` links.
With MagicGrid, L2 elements sit at **W3 (logical subsystems)** and L3 at **S3 (physical
components)** — see Part III.

### 9.2 Level 2 — Software subsystem decomposition

The top-level software architecture is a `PartDef` that owns the subsystem hierarchy.
Each subsystem is itself a `PartDef` with `domain: software`.

**The system-of-interest software block**:

```yaml
---
type: PartDef
name: KernelSoftware
domain: software
asilLevel: D
satisfies:
  - REQ-SYS-KERNEL-001
  - REQ-SYS-KERNEL-002
allocatedTo: Hardware::CortexM33Core
custom_fields:
  mg_cell: W3
  mg_layer: logical
  mg_soi: true
---

Top-level software container for the sabaton-rt kernel. All subsystems
are owned by this block. Allocated to Cortex-M33 Core 0 in the SMP/AMP variants.
```

**Subsystem blocks (children of the system-of-interest)**:

```yaml
---
type: PartDef
name: Scheduler
domain: software
asilLevel: D
supertype: Architecture::Logical::KernelSoftware
satisfies:
  - REQ-SCHED-001
  - REQ-SCHED-002
  - REQ-SCHED-003
features:
  - name: readyQueue
    type: Port
    typedBy: Interfaces::ReadyQueuePortDef
    direction: inout
  - name: tickInput
    type: Port
    typedBy: Interfaces::TickPortDef
    direction: in
custom_fields:
  mg_cell: W3
  mg_layer: logical
---

Priority-preemptive scheduler subsystem. Maintains one heapless bitmap-indexed
ready queue per priority level. Scheduling decision is O(1) in thread count
(REQ-SCHED-WCET-001). Context-switch initiation via the `contextSwitch` port.
```

Model every significant subsystem this way:

| Subsystem PartDef | Responsibilities | Requires |
|---|---|---|
| `Scheduler` | Priority ordering, ready queue, preemption | `REQ-SCHED-*` |
| `IpcLayer` | Queues, semaphores, mutexes, event flags | `REQ-IPC-*` |
| `MemoryManager` | TCB pool, byte pool, block pool | `REQ-MEM-*` |
| `SafetyLayer` | `safety_assert!`, reaction callback, watchdog tickle | `REQ-SAFETY-*` |
| `TimerSubsystem` | Software timers, tick accounting | `REQ-TIMER-*` |
| `MpuManager` | Per-thread MPU domain programming | `REQ-MPU-*` |
| `PortAdapter` | Hardware abstraction (PortVTable call-out) | `REQ-PORT-*` |

### 9.3 Level 2 — Interface definitions between subsystems

Interfaces are the **contracts** between subsystems. They are the most important
architectural artefact at L2 for safety: a misspecified interface is the single largest
source of integration defects.

**Defining a port type** (`PortDef`):

```yaml
---
type: PortDef
name: ReadyQueuePortDef
conjugates: Architecture::Interfaces::ReadyQueuePortDef
---

The ready-queue interface. Exported by the Scheduler; consumed by the PortAdapter
when building the initial stack frame and by the IPC layer when unblocking threads.
Operations: enqueue(priority, tcb_ptr), dequeue() → tcb_ptr, peek() → tcb_ptr.
```

**Defining a compatible port pair** (`InterfaceDef`):

An `InterfaceDef` is used when two specific port types always appear together and
you want to document the pairing as a reusable contract:

```yaml
---
type: InterfaceDef
name: SchedulerIpcInterfaceDef
ends:
  - name: scheduler
    typedBy: Architecture::Interfaces::ReadyQueuePortDef
    direction: out
  - name: ipcLayer
    typedBy: Architecture::Interfaces::ReadyQueuePortDef
    direction: in
    isConjugated: true
---

The contract between Scheduler and IpcLayer. The IPC layer calls into the
scheduler to enqueue a newly unblocked thread; the scheduler provides the
`dequeue` operation for the context-switch path.
```

**Wiring subsystems together** (connections on the parent block):

```yaml
---
type: PartDef
name: KernelSoftware
connections:
  - from: scheduler.readyQueue
    to: ipcLayer.readyQueueOut
    typedBy: Architecture::Interfaces::SchedulerIpcInterfaceDef
  - from: scheduler.tickInput
    to: portAdapter.tickOut
  - from: safetyLayer.watchdogTickle
    to: portAdapter.watchdogIn
---
```

This is the L2 Internal Block Diagram equivalent — the full connectivity of the
system expressed as a structured list rather than a graphical notation. It is
unambiguous, diff-able, and validated by the tool.

**Interface inventory command**:

```bash
syscribe -m model list PortDef Architecture::Interfaces
syscribe -m model list InterfaceDef Architecture::Interfaces
syscribe -m model links Architecture::Logical::Scheduler   # all inbound/outbound edges
```

### 9.4 Level 2 — Data flows

Data flows model what *information* moves between subsystems, not just that a connection
exists. Use `ItemDef` for the information type and `FlowDef` for the connection carrying it.

**Defining an item type** (the data flowing):

```yaml
---
type: ItemDef
name: ThreadControlBlockRef
domain: software
---

A reference (pointer or index) to a Thread Control Block. The item that flows
on the ready-queue interface. Safety property: integrity — a corrupted TCB ref
is the primary failure mode in FTE-KERNEL-101.
```

```yaml
---
type: ItemDef
name: TickEvent
domain: software
---

A tick event signalling the passage of one scheduler quantum. Sourced by the
SysTick ISR via the PortAdapter; consumed by the Scheduler (WCET budgeting)
and TimerSubsystem (software timer expiry check).
```

**Flow connections on the parent block**:

```yaml
---
type: PartDef
name: KernelSoftware
flowConnections:
  - from: portAdapter.tickOut
    to: scheduler.tickInput
    itemType: Architecture::Data::TickEvent
  - from: ipcLayer.unblockOut
    to: scheduler.enqueueIn
    itemType: Architecture::Data::ThreadControlBlockRef
---
```

The data flow diagram is derivable from `flowConnections:` — use `connectivity` to
visualise it:

```bash
syscribe -m model connectivity Architecture::Logical::KernelSoftware --depth 2 --format dot
# Pipe to graphviz: | dot -Tsvg > docs/diagrams/l2-data-flow.svg
```

### 9.5 Level 2 — Behavioral overview

At L2 you document the **sequence of major operations** each subsystem performs —
enough to understand the timing model and identify resource contention, but not the
internal algorithm.

**Subsystem function allocation** (`ActionDef` at L2):

```yaml
---
type: ActionDef
name: HandleContextSwitch
refines:
  - REQ-SCHED-001
allocatedTo: Architecture::Logical::Scheduler
custom_fields:
  mg_cell: W2
---

Invoked by PendSV ISR (via PortAdapter). Saves outgoing thread registers to
its TCB stack frame; selects highest-priority ready thread via `dequeue()`;
restores incoming thread registers from its TCB. Total WCET bounded by
REQ-SCHED-WCET-001.
```

**High-level state machine** (thread lifecycle at L2):

```yaml
---
type: StateDef
name: ThreadLifecycle
---

Thread state machine — the subsystem-level view. Unit-level transitions
(internal scheduler queue manipulations) are at L3.

  subStates:
    - name: Ready
      transitions:
        - trigger: ContextSwitchIn
          target: Running
    - name: Running
      transitions:
        - trigger: ContextSwitchOut
          target: Ready
        - trigger: BlockingCall
          target: Blocked
        - trigger: Terminate
          target: Terminated
    - name: Blocked
      transitions:
        - trigger: Unblock
          target: Ready
    - name: Terminated
```

Wire this to the subsystem:

```yaml
# on the Scheduler PartDef:
exhibitsStates:
  - Architecture::Behavior::ThreadLifecycle
```

### 9.6 Level 2 — ASIL allocation and freedom from interference

The L2 architecture document must explicitly allocate integrity levels to subsystems
and demonstrate freedom from interference (FFI) between subsystems at different integrity
levels (ISO 26262-6 §7.4.15 / IEC 61508-3 §7.4.3.6).

**ASIL allocation on subsystems**:

Set `asilLevel:` (or `silLevel:`) on every L2 subsystem PartDef. The validator checks
that a subsystem satisfying an ASIL D requirement itself carries ASIL D (E841).

**Documenting ASIL decomposition**:

When a high-integrity function is decomposed into two independent lower-integrity
implementations, use an `ADR` to document the independence argument:

```yaml
---
type: ADR
id: ADR-ASIL-DECOMP-001
name: "ASIL D decomposed as ASIL B + ASIL B for stack limit enforcement"
status: accepted
---

## Context

REQ-STKLIM-001 (ASIL D) requires stack overflow detection. ARMv8-M provides
PSPLIM (hardware, ASIL B channel) and the watchdog deadman (firmware, ASIL B
channel). Both channels are independent: the PSPLIM failure mode (FTE-KERNEL-103)
and the watchdog failure mode (FTE-KERNEL-104) have no shared cause.

## Decision

Decompose ASIL D → ASIL B (channel A: PSPLIM) + ASIL B (channel B: watchdog),
per ISO 26262-9 §5. Each channel satisfies one half of the original ASIL D demand.

## Consequences

Each channel must be verified independently. FTE-KERNEL-103 and FTE-KERNEL-104
must remain independent (no common-cause failure path).
```

**FFI argument via Allocation elements**:

When two subsystems at different integrity levels share a resource, document the
FFI argument as a standalone `Allocation` element (not just the lightweight field):

```yaml
---
type: Allocation
name: SafetyLayerToTimerSubsystem
allocatedFrom: Architecture::Logical::SafetyLayer
allocatedTo: Architecture::Logical::TimerSubsystem
---

SafetyLayer (ASIL D) calls into TimerSubsystem (ASIL B) for watchdog tickle.
FFI argument: the call is unidirectional (SafetyLayer → TimerSubsystem); the
TimerSubsystem cannot corrupt SafetyLayer data. Spatial isolation enforced by
MPU domain (REQ-MPU-DOMAIN-001). Temporal isolation enforced by PSPLIM.
See ARG-KERNEL-403 for the full freedom-from-interference argument.
```

### 9.7 Level 2 — Work product checklist (ISO 26262-6 §7 / IEC 61508-3 §7.4.3)

Use this to verify your L2 model is complete before moving to L3:

| Work product | syscribe element(s) | Check |
|---|---|---|
| Software subsystem decomposition | `PartDef` (one per subsystem, `domain: software`) | `list PartDef --domain software` |
| Interface specifications (between subsystems) | `PortDef`, `InterfaceDef`, `connections:` | `list PortDef` + `list InterfaceDef` |
| Data flow | `ItemDef`, `flowConnections:` | `connectivity` + review `flowConnections` |
| Control flow / scheduling model | `ActionDef` (`mg_cell: W2`), `successionConnections:` | `list ActionDef` |
| High-level state machines | `StateDef` on each stateful subsystem | `list StateDef` |
| Requirement allocation to subsystems | `satisfies:` on each `PartDef` | `syscribe matrix` — gaps = W300 |
| ASIL/SIL assignment to subsystems | `asilLevel:`/`silLevel:` on each subsystem | `audit` reports distribution |
| ASIL decomposition rationale | `ADR` per decomposition | `list ADR --status accepted` |
| Resource allocation (CPU, memory) | `Allocation` elements; body documents footprint | `matrix --allocations` |
| Freedom from interference argument | `Allocation` with FFI body; `Argument` (ARG-*) | `safety-case` |
| Error detection and handling overview | `ActionDef` bodies; `AOU-*` for integrator obligations | `list AssumptionOfUse` |

```bash
# Run the L2 completeness check
syscribe -m model matrix                          # W300 = subsystem not satisfying any req
syscribe -m model list PartDef --domain software  # inventory of SW subsystems
syscribe -m model matrix --allocations            # function → subsystem allocation gaps
```

---

### 9.8 Level 3 — Software unit decomposition

Level 3 decomposes each L2 subsystem into **software units** — the individually testable
modules that directly correspond to source files or compilation units. Each unit is a
`PartDef` with `supertype:` pointing to its L2 parent.

```yaml
---
type: PartDef
name: ReadyQueue
domain: software
supertype: Architecture::Logical::Scheduler     # L2 parent
asilLevel: D
custom_fields:
  mg_cell: S3
  mg_layer: physical
satisfies:
  - REQ-SCHED-001
implementedBy: kernel/src/scheduler/ready_queue.rs
---

The bitmap-indexed, fixed-size ready queue. Provides O(1) `enqueue`, `dequeue`,
and `peek_highest` operations. No heap allocation — all state is in the statically
allocated `ReadyQueueHarness` struct.

**Resource usage**: 8 bytes per priority level (32-bit bitmap + pointer).
At 32 priority levels: 256 bytes in `.bss`.
**Stack depth**: 4 words (enqueue); 3 words (dequeue/peek). No recursion.
```

```yaml
---
type: PartDef
name: ContextSwitchEngine
domain: software
supertype: Architecture::Logical::Scheduler
asilLevel: D
custom_fields:
  mg_cell: S3
  mg_layer: physical
satisfies:
  - REQ-SCHED-WCET-001
implementedBy: ports/cortex-m/src/context_switch.rs
---

The PendSV handler and register save/restore sequence. Implemented in naked
assembly with an explicit stack frame layout documented in REQ-PORT-FRAME-001.
```

The `implementedBy:` field references the source file(s) that realise the element.
A non-`draft` element whose **local** `implementedBy:` path does not exist on disk
raises warning **W023** (opt-in, draft-suppressed; gate with `validate --deny W023`);
remote URIs are accepted as external pointers. The link makes the path from design to
code explicit for reviewers and auditors to follow.

### 9.9 Level 3 — Detailed interface specification

At L3, interface contracts become precise enough to drive unit tests. Each `PortDef`
at L3 carries the exact function signatures and pre/post-conditions.

```yaml
---
type: PortDef
name: ReadyQueueEnqueuePortDef
---

**Operation**: `enqueue(priority: u8, tcb: *mut Tcb) -> Result<(), QueueFull>`

**Preconditions**:
  - `priority` ∈ [0, MAX_PRIORITY)  (else: panic in debug, UB in release — AOU)
  - `tcb` is non-null and aligned to `align_of::<Tcb>()`
  - Caller holds the scheduler critical section (not called from thread context)

**Postconditions**:
  - `peek_highest()` returns a thread with priority ≥ the enqueued thread's priority
  - The enqueued thread is reachable by `dequeue()` after any number of higher-priority
    `enqueue` / `dequeue` operations

**Error**: `QueueFull` when the priority level's slot is already occupied (this is a
design invariant violation — `safety_assert!` triggers the safety reaction).

**WCET**: O(1) — one bitmap write + one linked-list insert; ≤ 12 cycles at 120 MHz
(measured in TC-SCHED-WCET-001).
```

Map a concrete port to this definition on the unit:

```yaml
# on ReadyQueue PartDef:
features:
  - name: enqueuePort
    type: Port
    typedBy: Architecture::Interfaces::L3::ReadyQueueEnqueuePortDef
    direction: out
```

### 9.10 Level 3 — Data structure definitions

Data structures live at L3. Use `AttributeDef` for typed scalar/composite fields and
`EnumerationDef` for discrete state types.

**Thread state enumeration**:

```yaml
---
type: EnumerationDef
name: ThreadState
values:
  - name: Ready
    description: "Thread is in the ready queue, eligible for scheduling"
  - name: Running
    description: "Thread holds the CPU — exactly one thread per core"
  - name: Blocked
    description: "Thread is waiting on an IPC object or timer"
  - name: Suspended
    description: "Thread is suspended by the application"
  - name: Terminated
    description: "Thread has returned from its entry function"
---
```

**Thread Control Block data layout**:

```yaml
---
type: PartDef
name: ThreadControlBlock
domain: software
asilLevel: D
custom_fields:
  mg_cell: S3
  mg_layer: physical
features:
  - name: savedSp
    type: Attribute
    typedBy: Architecture::Data::StackPointer
    description: "Saved stack pointer; valid when state ≠ Running"
  - name: priority
    type: Attribute
    typedBy: Architecture::Data::Priority
    description: "Static priority in [0, MAX_PRIORITY); immutable after creation"
  - name: state
    type: Attribute
    typedBy: Architecture::Data::ThreadState
    description: "Current lifecycle state — transitions per ThreadLifecycle state machine"
  - name: stackBase
    type: Attribute
    typedBy: Architecture::Data::StackPointer
    description: "Base of the thread stack; used for PSPLIM initialisation"
  - name: stackSize
    type: Attribute
    typedBy: Architecture::Data::ByteCount
    description: "Size in bytes; used for MPU region calculation"
  - name: canary
    type: Attribute
    typedBy: Architecture::Data::U32
    description: "Stack canary at the top of the stack (lowest address); checked by SC-KERNEL-004"
  - name: nextReady
    type: Attribute
    typedBy: Architecture::Data::TcbRef
    description: "Intrusive linked-list link within the ready queue; null if not ready"
---

The Thread Control Block is the central data structure. Integrity of this struct is
ASIL D — corruption is the failure mode in FTE-KERNEL-101 (stack overflow), FTE-KERNEL-201
(MPU misconfiguration). The `canary` field is the primary detection mechanism (SC-KERNEL-004).
```

**Why this matters for safety**: the TCB layout document is the L3 artefact that your FMEA
(`FMEAEntry` referencing `component: ThreadControlBlock`) and FaultTree (`FTE-KERNEL-101`)
point to. A reviewer auditing the safety case can follow the chain:
SafetyGoal → FaultTree → FaultTreeEvent → TCB PartDef → canary field → TestCase.

### 9.11 Level 3 — Algorithms and control flow

`ActionDef` at L3 documents the algorithm for a specific unit operation. Include
sub-actions (the steps), control nodes (decision points), and the succession ordering.

```yaml
---
type: ActionDef
name: ScheduleNext
refines:
  - REQ-SCHED-001
allocatedTo: Architecture::Physical::ReadyQueue
custom_fields:
  mg_cell: S3
parameters:
  - name: currentThread
    type: Architecture::Data::TcbRef
    direction: in
subActions:
  - name: SaveContext
    description: "Save {r4-r11, lr} to currentThread.savedSp via PendSV exception frame"
    succession: [SelectNext]
  - name: SelectNext
    description: "Peek highest-priority non-empty ready queue level; O(1) CLZ on bitmap"
    succession: [LoadContext]
  - name: LoadContext
    description: "Restore {r4-r11, lr} from selectedThread.savedSp; update PSP"
controlNodes:
  - name: SameThread
    kind: decision
    description: "If selectedThread == currentThread, skip save/load (no-op switch)"
---

**Error path**: if the ready queue bitmap is zero (no ready thread), the kernel panics via
`safety_assert!(false, IDLE_CORRUPTION)` — this should never occur because the idle thread
is always ready.
```

### 9.12 Level 3 — Detailed state machines

L3 state machines are the unit-level refinement of the L2 thread lifecycle. They add
guard conditions, entry/exit actions, and internal transitions.

```yaml
---
type: StateDef
name: SchedulerUnitStateMachine
---

Scheduler state machine — unit level. Refines Architecture::Behavior::ThreadLifecycle.

  subStates:
    - name: Ready
      entryAction: "ready_queue.enqueue(self.priority, self)"
      exitAction: "ready_queue.remove(self)"
      transitions:
        - trigger: "dispatch()"
          guard: "self.priority == ready_queue.peek_highest().priority"
          target: Running
          action: "cpu.set_psp(self.saved_sp)"

    - name: Running
      entryAction: "CORE_LOCAL.current_thread = self"
      exitAction: "self.saved_sp = cpu.get_psp()"
      transitions:
        - trigger: "preempt(higher_prio)"
          target: Ready
          action: "pend_context_switch()"
        - trigger: "ipc_block(object)"
          target: Blocked
          action: "object.waitlist.push(self)"
        - trigger: "thread_return()"
          target: Terminated
          action: "thread_return_hook(); port.terminate()"

    - name: Blocked
      transitions:
        - trigger: "unblock()"
          target: Ready
        - trigger: "timeout()"
          guard: "self.timeout_ticks == 0"
          target: Ready
          action: "self.ipc_result = Timeout"

    - name: Terminated
      entryAction: "THREAD_POOL.release(self)"
```

Connect the state machine to the unit:

```yaml
# on the ReadyQueue or Scheduler PartDef at L3:
exhibitsStates:
  - Architecture::Behavior::SchedulerUnitStateMachine
```

### 9.13 Level 3 — WCET and resource documentation

**WCET on requirements**:

Every L3 safety requirement at SIL/ASIL level should carry a `wcet:` claim:

```yaml
---
type: Requirement
id: REQ-SCHED-WCET-001
name: "Scheduler context-switch shall complete within bounded WCET"
status: approved
asilLevel: D
wcet: "≤ 200 cycles at 120 MHz on Cortex-M33 (< 1.7 µs) measured at L5"
reqDomain: software
verificationMethod: test
---
```

Find all requirements that declare a WCET bound:

```bash
syscribe -m model list Requirement --has-wcet
```

Find SIL/ASIL requirements that *lack* a WCET declaration (a safety gap):

```bash
# All ASIL-D requirements; compare to --has-wcet output for the delta
syscribe -m model list Requirement --sil D
```

**WCET on PartDef (component budget)**:

Document the WCET budget on the component itself, separate from the requirement:

```yaml
---
type: PartDef
name: ReadyQueue
custom_fields:
  wcet_enqueue: "≤ 6 cycles (CLZ + 2× list ops)"
  wcet_dequeue: "≤ 8 cycles (CLZ + list dequeue + bitmap clear)"
  wcet_peek:    "≤ 3 cycles (CLZ only)"
---
```

Use `custom_fields:` for component-level timing budgets (syscribe does not parse these;
they are documentation for the engineer and auditor).

**Stack depth documentation**:

For `no_std` / embedded software where stack overflows are a primary failure mode,
document maximum call depth and stack consumption per unit in the PartDef body.
This directly feeds the PSPLIM/MPU domain configuration documented in REQ-MPU-DOMAIN-001.

### 9.14 Level 3 — Error detection and handling

Each L3 unit must document how it detects and handles errors. In a safety context,
error detection IS a safety mechanism — record it explicitly so it appears in the FMEA.

```yaml
---
type: ActionDef
name: HandleEnqueueError
---

Error handling for `ReadyQueue::enqueue` invariant violations.

**Detected conditions**:
- Priority out of range → `safety_assert!(priority < MAX_PRIORITY, PRIORITY_RANGE)`
- Double-enqueue (thread already ready) → `safety_assert!(!self.is_ready(), DOUBLE_ENQUEUE)`
- Null TCB pointer → `safety_assert!(!tcb.is_null(), NULL_TCB)`

**Safety reaction**: `safety_assert!` invokes the registered safety reaction callback
(see SafetyLayer; REQ-SAFETY-REACT-001). If no callback is registered, the system
panics via the port's default handler.

**Relationship to FMEA**: FM-KERNEL-001 (ready-queue pointer corruption) is caught
post-hoc by the canary check. The `safety_assert!` checks above catch precondition
violations at the call site — they are the primary detection mechanism (SC-KERNEL-001)
that gives FM-KERNEL-001 its DC = 0.93.
```

### 9.15 L2 → L3 traceability

The standard traceability chain from L2 to L3 uses `supertype:` (the L3 unit is a
specialisation of its L2 parent subsystem). This is automatically followed by the
`connectivity` and `who-verifies` commands.

```
Architecture::Logical::Scheduler          ← L2 subsystem PartDef
    ← supertype: on →
Architecture::Physical::ReadyQueue        ← L3 unit PartDef
Architecture::Physical::ContextSwitchEngine
Architecture::Physical::ThreadControlBlock
```

Confirm the chain:

```bash
# Show everything reachable from the L2 scheduler subsystem (2 hops)
syscribe -m model connectivity Architecture::Logical::Scheduler --depth 2

# What requirements does the L3 unit satisfy? (transitive via supertype)
syscribe -m model why Architecture::Physical::ReadyQueue
```

The `links` command shows both the structural chain and the requirement traces:

```bash
syscribe -m model links Architecture::Physical::ReadyQueue
# Outbound: satisfies REQ-SCHED-001, supertype Logical::Scheduler
# Inbound:  verifies TC-SCHED-001 (via REQ-SCHED-001)
```

### 9.16 Directory layout for L2 and L3

Separate L2 and L3 into distinct directories. This makes `appliesWhen:` package-level
gating clean and makes it obvious to a reviewer which level they are looking at:

```
model/
  Architecture/
    _index.md                      ← type: Package
    Logical/                       ← L2 subsystems
      _index.md
      KernelSoftware.md
      Scheduler.md
      IpcLayer.md
      MemoryManager.md
      SafetyLayer.md
    Physical/                      ← L3 units
      _index.md
      ReadyQueue.md
      ContextSwitchEngine.md
      ThreadControlBlock.md
      Semaphore.md
      BytePool.md
    Interfaces/
      _index.md
      ReadyQueuePortDef.md         ← L2-level port contracts
      L3/
        _index.md
        ReadyQueueEnqueuePortDef.md  ← L3 precise interface specs
    Behavior/
      _index.md
      ThreadLifecycle.md           ← L2 state machine
      SchedulerUnitStateMachine.md ← L3 state machine
    Data/
      _index.md
      ThreadState.md               ← EnumerationDef
      ThreadControlBlock.md        ← AttributeDef layout (if separate from PartDef)
    Decisions/
      _index.md
      ADR-SCHED-001.md
      ADR-ASIL-DECOMP-001.md
```

### 9.17 Key commands for software architecture work

```bash
# Inventory L2 subsystems and L3 units
syscribe -m model list PartDef Architecture::Logical     # L2 subsystems
syscribe -m model list PartDef Architecture::Physical    # L3 units

# Interface and data structure inventory
syscribe -m model list PortDef Architecture::Interfaces
syscribe -m model list InterfaceDef Architecture::Interfaces
syscribe -m model list StateDef Architecture::Behavior
syscribe -m model list ActionDef Architecture::Behavior
syscribe -m model list EnumerationDef Architecture::Data

# Allocation and connectivity
syscribe -m model matrix --allocations                   # function → L2 → L3 chain
syscribe -m model connectivity Architecture::Logical::Scheduler --depth 3 --format dot

# Coverage — which L3 units satisfy no requirements?
syscribe -m model matrix                                 # W300 = unsatisfied leaf requirement
syscribe -m model list PartDef --domain software         # cross-reference with matrix gaps

# WCET coverage
syscribe -m model list Requirement --has-wcet            # reqs with WCET claims
syscribe -m model list Requirement --sil 4               # all SIL4 reqs (compare for gaps)

# Traceability from L3 unit to test evidence
syscribe -m model why Architecture::Physical::ReadyQueue
syscribe -m model links Architecture::Physical::ReadyQueue

# Safety case connection
syscribe -m model safety-case SG-KERNEL-001              # GSN tree — which L3 units appear?
```

### 9.18 Connecting L2/L3 to the safety analysis

The software architecture at L2 and L3 is the *subject* of the fault tree and FMEA — every
`FaultTreeEvent` and `FMEAEntry` should be traceable to a specific L3 unit.

**FaultTreeEvent → L3 unit**:

In the FaultTreeEvent body, cite the component:

```
FM-KERNEL-001 (FMEA). Component: Architecture::Physical::ReadyQueue.
Failure mode: corruption of nextReady pointer via stack overflow into TCB.
```

**FMEAEntry → L3 unit**:

```yaml
entries:
  - id: FM-KERNEL-001
    component: Architecture::Physical::ThreadControlBlock   # QName or display name
    failureMode: "nextReady pointer corruption"
    faultTreeRef: FTE-KERNEL-101
```

This creates a complete chain an assessor can walk:
SafetyGoal → FaultTree → FaultTreeEvent → L3 PartDef → source file → TestCase → result.

---
