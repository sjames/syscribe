---
type: Package
name: Grid-Tie Inverter (US / India)
---

A three-phase, transformerless, string-class **grid-tie inverter** with a **hybrid multi-string, high-voltage solar input** (several independent MPPT trackers fed from 1000 V strings), one hardware platform sold into **two grid-code regions**: the United States (IEEE 1547-2018 / UL 1741 SB / NEC) and India (CEA connectivity regulations / BIS IS 16221 / IS 16169). The control computer is **an STM32 microcontroller** (customer constraint, `REQ-GTI-STK-008`).

This is a dogfooding model: it exercises the product-line, requirements, architecture, safety/security and the executable-behaviour features (state machines, composition, fault injection) of syscribe on a realistic multi-region product. Start with [Analysis](Analysis/_index.md), which records the domain analysis, the assumptions to be confirmed and the regulatory-verification status of every limit used.

Regulatory values are **not** hard-coded as facts. They live in per-region grid profiles (see `Analysis`) with a verification status; nothing in this model is a certified compliance statement.
