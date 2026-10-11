---
id: SG-GTI-005
type: SafetyGoal
status: approved
name: Do not leave first responders exposed to array voltage
silLevel: 1
safeState: Controlled array conductors at or below 30 V
ftti: 30s
appliesWhen: Features::Protection::RapidShutdown
hazardousEvents:
  - HE-GTI-005
---

A firefighter cutting into a roof or conduit must not meet a live 1000 V DC conductor. On loss of AC or on an initiator signal the controlled conductors **shall** fall to a touch-safe voltage (NEC 690.12 style, `[unverified]`).
