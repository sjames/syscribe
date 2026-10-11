---
type: Allocation
name: Firmware to STM32
allocatedFrom:
  - System::Software::Firmware
allocatedTo:
  - System::Hardware::Stm32G474
---

The firmware image executes on the STM32G474 (Cortex-M4F). Fast loops run from HRTIM and ADC interrupts; supervisor and protection share the core under a fixed-priority scheduler. Resource margins: REQ-GTI-SYS-072.
