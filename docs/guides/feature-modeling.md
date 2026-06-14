# Part X — Feature Modeling and the 150% Product Line Model

`GUIDE · PART X — FEATURE MODELING`

### 10.1 What the 150% model is

In Product Line Engineering (PLE), a **150% model** is a single model that contains
*all* requirements, architecture elements, tests, and analysis artefacts for *all*
possible product variants simultaneously — including elements that are mutually exclusive
and can never appear together in any real product. The "150%" label reflects the fact
that the model is deliberately *larger* than any single product; it is the superset.

From this superset, you derive a **100% model** — the exact set of elements active in a
specific product variant — by applying a **Configuration**: a named feature selection
that resolves every `appliesWhen:` expression and keeps only the elements whose
predicate holds true.

Traditional PLE tools (PTC Integrity Modeler, pure::variants, Gears) implement this
as a feature model + a configuration engine. syscribe does the same thing in plain text
and git:

| PLE tool concept | syscribe equivalent |
|---|---|
| Feature model | `FeatureDef` tree |
| Feature node (optional/mandatory/alternative) | `FeatureDef` with `groupKind:` |
| Feature constraint (requires/excludes) | `requires:` / `excludes:` on `FeatureDef` |
| Product configuration | `Configuration` with `features:` map |
| Variant element | Any element with `appliesWhen:` |
| 100% projection | `syscribe -m model validate --config CONF-X` |
| Configuration SAT check | `syscribe -m model feature-check --deep` |
| Diff between variants | `syscribe -m model diff --config A --config B` |

### 10.2 Defining the feature model

Features are `FeatureDef` elements, typically in a `Features/` package. The hierarchy
reflects the decomposition of variability in your product line.

```yaml
---
type: FeatureDef
id: FEAT-PORT-001
name: Port
groupKind: alternative   # exactly one child must be selected (XOR)
mandatory: true          # always selected (the root feature)
---

The hardware/software port must be exactly one of: CortexM, CortexM33,
RiscV, Hosted (for testing).
```

```yaml
---
type: FeatureDef
id: FEAT-PORT-CM33-001
name: CortexM33
groupKind: alternative   # child of the Port alternative group
requires:
  - Features::MPU          # CortexM33 implies MPU is available
---

ARMv8-M Mainline port. Enables PSPLIM stack-limit and MPU domain features.
```

```yaml
---
type: FeatureDef
id: FEAT-MPU-001
name: MPU
groupKind: optional       # can be present or absent (BooleanOr)
requires:
  - Features::CortexM33   # MPU only makes sense with CortexM33
---

Memory Protection Unit support. When selected, enables REQ-MPU-* and the
per-thread MPU domain mechanism.
```

#### GroupKind values

| `groupKind` | Meaning | Example |
|---|---|---|
| `mandatory` | Always selected when parent is selected | A core feature every product must have |
| `optional` | May be selected or not (boolean) | An add-on feature |
| `alternative` | Exactly one child selected (XOR group) | Target port: Cortex-M3, Cortex-M33, RISC-V |
| `or` | One or more children selected | Communication protocol group |

#### Parameterised features

Features can carry typed parameters — numeric or enum values that vary across configurations
without requiring separate feature nodes:

```yaml
---
type: FeatureDef
id: FEAT-TOPO-001
name: Topology
mandatory: true
parameters:
  - name: maxCpus
    type: integer
    range: [1, 8]
    default: 1
    isRequired: true        # W017 if unbound in any Configuration
---
```

In a `Configuration`, bind the parameter:

```yaml
parameterBindings:
  Features::Topology.maxCpus: 2
```

Cross-feature parameter constraints live in the Package `_index.md`:

```yaml
# model/Features/_index.md
parameterConstraints:
  - id: PC-001
    expression: "Features::Topology.maxCpus > 1 implies Features::Smp or Features::Amp"
    severity: error   # E213 if violated
```

### 10.3 Defining product configurations

A `Configuration` is a feature selection that resolves the 150% model into one product:

```yaml
---
type: Configuration
id: CONF-RP-PICO2-AMP-001
name: "Raspberry Pi Pico 2 (RP2350, dual Cortex-M33) AMP — HIL"
featureModel: Features::Sabaton
features:
  Features::Port: true
  Features::CortexM33: true
  Features::Amp: true
  Features::MPU: true
  Features::StackLimit: true
  Features::Async: true
  Features::Single: false
  Features::HostedPort: false
parameterBindings:
  Features::Topology.maxCpus: 2
---

AMP configuration: Core 0 runs the kernel; Core 1 runs a safety monitor
thread under AMP isolation. Uses the RP2350 dual-core hardware.
```

Validate this configuration in isolation:

```bash
syscribe -m model validate --config CONF-RP-PICO2-AMP-001
```

This projects the 150% model onto this product, runs every `appliesWhen:` predicate, and
checks:
- E226: a cross-reference that resolves globally but not in this config (an element
  references something that is `appliesWhen:` gated off)
- W019: a traceability link that escapes the configuration boundary
- W015: a requirement active in this config with no test case running in it

### 10.4 Gating elements with `appliesWhen:`

Any element — requirements, architecture parts, test cases, safety artefacts — can carry
`appliesWhen:` to restrict it to specific variants. The expression uses FeatureDef QNames
with `and` / `or` / `not` / parentheses.

**On a Requirement**:

```yaml
---
type: Requirement
id: REQ-MPU-DOMAIN-001
name: "Kernel shall enforce per-thread MPU domain isolation"
status: approved
asilLevel: D
appliesWhen: Features::MPU and Features::CortexM33
---
```

This requirement only appears in the coverage matrix for configurations where both
`MPU` and `CortexM33` are selected. It is `—` (N/A) for all others.

**On an Architecture element**:

```yaml
---
type: PartDef
name: MpuDomainManager
domain: software
appliesWhen: Features::MPU
satisfies:
  - REQ-MPU-DOMAIN-001
---
```

**On a TestCase**:

```yaml
---
type: TestCase
id: TC-MPU-FAULT-HIL-001
name: "MPU read-only region write causes MemManage trap on hardware"
testLevel: L5
appliesWhen: Features::MPU and Features::CortexM33
verifies: [REQ-MPU-FAULT-001]
---
```

**On an entire Package** (gates the whole subtree):

```yaml
# model/Requirements/MPU/_index.md
---
type: Package
name: MPU
appliesWhen: Features::MPU
---
```

Every requirement in `model/Requirements/MPU/` is automatically gated on `Features::MPU`.
One declaration per path (E228 if a parent already gates the subtree); empty gated packages
warn W026.

### 10.5 Feature model integrity checks

Before depending on the feature model for certification, validate it:

```bash
syscribe -m model feature-check               # constraints, orphans, dead features
syscribe -m model feature-check --deep        # SAT-backed: void model, dead/core/
                                              # false-optional features, full semantics
syscribe -m model feature-check --count       # number of valid configurations
syscribe -m model feature-check --enumerate   # list all valid configurations
```

`feature-check --deep` uses SAT analysis:
- **void model**: no valid configuration exists (all features conflict)
- **dead feature**: a feature that can never be selected in any valid configuration
- **core feature**: always selected in every valid configuration (may as well be mandatory)
- **false-optional**: declared optional but forced true by constraints

These findings surface design errors that are impossible to spot manually in a large feature
model.

### 10.6 Understanding variant differences

Compare two configurations to see what changes:

```bash
# Elements active in AMP but not in single-core QEMU (and vice versa)
syscribe -m model diff --config CONF-RP-PICO2-AMP-001 --config CONF-LM3S-QEMU
```

Explain why a specific element is active in a configuration:

```bash
syscribe -m model why-active REQ-MPU-DOMAIN-001 --config CONF-RP-PICO2-HIL
```

Output:
```
Element: Requirements::MPU::MpuDomain::REQ-MPU-DOMAIN-001
appliesWhen: Features::MPU and Features::CortexM33
  Features::MPU → selected: true
  Features::CortexM33 → selected: true
Verdict: active
```

```bash
syscribe -m model why-active REQ-MPU-DOMAIN-001 --config CONF-LM3S-QEMU
# Verdict: inactive (Features::MPU → selected: false)
```

### 10.7 Coverage matrix across the product line

The `matrix` command shows requirement coverage across *all* configurations simultaneously.
This is the 150% model's "projection verification" view — every row is a requirement, every
column is a configuration, every cell shows whether that requirement is covered (✓), gapped
(✗), or not applicable in that variant (—):

```bash
syscribe -m model matrix                      # full 150% matrix (all reqs × all configs)
syscribe -m model matrix --gaps-only          # only rows with at least one gap
syscribe -m model matrix --status approved    # only approved requirements
syscribe -m model matrix --tag safety         # only safety-tagged requirements
syscribe -m model matrix --features           # feature × configuration selection grid
```

**Interpreting the matrix for product line certification**:

A requirement that shows `—` in all configurations is `appliesWhen:` gated to features that
no configuration currently selects — it is dead in your current product line. Either add a
configuration that selects it or remove it from the 150% model.

A requirement that shows `✗` in one configuration but `✓` in another typically means the
verification is present but not yet connected (`appliesWhen:` on the test case may need
extending to that configuration).

### 10.8 Safety analysis in a product line

The 150% model approach is especially valuable for safety work, where the cost of separate
analyses per variant is very high.

**Approach**: author the HARA, SafetyGoals, FaultTrees, and FMEAs once in the 150% model.
Gate variant-specific elements with `appliesWhen:`. Safety goals that apply across the whole
product line have no `appliesWhen:`; variant-specific ones do.

```yaml
# SafetyGoal that applies to all variants
---
type: SafetyGoal
id: SG-KERNEL-001
name: "Kernel scheduler shall not starve any safety-relevant thread"
status: approved
asilLevel: D
---

# SafetyGoal specific to AMP variants only
---
type: SafetyGoal
id: SG-KERNEL-006
name: "AMP core isolation shall prevent cross-core interference"
status: approved
asilLevel: D
appliesWhen: Features::Amp
---
```

**FaultTree gating**: if a FaultTree analysis changes between variants (e.g., the MPU
hardware fault path only exists in configurations with `Features::MPU`), gate the relevant
`FaultTreeEvent` elements:

```yaml
---
type: FaultTreeEvent
id: FTE-KERNEL-101
name: "Stack overflow corrupts TCB"
eventKind: basic
failureRate: 5.0e-9
diagnosticCoverage: 0.97    # high DC because PSPLIM is available
latentDiagnosticCoverage: 0.75
appliesWhen: Features::StackLimit
---

```yaml
---
type: FaultTreeEvent
id: FTE-KERNEL-102
name: "Stack overflow undetected (no stack-limit hardware)"
eventKind: basic
failureRate: 5.0e-9
diagnosticCoverage: 0.30    # low DC — software-only detection
latentDiagnosticCoverage: 0.15
appliesWhen: not Features::StackLimit
---
```

When you run `syscribe metrics --config CONF-RP-PICO2-HIL`, FTE-KERNEL-101 is used
(PSPLIM available, high DC). When you run `syscribe metrics --config CONF-LM3S-QEMU`,
FTE-KERNEL-102 is used instead (no PSPLIM, lower DC). The PMHF computation is
automatically configuration-aware.

**ASIL decomposition across variants**: if variant A decomposes an ASIL D requirement
into two ASIL B requirements implemented on independent channels, and variant B uses a
single ASIL D implementation, both strategies can coexist in the 150% model gated by
the appropriate features. The validator checks per-configuration that the ASIL integrity
chain is consistent within that variant (E841–E843).

### 10.9 Certification per variant

A certifiable product variant is a named, baselined configuration. The workflow:

```bash
# Step 1: Validate the specific variant
syscribe -m model validate --config CONF-RP-PICO2-HIL

# Step 2: Check its coverage (every active requirement has test evidence)
syscribe -m model matrix --config CONF-RP-PICO2-HIL --gaps-only

# Step 3: Compute safety metrics for this variant's fault trees
syscribe -m model metrics --config CONF-RP-PICO2-HIL

# Step 4: Run the safety-readiness audit for this variant
syscribe -m model audit --config CONF-RP-PICO2-HIL

# Step 5: Baseline it in git
git tag v1.0-CONF-RP-PICO2-HIL-2026-06-14

# Step 6: Capture the evidence artefacts
syscribe -m model validate --config CONF-RP-PICO2-HIL --json \
  > artifacts/validation-CONF-RP-PICO2-HIL-$(git describe --tags).json
syscribe -m model metrics --config CONF-RP-PICO2-HIL --json \
  > artifacts/metrics-CONF-RP-PICO2-HIL-$(git describe --tags).json
```

The `git tag` is the baseline. The JSON artefacts are the evidence record tied to that
baseline. An auditor can reproduce either by checking out the tagged commit and re-running
the same commands.

### 10.10 Assisted configuration

When a new product variant needs to be defined, the `configure` command helps navigate the
feature model:

```bash
# Start with a partial selection; see what is forced and what remains free
syscribe -m model configure "Features::CortexM33: true, Features::Amp: true"
```

Output tells you:
- Which features are **forced true** by the `requires:` constraints on your selection
- Which features are **forced false** by `excludes:` constraints
- Which features remain **free** for you to choose
- Whether the partial selection is **satisfiable** at all

This replaces the manual constraint-chasing that traditionally requires a domain expert
to work through feature dependency matrices in a spreadsheet.

### 10.11 Trade study across variants (with MagicGrid)

If you have used MagicGrid (Part III), the `trade-study` command evaluates each
`Configuration` against the B4 Measures of Effectiveness and scores them:

```bash
syscribe -m model trade-study
```

For product line decisions — "which variant architecture best meets the stakeholder needs?"
— this provides a structured, traceable answer grounded in the W4/S4 parameter bindings
of each configuration, rather than in an informal spreadsheet.

```yaml
# A Configuration as a MagicGrid trade-study variant
---
type: Configuration
id: CONF-CM33-HIGH-PERF
name: "Cortex-M33 high-performance variant"
custom_fields:
  mg_variant: true
featureModel: Features::Sabaton
features:
  Features::CortexM33: true
  Features::MPU: true
parameterBindings:
  Features::Topology.maxCpus: 1
  W4::ContextSwitchWCET.value: 8     # µs, matches MoP constraint
---
```

### 10.12 The 150% model in practice — guidance for new product lines

**Start small**: do not try to capture all variability up front. Begin with a single
configuration (the reference product) and no `appliesWhen:` gates. Add features and
`appliesWhen:` expressions as the second and third variants emerge.

**Feature granularity**: a feature should correspond to a coherent set of requirements,
one or more architecture elements, and a set of tests that live or die together. If
toggling a "feature" requires touching dozens of unrelated files, split it.

**Gate packages, not files**: when an entire domain only applies to one variant family
(e.g., all AMP-specific requirements), gate the whole `_index.md` rather than adding
`appliesWhen:` to every file. This avoids W026 false positives and keeps individual
files clean.

**Safety goal scope**: prefer safety goals with no `appliesWhen:` where the goal is
common to all variants, even if the *mechanism* that satisfies it varies. The mechanism
(PartDef, FaultTreeEvent) gets `appliesWhen:`; the goal does not. This keeps the HARA
and safety case structure stable across the product line.

**Baseline discipline**: every Configuration that ships to a customer must have a git tag
before release. The tag name should encode the Configuration ID and the date. The `audit`
output at that tag is the acceptance record.

---
