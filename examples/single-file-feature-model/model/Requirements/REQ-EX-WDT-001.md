---
id: REQ-EX-WDT-001
type: Requirement
name: The watchdog shall reset the system if not serviced within timeoutMs
status: approved
reqDomain: software
verificationMethod: test
appliesWhen: Features::Wdt
satisfies: []
---

The system **shall** reset within `timeoutMs` of the watchdog not being
serviced. Active only in products that select the `Wdt` feature.
