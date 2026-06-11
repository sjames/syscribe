---
type: PartDef
name: BadMoP
custom_fields:
  mg_mop: true
  mg_mop_refines: RangeMoE
  mg_mop_unit: kWh
---

mg_mop declared on a PartDef rather than a ConstraintDef/CalculationDef → MG050
(wrong host) under the gate. Without the gate the field is inert (no MG05x).
