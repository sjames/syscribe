---
id: REQ-TRS-FTA-004
type: Requirement
name: "Tool shall validate fault-tree structure: gate cycles, arity, value ranges, reachability and stray nodes"
status: draft
reqDomain: software
verificationMethod: test
---

The validator **shall** report: a gate cycle or a gate listing itself in `inputs` (`E980`); a `NOT` gate with more than one input, an `XOR` gate with more than two, or an `inhibit` gate without a conditioning input (`E981`); a negative or non-finite `failureRate` (`E982`); a `probability` outside 0..1 (`E983`); a `ccfBeta` outside 0..1 (`E984`); a node not reachable from the tree's top node (`W980`); a gate/event outside any `FaultTree` directory (`W981`); a single-input `AND`/`OR` gate (`W982`); a single-member `ccfGroup` (`W983`); a `ccfGroup` with missing or inconsistent betas (`W984`); and an unparsable `missionTime` (`W987`).

**Source:** GitHub issue #212.

**Acceptance criteria:** Each fixture raises exactly the documented code; a well-formed tree raises none of them.
