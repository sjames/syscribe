---
type: PlantDef
name: GridImpedance
parameters:
  - {name: zIsland, value: "20 SI::ohm", description: "impedance seen at the point of connection once the grid breaker has opened [unverified]"}
  - {name: tau, value: "20 ms", description: "time constant of the voltage response [unverified]"}
inputs:
  - {name: iInj, unit: "SI::A", description: "perturbation current the inverter injects"}
  - {name: zGrid, unit: "SI::ohm", value: "0.4 SI::ohm", description: "impedance of the grid while it is connected"}
  - {name: island, description: "1 once the grid breaker has opened, else 0"}
outputs:
  - {name: dv, unit: "SI::V", description: "change of the line voltage the injection causes"}
states:
  - {name: x, value: "0 SI::V"}
equations:
  - "der(x) = (-(zGrid + island * (zIsland - zGrid)) * iInj - x) / tau"
  - "dv := x"
maxStep: "10 ms"
---

How the line voltage at the point of connection answers a current the inverter injects: a first-order response to `-z · iInj`, where `z` is the grid impedance while the grid is connected and the much larger island impedance once its breaker has opened. This is what the perturbation-response check (`REQ-GTI-SW-008`) measures: a connected grid answers with a small, known change; an islanded or weak grid with a large one; a sensing chain that reads a fixed value with none at all. The values are illustrative and **unverified**.
