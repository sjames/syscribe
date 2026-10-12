# Behavioural models of the architecture (`models:`)

An executable model (parts that `exhibitsStates:` a machine, joined in a composite that a TestCase drives) and the architecture (parts with a `domain:` that `satisfies:` requirements) are usually two sets of elements. They differ for good reasons: two machines can sit inside one component, one sensor component can have two independent chains, a scenario needs a grid source that is not part of the product, and a model covers only part of what a component does. `models:` records which architecture element each executable part stands for, so the two are checked against each other (GH #330, spec §12.11).

Nothing changes in a model that has no `models:`.

## Declaring it

<!-- syscribe-example: skip reason="an excerpt of model_gti; the architecture element and machine it names live there" -->
```yaml
---
type: PartDef
name: GridMonitorPart
exhibitsStates: [Behavior::Machines::GridMonitor]
models: [System::Software::ProtectionManager]
fidelity: "fast-stage voltage/frequency window only"
abstracts: [arcTrip, profile, relayCmd]
portMap:
  samples: gridSamples
  out: trip
features:
  - {name: samples, type: Port, direction: in}
  - {name: out, type: Port, direction: out}
---
```

* **`models:`** names one or more architecture elements: a `PartDef` or `Part` with a `domain:` on itself or its definition chain.
* **`portMap:`** maps an executable port to the architecture port it stands for. Ports with the same name need no entry, and neither does a port typed by the same `PortDef` (and conjugation) as exactly one port of the modelled element ([typed ports](system-composition.md#typed-ports)); two corresponding ports typed differently are `E991`. A list maps one executable port to several architecture ports (one comparator output that drives two trip lines); `null` says the port has no counterpart on purpose.
* **`abstracts:`** says what the model leaves out. A port name exempts that architecture port from `W990`. A requirement id says the model cannot be evidence for that requirement, so a TestCase that claims it is reported (`W992`). Anything else is a free name for the reader.
* **`fidelity:`** is prose.

## Many-to-one and one-to-many

Two machines may model one element. Ports are then checked on the union: each architecture port needs a counterpart in one of them.

One part may model an element twice, once per chain. Put `portMap:` (or `models:`) on the usage, the composite's `features:` entry, to say which chain each instance is:

```yaml
features:
  - {name: sensorSw, typedBy: Behavior::Parts::SensorPart}                                       # inp: acSig, out: adcOut (the definition's map)
  - {name: sensorHw, typedBy: Behavior::Parts::SensorPart, portMap: {inp: acSig, out: hwVoltage}}
```

## Checks

| Code | What it means | Usual fix |
|---|---|---|
| `E990` | The target does not resolve or is not an architecture element | Fix the reference, or add `domain:` to the target |
| `E991` | An executable port has no counterpart, a `portMap:` key is not a port, or the directions disagree | Map the port, map it to `null`, or fix the direction |
| `W990` | An architecture port has no counterpart in any part that models it | Add or map a port, or name it in `abstracts:` |
| `W991` | A connection of the executable composite has no architecture counterpart, or the reverse | Add the missing connection on the side that lacks it. Connections between a software and a hardware element are not checked, since the architecture joins those through allocation (§12.6) |
| `W992` | A TestCase verifies a requirement that no part of its subject models | Remove the requirement from `verifies:`, or model the element that satisfies it |

`W992` is the check that would have caught the grid-tie inverter's protection TestCase listing `REQ-GTI-SYS-037` (10 µs overcurrent trip) and `REQ-GTI-HW-002` (relay weld detection): the power stage and relay parts model those components, but name those two requirements in `abstracts:`, so the scenarios are not evidence for them.

## Queries

`models:` is a trace link: `links` and `refs` list it, `follow <arch> modeledBy` walks it backwards, `impact <arch>` reaches the executable parts, `trace <REQ>` adds a *Behavioural models of the satisfiers* table, and `why <exec-part>` answers with the requirements its modelled elements satisfy. It takes part in suspect-link detection, so a change to the architecture element raises `W090` on the behaviour once baselined with `suspect accept`.

## Limits of this version

* A `models:` written on a part usage (a composite's `features:` entry) is checked and counted for `W992`, but `links`, `refs`, `follow`, `impact` and suspect-link detection see only a `models:` on a definition.
* `W991` matches direct architecture connections. Two executable parts that model elements at different levels of the hierarchy, joined in the architecture through a delegated port, are reported; connections between a software and a hardware element are not checked at all.

The worked example is `model_gti/Behavior/Parts/` and `model_gti/Behavior/Chain/ProtectionChain.md`.
