---
type: PartDef
name: Control Board
domain: hardware
features:
  - {name: mcu, typedBy: System::Hardware::Stm32G474}
  - {name: comms, typedBy: System::Hardware::CommsHardware}
---

Carries the STM32, the hardware overcurrent comparators, the independent hardware watchdog, gate-driver interface and the auxiliary supply.
