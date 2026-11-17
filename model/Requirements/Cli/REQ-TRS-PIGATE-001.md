---
type: Requirement
id: REQ-TRS-PIGATE-001
name: "Under --config, a PlanningItem reference to an element gated out of the Configuration is not dangling"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
  - planning
---

A PlanningItem describes the project, not one product. In a projection (`--config`, `validate --config`, `validate --all-configs`) a `parent:`, `achieves:` or `blockedBy:` reference to an element that exists in the base model but is gated out of the Configuration by its `appliesWhen:` shall not be `E710`, `E714` or `E720`, so that one planning item can span requirements gated on different features without failing a Configuration that selects only some of them (GH #493). A reference that resolves to nothing in the base model either remains `E710`/`E714`/`E720`, in the base `validate` and under `--config`.
