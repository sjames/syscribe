# Appendix A — Traditional Tool Mapping

`GUIDE · APPENDIX A — TOOL MAPPING`


### A.1 IBM DOORS / DOORS Next

| DOORS concept | syscribe equivalent |
|---|---|
| Module | Directory of `.md` files |
| Requirement row | `.md` file with `type: Requirement` |
| Attribute | Frontmatter field (`asilLevel:`, `status:`, etc.) |
| Link (satisfies, derived from, verifies) | `derivedFrom:`, `satisfiedBy:`, `verifies:` fields |
| Baseline | Git tag (`git tag v1.0-sil4-2026`) |
| View (filtered) | `list Requirement --status approved --sil 4` |
| Trace matrix | `matrix` command |
| Module export (Word/PDF) | Markdown renders natively in GitHub/GitLab; use pandoc for PDF |
| Change notice | Pull request + review |
| DXL script | `syscribe` CLI (analysis) + shell scripts |

### A.2 Polarion / Jira

| Polarion concept | syscribe equivalent |
|---|---|
| Work item | `.md` file |
| Work item type | `type:` frontmatter field |
| Custom field | Any frontmatter field; `custom_fields:` for MagicGrid |
| Link role | Field names encode role (`derivedFrom:`, `verifies:`, etc.) |
| Branch / variant | `Configuration` + `appliesWhen:` + `--config` projection |
| Report template | `syscribe` CLI command + `--json` for custom scripts |

### A.3 Cameo / MagicGrid

| Cameo / MagicGrid concept | syscribe equivalent |
|---|---|
| SysML Block | `PartDef` |
| SysML Block instance | `Part` |
| SysML Port | `PortDef` / `Port` |
| SysML Requirement | `type: Requirement` |
| MagicGrid cell | `custom_fields: { mg_cell: B1 }` |
| System of interest | `custom_fields: { mg_soi: true }` |
| External actor | `custom_fields: { mg_external: true }` |
| MoE | `CalculationDef` with `mg_moe: true` |
| MoP | `ConstraintDef` with `mg_mop: true` |
| Allocation | `allocatedTo:` field (or standalone `Allocation` element) |
| Internal Block Diagram | `connections:` list on a `PartDef` |
| Trade study | `syscribe trade-study` |

### A.4 Automotive SPICE (ASPICE)

Automotive SPICE (ISO 15504 / VDA Scope) is the process assessment model used across
the automotive supply chain. It defines the expected work products for each process area.
The table below maps the primary ASPICE work products to syscribe elements, showing
where syscribe produces the required evidence.

**SWE.1 — Software Requirements Analysis**

| ASPICE work product | syscribe artefact | Notes |
|---|---|---|
| Software requirements specification | `model/Requirements/**/*.md` | `list Requirement`; body = prose spec |
| Software requirements attributes | Frontmatter fields (`asilLevel:`, `status:`, `verificationMethod:`, `wcet:`) | `list Requirement --status approved` |
| Consistency with system requirements | `derivedFrom:` + `breakdownAdr:` | `trace REQ-X` shows the full chain |
| Bi-directional traceability to system reqs | `derivedFrom:` (down) + `satisfies:` (up) | `who-verifies` + `why` commands |
| SW requirement status | `status: draft|review|approved|verified` | `audit` shows per-package breakdown |

**SWE.2 — Software Architectural Design**

| ASPICE work product | syscribe artefact | Notes |
|---|---|---|
| Software architectural design (SAD) | `Architecture::Logical/**` (L2 PartDefs) | Part IX §9.2–9.7 |
| Software interfaces between components | `PortDef`, `InterfaceDef`, `connections:` | `list PortDef` + `list InterfaceDef` |
| Dynamic behavior description | `StateDef` (L2), `ActionDef` (W2) | `list StateDef`, `list ActionDef` |
| Resource consumption estimates | `PartDef` body + `custom_fields` | `show Architecture::Logical::Scheduler` |
| Bi-directional traceability SW req ↔ architecture | `satisfies:` on PartDef | `matrix` W300 = unallocated reqs |
| Consistency with system architectural design | `supertype:` from SW subsystem to system PartDef | `connectivity` command |

**SWE.3 — Software Detailed Design and Unit Construction**

| ASPICE work product | syscribe artefact | Notes |
|---|---|---|
| Software detailed design (SDD) | `Architecture::Physical/**` (L3 PartDefs) | Part IX §9.8–9.14 |
| Unit interfaces | `PortDef` (L3), typed `features:` on PartDef | `list PortDef Architecture::Interfaces::L3` |
| Data types and global data | `AttributeDef`, `EnumerationDef` | `list AttributeDef`, `list EnumerationDef` |
| Unit algorithms | `ActionDef` with `subActions:`, `controlNodes:` | `list ActionDef Architecture::Physical` |
| Error handling | `ActionDef` error path body; `AOU-*` | `list AssumptionOfUse` |
| WCET / resource usage | `wcet:` on Requirement; body on PartDef | `list Requirement --has-wcet` |
| Bi-directional traceability detail ↔ architecture | `supertype:` (L3→L2) + `satisfies:` | `links` + `connectivity` |
| Coding standard compliance | Enforced by clippy deny lints (external) | Document in ADR; cite `CLAUDE.md` |

**SWE.4 — Software Unit Verification**

| ASPICE work product | syscribe artefact | Notes |
|---|---|---|
| Unit test specification | `TestCase` L1/L2/L3 | `list TestCase --level L1` |
| Unit test cases (coverage criteria) | `testFunctions:` + Gherkin body | `scaffold-gherkin` command |
| Unit test results | Ingested via `ingest-results` | `matrix` shows `✓` vs `▣` |
| Coverage of requirements by unit tests | `matrix` + `verification-depth` | `--min-levels 1` for unit coverage |
| Bi-directional traceability req ↔ unit test | `verifies:` on TestCase | `who-verifies REQ-X` |

**SWE.5 — Software Integration and Integration Testing**

| ASPICE work product | syscribe artefact | Notes |
|---|---|---|
| SW integration test specification | `TestCase` L4 (QEMU) + `TestPlan` | `list TestCase --level L4` |
| SW integration order / strategy | `TestPlan` body (`scope: integration`) | `testplan TP-X` |
| Integration test results | Ingested via `ingest-results` | `matrix` combined L1+L4 |
| Regression test baseline | Git tag + persisted `--json` artefacts | §8.5 in this guide |
| Bi-directional traceability req ↔ integration test | `verifies:` on L4 TestCase | `verification-depth` |

**SWE.6 — Software Qualification Testing**

| ASPICE work product | syscribe artefact | Notes |
|---|---|---|
| SW qualification test specification | `TestCase` L5 + `TestPlan` (`scope: certification`) | `testplan TP-SAFETY-001` |
| SW qualification test results | Ingested results; `audit` verdict | `audit --config CONF-HIL` |
| Release (baseline) evidence | Git tag + JSON artefact bundle | §8.5 |
| Traceability req ↔ qualification test | `matrix --config CONF-HIL` | All rows ✓ for release |

**SUP.8 — Configuration Management (traceability and baseline)**

| ASPICE work product | syscribe artefact | Notes |
|---|---|---|
| Configuration identification | `id:` on every element | `list Requirement --json` → stable IDs |
| Configuration baseline | Git tag | `git tag v1.0-ASPICE-SWE3` |
| Change management | Pull request → git log | Every change has a diff + review record |
| Configuration status accounting | `status:` field; `audit` | `audit` shows approved/draft split |

**SUP.10 — Change Request Management**

syscribe does not implement a change-request workflow natively. Integration approaches:
- Use GitHub/GitLab issues as change requests; reference the issue number in the PR body
  and in the commit message (e.g., `Resolves #42 — REQ-SCHED-003: tighten WCET bound`)
- The PR body is the impact analysis; the diff is the change evidence
- Use `extRef:` on modified elements to link back to the change-request tracking system

**ASPICE readiness command sequence**:

```bash
# ASPICE SWE.1 — requirement completeness
syscribe -m model audit                         # approved/draft split per package
syscribe -m model list Requirement --status draft  # outstanding items

# ASPICE SWE.2 — architecture coverage
syscribe -m model matrix                        # W300 = requirement not allocated to subsystem

# ASPICE SWE.3 — detail design traceability
syscribe -m model list PartDef Architecture::Physical   # L3 unit inventory
syscribe -m model matrix --allocations          # function → L2 → L3 allocation chain

# ASPICE SWE.4/SWE.5 — test coverage by level
syscribe -m model verification-depth            # per-requirement distinct level count
syscribe -m model matrix --gaps-only            # outstanding coverage gaps

# ASPICE SWE.6 — qualification evidence
syscribe -m model validate --config CONF-HIL    # per-variant gate
syscribe -m model audit --config CONF-HIL       # readiness verdict for that variant
```

### A.5 AUTOSAR tooling (EB Tresos, Vector DaVinci)

syscribe is not a functional configuration tool and does not replace AUTOSAR tooling.
Its role is upstream: capturing the safety requirements, safety goals, and architectural
decisions that drive AUTOSAR configuration choices. Use `extRef:` to link syscribe
requirements to AUTOSAR parameter IDs, and `satisfiedBy:` to point from requirements
to the architecture elements that are realized in AUTOSAR configuration.

---
