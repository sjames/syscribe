---
id: SG-GTI-002
type: SafetyGoal
status: approved
appliesWhen: Features::Protection::Afci
name: Do not sustain a DC series arc
silLevel: 2
safeState: Affected input interrupted, bridge off
ftti: 2.5s
hazardousEvents:
  - HE-GTI-002
---

A series arc in a 1000 V string can ignite the roof. The inverter **shall** detect and interrupt it within the UL 1699B time and stay off until a person clears it.
