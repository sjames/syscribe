---
type: Package
name: Analysis
---

# Domain analysis, assumptions and regulatory matrix

This page is the analysis behind the requirements. Decisions taken on the stakeholder's behalf are recorded as `proposed` ADRs (`ADR-GTI-ASM-*`, see `Decisions`) so they stay open until someone confirms or rejects them.

## 1. Product in one paragraph

A **10 kW-class, three-phase, transformerless grid-tie inverter** converting DC from **several independent high-voltage PV strings** (max 1000 V DC) into AC fed to the utility grid. Each string input pair is tracked by its own **maximum power point tracker (MPPT)**; this multi-string input with per-tracker MPPT is what the stakeholder calls the *hybrid solar mode* (see `ADR-GTI-ASM-001` for the reading of "hybrid" and of the word "moot", taken as MPPT). One hardware platform is sold in **two regions**, US and India, differing in grid code, nominal voltage and frequency, mandatory protective functions and certification. The control computer is an **STM32** microcontroller (customer constraint).

## 2. Stakeholders and their concerns

| Stakeholder | Concern |
|---|---|
| Plant owner / homeowner | Energy yield, uptime, payback; no nuisance trips |
| Installer / commissioning engineer | Safe, quick commissioning; region selected correctly and locked |
| Firefighter / first responder | De-energised array conductors on demand (US: rapid shutdown) |
| Utility / DISCOM / grid operator | Grid-code behaviour: trip limits, ride-through, no islanding, power quality, remotely curtailable |
| Certification bodies (NRTL / BIS) | Evidence against UL 1741 SB, IEEE 1547.1, UL 1699B; IS 16221, IS 16169 |
| Manufacturer | One hardware BOM, firmware-selected region, STM32 platform, cost, field-updatable firmware |
| Cyber / regulator | Authenticated remote control and firmware; secure by default |

## 3. Operating context and region differences

| Aspect | United States | India |
|---|---|---|
| Grid | 60 Hz, three-phase 208 V (or 480 V) | 50 Hz, three-phase 415 V line-to-line (230 V phase) |
| Interconnection standard | IEEE 1547-2018, tested to IEEE 1547.1; UL 1741 SB (grid-support functions); California Rule 21 where applicable | CEA *Technical Standards for Connectivity of Distributed Generation Resources* Regulations (2013, with amendments); utility/DISCOM requirements |
| Product safety | UL 1741, NEC 690 | BIS CRS: IS 16221 (Part 1 & 2, ≈ IEC 62109) |
| Anti-islanding | UL 1741 / IEEE 1547 (clear island ≤ 2 s) | IS 16169 (≈ IEC 62116) |
| DC arc-fault | NEC 690.11, UL 1699B — **mandatory** | not mandated in the sources consulted; **optional** feature |
| Rapid shutdown | NEC 690.12 — **mandatory** on buildings | not mandated; **optional** feature |
| Smart-inverter functions | Volt-var, freq-watt, volt-watt, ride-through categories (IEEE 1547) | fixed power factor / curtailment by remote setpoint; ride-through per utility |
| Communications | IEEE 2030.5 / SunSpec Modbus (Rule 21) | Modbus, remote monitoring; utility SCADA per DISCOM |
| EMC | FCC Part 15 (Class A/B) | IEC 61000-6 series per BIS CRS |
| Climate | −25…+60 °C, outdoor, NEMA 4X | −25…+60 °C, **high heat, dust, monsoon humidity**, IP65 |

The differences are **not** a second product: the same power stage and the same firmware image serve both regions. The region selects a *grid profile* (limits and timings), enables region-mandatory features, and is locked after commissioning (`REQ-GTI-SYS-041`).

## 4. Hybrid multi-string high-voltage PV input

* Up to **2 or 3 MPPT trackers** (variant), each with **two string inputs** combined in parallel after per-string current sensing and reverse-current protection.
* **High-voltage strings:** open-circuit voltage up to **1000 V DC** (including the cold-temperature Voc rise); MPPT window ~200–850 V; start-up above a minimum voltage.
* **Unequal strings:** each tracker works independently, so strings of different orientation/length/shading do not drag one another down.
* **Partial shading** creates several local maxima on the P-V curve, so a perturb-and-observe tracker can lock on a local one. The design therefore adds a periodic **global scan** (`REQ-GTI-SYS-023`).
* **String monitoring** detects under-performing or lost strings (soiling, shading, blown fuse, connector failure) and is an operator-visible diagnostic.
* **Transformerless topology** means no galvanic isolation between array and grid: **insulation resistance monitoring** before connecting and **residual-current monitoring** (RCMU) while running are mandatory (IEC 62109-2 / IS 16221-2 / UL 1741).
* **DC arc faults** are a genuine fire hazard at 1000 V: detection and interruption (US mandatory).

## 5. The STM32 constraint

The controller **shall be an STM32 part**. The selection of the *exact* part is a design decision (`ADR-GTI-MCU-001`), not a requirement, so that the requirement stays stable if the part changes. Selection criteria derived from the analysis: Cortex-M with FPU and DSP instructions (control loops at 16–50 kHz), high-resolution PWM timers (HRTIM) for the boost stages and bridge, fast 12-bit ADCs with simultaneous sampling, hardware comparators for overcurrent trips independent of software, and vendor safety software (IEC 60730 Class B / UL 1998 self-test libraries). Candidate: **STM32G474** (Cortex-M4F @ 170 MHz, HRTIM, 5 ADCs, 7 comparators); reviewed against alternatives in the ADR. The grid-tie safety functions (anti-islanding, V/f trips, overcurrent) are single-fault-tolerant by design: a **redundant hardware trip path** exists that does not depend on the main control loop.

## 6. Regulatory-verification policy

Regulatory numbers in this model are **inputs to be reviewed by a compliance engineer, not facts**. Primary standards were not accessible while writing this model, so every limit carries a verification status:

* **verified** — checked against the primary standard text by a named person (none yet);
* **secondary** — taken from a reputable secondary source or well-known value, primary text not checked;
* **unverified** — recalled or inferred; must be confirmed before use.

Limits live in **grid profiles**: the `GridProfile.US` and `GridProfile.IN` features carry them as typed feature parameters, bound per `Configuration`. Requirements refer to "the active grid profile" instead of repeating a number, which keeps one requirement text for both regions and confines the numbers to one reviewable place. A requirement body may quote the default figure for readability, tagged `[secondary]` or `[unverified]`.

## 7. Regulatory matrix (every limit used)

| Id | Quantity | US value | India value | Status | Source to check |
|---|---|---|---|---|---|
| RM-01 | Nominal frequency | 60 Hz | 50 Hz | secondary | grid fact |
| RM-02 | Nominal line voltage | 208 V (480 V variant) | 415 V | secondary | grid fact |
| RM-03 | Over-voltage trip (stage 2, fast) | ≥ 1.20 pu, clear ≤ 0.16 s | > 1.10 pu per CEA/DISCOM (secondary-source figure) | unverified | IEEE 1547-2018 Table 11 / CEA 2013 |
| RM-04 | Under-voltage trip (stage 2, fast) | 0.45 pu / ≤ 0.16 s (category not confirmed) | < 0.85 pu | unverified | IEEE 1547-2018 Table 13 / CEA 2013 |
| RM-05 | Over-frequency trip | 62.0 Hz / 0.16 s | 50.5 Hz | unverified | IEEE 1547-2018 Table 19 / CEA 2013 |
| RM-06 | Under-frequency trip | 56.5 Hz / 0.16 s | 47.5 Hz | unverified | IEEE 1547-2018 Table 19 / CEA 2013 |
| RM-07 | Reconnect delay after grid returns | 300 s default (adjustable) | ≥ 60 s | unverified | IEEE 1547-2018 §4.10 / CEA 2013 |
| RM-08 | Reconnect window | V 0.917–1.05 pu, f 59.5–60.1 Hz | V and f within normal band | unverified | IEEE 1547-2018 Table 4 |
| RM-09 | Anti-islanding clearing time | ≤ 2 s | ≤ 2 s (IEC 62116 test) | secondary | IEEE 1547 §8.1 / IS 16169 |
| RM-10 | DC current injection | ≤ 0.5 % of rated current | **not confirmed** (do not assume the same) | unverified | IEEE 1547 §7.4 / CEA 2013 |
| RM-11 | Current distortion at rated power (IEEE 1547 uses TDD, IEC 61727 THD) | ≤ 5 % | ≤ 5 % (assumed from IEC 61727) | unverified | IEEE 1547 §10 / IEC 61727 |
| RM-12 | Power-factor capability | 0.8 lead … 0.8 lag (design intent; the 1547-2018 Category B reactive capability is stated differently and must be checked) | ≥ 0.95 lag … 0.95 lead (utility) | unverified | IEEE 1547 §5 / utility |
| RM-13 | Rapid-shutdown | ≤ 30 V within 30 s outside array boundary | not applicable | unverified | NEC 690.12 (edition-dependent) |
| RM-14 | DC arc-fault | detect & interrupt per UL 1699B | optional | unverified | NEC 690.11 / UL 1699B |
| RM-15 | Ride-through category | Category II or III selectable | per DISCOM (LVRT) | unverified | IEEE 1547-2018 §6.4 |
| RM-16 | Insulation resistance before connect | per IEC 62109-2 formula | same (IS 16221-2) | secondary | IEC 62109-2 §13.9 |
| RM-17 | Residual-current monitoring | 300 mA continuous, step trips 30/60/150 mA | same | secondary | IEC 62109-2 §13.9 |

Where the matrix says *unverified*, the corresponding requirement, parameter or test is tagged `reg-unverified` so that `list --tag reg-unverified` is the compliance engineer's worklist.

## 8. Hazards considered (summary, detailed in `Safety`)

Islanded grid energised by the inverter (lineworker electrocution); DC arc fire at 1000 V; shock via transformerless array leakage; fire from overheating; unintended export beyond grid limits; loss of control by a firmware fault; malicious remote curtailment or region change. The detailed HARA, FMEA with injectable faults and the fault tree are under `Safety`; the executable composite and its `@invariant` scenarios are under `Behavior`.

## 9. Out of scope (recorded so it is not silently assumed)

Battery storage and hybrid-inverter islanding/backup; DC-coupled storage; module-level power electronics; transformer-isolated variants; 1500 V DC; single-phase/split-phase variants; utility-scale central inverters. The word *hybrid* is read as **multi-string HV PV**, per the stakeholder's own definition.
