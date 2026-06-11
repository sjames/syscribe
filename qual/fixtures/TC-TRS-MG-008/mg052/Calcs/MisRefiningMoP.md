---
type: ConstraintDef
name: MisRefiningMoP
custom_fields:
  mg_mop: true
  mg_mop_refines: PlainCalc
  mg_mop_unit: kWh
---

mg_mop_refines resolves to PlainCalc, which is a CalculationDef without the
mg_moe marker → MG052 (refines target not an mg_moe) under the gate.
