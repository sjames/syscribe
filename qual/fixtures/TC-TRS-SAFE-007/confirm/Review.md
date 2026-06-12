---
type: ConfirmationMeasure
id: CM-BRK-001
name: "Confirmation review of the braking work products"
status: approved
measureType: confirmation_review
independenceLevel: I2
confirms:
  - SG-BRK-001
---

A confirmation review exists (so the confirmation-tracking check is active), but it is
not an I3 functional_safety_assessment — the ASIL D goal still lacks its required
independent assessment, so W039 fires.
