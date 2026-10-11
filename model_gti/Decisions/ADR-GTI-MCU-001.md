---
type: ADR
id: ADR-GTI-MCU-001
name: "Use the STM32G474RE as the control microcontroller"
status: proposed
date: "2026-10-11"
tags: [architecture, mcu, stm32, constraint]
---

## Context

The customer requires an STM32 part (`REQ-GTI-STK-007`, `REQ-GTI-SYS-070`). The part must run, on one core: a 16–48 kHz interleaved boost and three-level bridge control (PWM with fine resolution), up to three MPPT instances, a PLL and current loop for a 50/60 Hz grid, the V/f trip table, anti-islanding perturbation, arc-fault signature analysis (about 100 kS/s per input), insulation monitoring, SunSpec/IEEE 2030.5 and an authenticated bootloader, with at most 70 % CPU use (`REQ-GTI-SYS-072`). It must also support safety self-test (`REQ-GTI-SYS-071`) and hardware trips independent of software (`REQ-GTI-SYS-037`).

Figures below are recalled from vendor documentation and **must be checked against the current datasheet** before the part is frozen; they are not verified here.

## Options

| Part | Core | Timer | Analogue | Verdict |
|---|---|---|---|---|
| STM32F334 | M4F 72 MHz | HRTIM | 2 ADC, 3 comparators | too little CPU and ADC for 3 trackers plus arc analysis |
| **STM32G474RE** | M4F 170 MHz, FPU, CORDIC and FMAC accelerators | HRTIM (≈184 ps), 10 channels | 5 ADCs, 7 comparators, 4 op-amps | **chosen** |
| STM32H745 (dual core) | M7 480 MHz + M4 | HRTIM | 3 ADCs | headroom for a split safety/comms design but higher cost and power; keep as growth option |
| STM32H563 | M33 250 MHz | no HRTIM | 2 ADCs | no high-resolution PWM |

## Decision

Use the STM32G474RE. HRTIM drives the boost stages and bridge; the on-chip comparators combined with HRTIM fault inputs give the hardware overcurrent trip; CORDIC/FMAC offload the PLL and arc-signature filters. An external hardware watchdog supplements the on-chip window and independent watchdogs.

## Consequences

* One core runs both control and protection, so freedom from interference is by priority, MPU regions and the independent hardware trip path, not by a second core. If the safety case requires a physically separate channel, the H745 or a second small STM32 is the escalation path.
* The CPU budget must be confirmed by measurement on the evaluation board (`REQ-GTI-SYS-072`).
* 512 kB flash must hold two firmware images for the anti-rollback update; confirm after the first build.
* The status stays `proposed` until the resource analysis and the datasheet check are done.
