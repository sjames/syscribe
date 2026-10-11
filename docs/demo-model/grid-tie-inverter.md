# Grid-Tie Inverter (US / India)

`EXAMPLE · REFERENCE MODEL · model_gti/`

A three-phase, 10 kW, transformerless grid-tie inverter with a **multi-string, high-voltage (1000 V) solar input and one MPPT tracker per string pair**, sold into two grid-code regions (United States, India) from one hardware platform, with a mandatory **STM32** controller. It is the first model built to exercise the executable parts of syscribe together with the product-line and safety features.

```bash
syscribe -m model_gti/ validate --all-configs
syscribe -m model_gti/ simulate TC-GTI-PROT-US-001 --coverage
syscribe -m model_gti/ mutate TC-GTI-PROT-IN-001
syscribe -m model_gti/ inject TC-GTI-PROT-US-001 --ft FT-GTI-001 --max-order 3 --report-only
syscribe -m model_gti/ matrix
```

## What it contains

| Area | Content |
|---|---|
| `Analysis/` | Domain analysis, stakeholder concerns, US/India comparison, regulatory matrix with a verification status for every limit |
| `Features/`, `Configurations/` | Product line: region (US/IN), 2 or 3 trackers, global scan, string monitoring, arc-fault, rapid shutdown, communications, grid-support functions; four stored products and a feature-gated 150 % model (896 valid configurations) |
| `Requirements/` | 8 stakeholder needs, system requirements with regional variants, hardware and software leaves, each traced to a satisfying element and a test |
| `System/`, `Allocations/` | Hardware (power stage, DC input, AC output, sensing, STM32G474 control board) and firmware components; firmware allocated to the STM32 |
| `Decisions/` | Requirement-breakdown ADR, the STM32G474 selection ADR, four recorded assumptions (open until confirmed), and the fault-campaign-driven sensing ADR |
| `Safety/`, `Security/` | HARA (4 goals), architecture FMEA, an FMEA and fault tree with injectable faults, a TARA |
| `Behavior/` | Executable state machines: grid monitor, supervisor, hardware trip, relays, power stage, MPPT tracker; the protection chain as a composite with a US and an India variant |
| `Verification/`, `TestPlans/` | Gherkin test cases (executable and descriptive) and one verification plan per product |

## Assumptions the model makes on the user's behalf

The request said "hybrid solar mode … multi string high voltage solar strings with moot". The model reads *hybrid* as multi-string high-voltage PV (no battery) and *moot* as MPPT. Power class (10 kW), phase configuration (three-phase), DC voltage class (1000 V) and the exact STM32 part are assumptions; all are recorded as `proposed` ADRs (`ADR-GTI-ASM-*`, `ADR-GTI-MCU-001`).

**Regulatory values are not facts.** Primary standards were not accessible when the model was written. Every limit is a review input carrying a verification status in the regulatory matrix; tests and requirements that quote one are tagged `reg-unverified` (`syscribe -m model_gti/ list Requirement --tag reg-unverified`).

## What the executable model found

* **A real design flaw.** The first version of the protection chain fed the software monitor and the hardware trip from one voltage-sensing chain. `inject` reported a single-fault defeat of the invariant "after a grid loss the grid is never energised for longer than 2 s". The fix (an independent sensing chain, `REQ-GTI-HW-008`, `ADR-GTI-SAFE-001`) came from the campaign, and the fault tree's minimal cut sets (order ≤ 3) are all defeated, as the tree predicts, in both regional variants.
* **Test gaps found by mutation.** The first scenario set killed 33 % of the mutants of the grid monitor; boundary scenarios for every limit, recovery edge and insulation threshold bring the chain to 91 % (the rest are mostly equivalent mutants).
* **Three more flaws found by an independent review and fixed with new scenarios:** a dead grid at power-up was never reported (the monitor only reports changes), so the inverter connected to it after the reconnect delay; the hardware trip latched forever after its first operation; and a latched hardware trip did not keep relay B and the gate drive from being commanded closed afterwards. The reconnection window was also wider than the trip window; it is now the narrower window of RM-08. Three `@invariant` scenarios (first loss, second loss after a full cycle, dead grid at power-up) now run under every fault set.
* **A genuine timing defect in the tool**: see finding 1 below.

## Tool findings from building the model

1. **Time accumulates in binary floating point.** `When 99 ms elapses` followed by `When 1 ms elapses` at about 300 s of virtual time does not reach a 100 ms timer started at 300 s (300.099 + 0.001 ≠ 300.1). Scenarios that split a delay at a boundary fail unpredictably. The model works around it with 2 ms margins.
2. **Feature parameters cannot reach state machines.** `GridProfile` parameters (reconnect delay, limits) cannot be used in a machine's guards or `after:` durations, so the US and India machines are generated as two copies.
3. **A common platform element that satisfies a region-specific requirement is flagged `W019` in the other region.** The model needs one gated "realisation" element per optional feature purely to keep projections clean.
4. **FMEA rows cannot be variant-gated** (`appliesWhen:` on a row is `E922`); a row whose `ref` is a gated element raises `W904` in products without it.
5. **Fault models cannot depend on the region.** A "sensor reads nominal" fault needs 60 Hz in the US and 50 Hz in India; the model injects only the voltage.
6. **Test plans need their own `appliesWhen:`** to project cleanly; a `selection:` sweeps in test cases that are inactive in the bound configuration (`W611`).
7. **Configuration ids must end in a 3+ digit number** (`CONF-GTI-US-M3` is `E200`), unlike `FEAT-` ids.
8. **Mutation reports equivalent mutants as survivors**: swapping two transitions with mutually exclusive guards, and shifting a bound by one where values move in steps of 50.
9. **`inject --max-order 2` prints "stopped early: too many cut sets"** for a small tree whose cut sets above order 2 are not needed.
10. **Behaviour parts and architecture parts are separate elements** with no declared link (`Behavior::Parts::GridMonitorPartUs` models `System::Software::ProtectionManager`); today the connection is prose.
11. **Continuous behaviour is out of scope of a state machine.** Anti-islanding perturbation and waveforms are not modelled; grid loss is a sample with zero voltage.
12. **The first `@invariant` was too weak** to see multi-cycle behaviour; the invariants check only the energisation property, not the full safe state (pwm off and both relays open), because a welded relay makes the latter impossible by definition.
13. **Open question the checks surfaced:** with arc-fault protection optional in India, safety goal `SG-GTI-002` has no requirement in an Indian product without it (`W805`). The model gates the goal with the feature; the residual DC arc hazard of such a product needs an installation-rule answer from the safety engineer.
