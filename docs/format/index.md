# Format Overview

`FORMAT · OVERVIEW`

Syscribe is a plain-text representation of SysMLv2 models. Each model element is a `.md` file. The directory tree encodes namespace hierarchy. YAML frontmatter declares the element's type and metadata. The Markdown body is the element's documentation (`doc` annotation in SysML terms).

## Core idea

```
model/
  _index.md                    # root namespace metadata
  UAV/
    _index.md                  # UAV package
    UAVSystem.md               # part def UAVSystem
    Avionics/
      FlightController.md      # part def FlightController
  Requirements/
    FaultTolerantFCReq.md      # native Requirement REQ-UAV-FC-001
  Verification/
    FCFaultInjectionTest.md    # native TestCase TC-UAV-FC-001
```

A file at `model/UAV/Avionics/FlightController.md` has qualified name `UAV::Avionics::FlightController`. Cross-references use `::` notation throughout.

## Frontmatter + body

```yaml
---
type: PartDef                          # element type — required
name: FlightController                 # display name — defaults to filename stem
supertype: Parts::Part                 # specialisation
domain: software                       # traceability domain
satisfies:
  - REQ-UAV-FC-001                     # resolved by stable ID
  - REQ-UAV-SAFE-001
features:
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: in
---

Flight computer executing attitude control, sensor fusion, and fault detection.
Runs on a dual-core ARM Cortex-A53 at 1.2 GHz with ECC SRAM.
```

## Three kinds of elements

| Kind | ID format | Key fields |
|---|---|---|
| **SysML elements** | Path-derived qualified name | `type`, `supertype`, `features`, `connections` |
| **Native Requirements** | `REQ-*` stable ID | `id`, `title`, `status`, `reqDomain`, `silLevel`, `derivedFrom` |
| **Native TestCases** | `TC-*` stable ID | `id`, `title`, `status`, `testLevel`, `verifies`, Gherkin body |
| **ADRs** | `ADR-*` stable ID | `id`, `title`, `status` (proposed/accepted/deprecated/superseded) |

## What makes it different

- **No binary files.** Every element is a readable `.md` file. `git diff` works normally.
- **LLM-friendly.** Language models can read, write, and reason about models without special tooling.
- **Validation built-in.** The Rust engine checks cross-references, domain rules, Gherkin syntax, and 80+ other rules at every `cargo run`.
- **Diagrams in frontmatter.** BDD/IBD/StateMachine shapes and edges are declared as YAML, laid out once, and rendered as SVG by the web server.

## Further reading

- [Element Types](elements.md) — the complete type table
- [Frontmatter Reference](frontmatter.md) — every common field documented
- [Diagrams](diagrams.md) — all diagram kinds and their frontmatter schema
- [Operations](operations.md) — synchronous operations and async receptions on ports and interfaces
