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

Building this model exposed twelve tool problems. Seven became issues (#310–#316) and were fixed in the tool, and the model was then changed to use the fixes:

| Finding | Issue | Fix, and what the model now does |
|---|---|---|
| Virtual time in binary floating point: `99 ms` + `1 ms` at 300 s missed a 100 ms timer | #310 | Time is kept on a 1 ns grid; the scenarios use the exact split again |
| State machines could not read feature parameters, so the US and India machines were two hand-synced copies | #311 | `{{Features::GridProfile.…}}` in guards, assignments and durations, run with `--config`; one monitor, one supervisor, one chain |
| A platform element satisfying a region-specific requirement raised `W019` in the other region | #312 | A `satisfies:` of a requirement the product lacks is not an escape; the 13 per-feature "realisation" elements are gone |
| FMEA rows could not be variant-gated; fault values could not follow the region | #313 | `appliesWhen:` on a row; placeholders in `injection:` |
| Test plans leaked into other products' projections, and `selection:` swept in inactive test cases | #314, #324 | A plan follows its bound configurations; the hand-written plan `appliesWhen:` is gone |
| Mutation reported equivalent mutants (swapping transitions with disjoint guards) as survivors | #315 (a) | Not generated; case (b), literal shifts on a stepped variable, stays open |
| `inject --max-order` printed "too many cut sets" when it only pruned by order | #316 | Says higher-order cut sets were left out |
| `Configuration` ids had to end in a number (`CONF-GTI-US-M3` was `E200`) | #327 | `CONF-*` needs no numeric suffix, like `FEAT-*` |

Not changed in the tool:

* Behaviour parts and architecture parts are separate elements with no declared link (`Behavior::Parts::GridMonitorPart` models `System::Software::ProtectionManager`); today the connection is prose.
* A state machine cannot express continuous behaviour. Anti-islanding perturbation and waveforms are not modelled; grid loss is a sample with zero voltage, so the `@invariant` never exercises real islanding with the voltage still in window.
* An `@invariant` cannot express the full safe state ("pwm off and both relays open"): a welded relay makes it impossible by definition. The invariants check that the grid is not energised.

## Open decisions

* **Sensing common cause.** The two voltage-sensing chains are independent, but a fault that blinds both (FTE-GTI-009, for example a wiring error at the grid connection point) is a single cut set, and the campaign reports it as defeated. The mitigation is specified: `REQ-GTI-SYS-045`, with a perturbation-response check (`REQ-GTI-SW-008`) that relies on how the grid responds to the inverter's own action instead of the absolute voltage, and current sensing for it (`REQ-GTI-HW-010`), verified by `TC-GTI-SAF-009`. It is not yet modelled in the executable chain, so the campaign still reports the defeat; the thresholds are unverified and the choice of measure is for the safety engineer to confirm.
* **DC arc hazard in India.** With arc-fault protection optional there, safety goal `SG-GTI-002` has no requirement in an Indian product without it (`W805`). The model gates the goal with the feature; the safety engineer must decide how such a product covers the hazard (installation rules, conductor and connector selection).
