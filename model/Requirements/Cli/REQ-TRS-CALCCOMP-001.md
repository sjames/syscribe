---
type: Requirement
id: REQ-TRS-CALCCOMP-001
name: "A calculation composes: other calculations are read by name or called with arguments, and a used calculation raises no W007"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

A `CalculationDef` qualified name in a calculation `body:`, a `Constraint` `bindingConnections` `right:` or a `parameterConstraints` expression shall evaluate that calculation in the same environment; `Calc::Name(param = value, …)` shall call a calculation that has `in` parameters (an unknown parameter name is an error, an omitted one takes its default) (GH #491). A calculation reached again while it is being evaluated is a cycle and an error naming the chain. A calculation read this way, or exported by a `buildVar`, shall not raise `W007`.
