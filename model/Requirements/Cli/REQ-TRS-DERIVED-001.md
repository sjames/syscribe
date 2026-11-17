---
type: Requirement
id: REQ-TRS-DERIVED-001
name: "A derived parameter is computed from the Configuration's effective values and used wherever a bound value is"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

A parameter declared `derivedFrom: "<expression>"` shall take the value of that expression, in the language of `syscribe eval`, over the Configuration's effective parameter values (binding, fixed `value:`, derived value, `default:`) (GH #490). `show` and `configure` shall print it with source `derived`; `build-config` shall export it through its `buildVar` (an expression that cannot be computed is `E248`, not an omission); `parameterConstraints` and `{{Feature.param}}` placeholders shall read it; `range:` shall check it, and an `Integer` parameter shall be whole (`E249`); binding it in a Configuration remains `E204`. An expression that does not parse, or fails for a Configuration whose inputs all have values, is `E248`; a cycle across parameters is an error.
