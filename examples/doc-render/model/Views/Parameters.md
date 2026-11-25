---
type: View
name: Parameters
expose: ["Features::*"]
rows: parameters
columns:
  - {field: key, heading: Parameter}
  - {field: quantity, heading: Value, align: right}
  - {field: range, heading: Range, empty: "—"}
  - {field: derivedFrom, heading: Derived from, empty: "—"}
sort: [key]
caption: "Feature parameters"
---

The parameters of the heater, with the value each takes: under `--config` the value of that
Configuration, otherwise the fixed, derived or default value.
