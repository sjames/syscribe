---
type: PartDef
name: Signal Output Module
domain: hardware
features:
  - name: aspectCount
    typedBy: ScalarValues::Integer
---

The Signal Output Module is a vital relay-based signal lamp driver. Each signal aspect (e.g., red, yellow, green, double yellow) is driven through a series safety relay chain. The relay chain design ensures that:

- A lamp-stick fault (an aspect illuminated when not commanded) is detectable by monitoring the relay coil current versus the commanded state.
- A stuck relay in the energised position is reported to the vital processor within one scan cycle.
- Loss of supply voltage causes all aspects to extinguish (most-restrictive state).

The vital processor receives confirmation of the actual state of each relay independently from the command path. A signal is only reported as showing the commanded aspect when the feedback confirms the expected relay positions; any discrepancy is treated as a failure and results in a safe-state transition.

The module supports multi-aspect colour light signals. Aspect selection is mutually exclusive at the hardware level: only one relay chain can be energised at a time, preventing simultaneous display of conflicting aspects.
