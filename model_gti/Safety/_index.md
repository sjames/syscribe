---
type: Package
name: Safety
---

Hazard analysis, FMEA and fault tree of the grid-tie inverter. The risk assessment uses the IEC 61508 risk graph as a proposal (inverter product standards IEC 62109 / UL 1741 do not prescribe SIL); the compliance engineer decides the integrity scheme. FMEA-GTI-002 and FT-GTI-001 carry `injection:` fault models so that `syscribe inject` can run the executable protection chain under each failure.
