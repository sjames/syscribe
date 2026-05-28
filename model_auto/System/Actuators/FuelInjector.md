---
type: PartDef
name: Fuel Injector
domain: hardware
features:
  - name: staticFlowRate
    type: ScalarValues::Real
    unit: cc/min
  - name: operatingPressure
    type: ScalarValues::Real
    unit: bar
---

Solenoid-type port fuel injector. Controlled by `System::Software::FuelControl`
using pulse-width modulation to deliver the commanded fuel mass per cycle.

Peak-and-hold driver: 4 A peak for 1 ms pull-in, then 1 A hold.
