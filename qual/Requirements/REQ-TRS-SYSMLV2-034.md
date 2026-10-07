---
id: REQ-TRS-SYSMLV2-034
type: Requirement
name: Tool shall map a SysMLv2 calc def/calc onto a native CalculationDef/Calculation carrying in/out/return parameters, returnType, the expression as an opaque kerml body string, typedBy and doc
status: verified
reqDomain: software
verificationMethod: test
---

A `calc def` shall be synthesized into a native `CalculationDef` (package level) and a named
`calc` usage into a native `Calculation` (nested in a `part def` body — the only place the parser
accepts a calc usage). Both carry:

- `parameters:` — each `in`/`out`/`inout` declaration as a `{name, typedBy, direction}` map, plus
  each `return` declaration as a `direction: return` entry;
- `returnType:` — the type of the first `return` declaration;
- `body:` — the body's expression(s) rendered to text and kept as an **opaque string**, with
  `bodyLanguage: kerml`; both absent for a bodyless calc;
- `typedBy:` on a usage (`calc c : CalcDef`);
- the body's `doc` as the element's doc text.

Nested calc/part members of a calc body are not mapped. `evaluate:`/`bodyLanguage: budget`
(§22.2) remain hand-authored only. Mapped calcs no longer count toward `W543`.
