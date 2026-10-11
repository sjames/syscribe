---
type: PartDef
name: Control Board
domain: hardware
features:
  - {name: mcu, typedBy: System::Hardware::Stm32G474}
  - {name: comms, typedBy: System::Hardware::CommsHardware}
  - {name: adcIn, type: Port, direction: in}
  - {name: hwVin, type: Port, direction: in}
  - {name: arcIn, type: Port, direction: in}
  - {name: auxIn, type: Port, direction: in}
  - {name: pwmOut, type: Port, direction: out}
  - {name: relayCmd, type: Port, direction: out}
  - {name: hwTripGate, type: Port, direction: out}
  - {name: hwTripRelay, type: Port, direction: out}
  - {name: rsdCmd, type: Port, direction: out}
  - {name: comms, type: Port, direction: inout}
---

Carries the STM32, the hardware overcurrent comparators, the independent hardware watchdog, gate-driver interface and the auxiliary supply.
