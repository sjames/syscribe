---
type: Package
name: Behavior
---

Executable behaviour of the protection chain and the MPPT tracker. The chain is a composite of state machines on one virtual clock (`syscribe simulate`, `inject`); the numbers in the machines are the grid-profile defaults, all tagged unverified in the regulatory matrix (Analysis). Two regional variants differ only in their limits and reconnect delay.

Modelling limits (see FINDINGS.md): a state machine cannot read a feature parameter, so the profile values are repeated as literals in the two regional machines; continuous signals (anti-islanding perturbation, waveforms) are not modelled, grid loss is a sample with zero voltage.
