# PLE fixture

A small product line for **front-end contract tests** (GH #430): the model a VS Code extension, or any other
client, can run against the real `syscribe` binary and assert exact data on. See
[Front-end CI](../../docs/guides/frontend-ci.md) and `crates/syscribe/tests/frontend_contract.rs`.

| It covers | Where |
|---|---|
| a mandatory **alternative** group (`Platform`: `CortexM` / `RiscV`) and a mandatory **or** group (`Link`: `Lora` / `Wifi` / `Cellular`), both abstract | `Features/Platform`, `Features/Link` |
| **cross-tree constraints** (`Camera` requires `CortexM`, excludes `Cellular`) | `Features/Camera.md` |
| **typed parameters** with unit, range, enumeration, default, required flag and a binding window | `Propulsion.motorKV`, `Logging.{levelName,bufferKiB,rateHz}` |
| **parameter constraints** (an error and a warning) | `Features/_index.md` |
| **three binding stages** in a `derivedFrom:` chain with a narrowing, plus an unstaged product | `Configurations/` (`BUILD` compile → `DEPLOY` load → `FLIGHT` runtime; `ALT`) |
| requirements gated by features, tests on two rigs, a requirement with no test | `Requirements/`, `Verification/` |
| a `PlanningItem` tree with an open question | `Planning/` |

`validate` reports **no errors** (a few warnings by design: orphan requirements, one untested requirement).
`EXPECTED.json` lists exactly what is in it.

```bash
syscribe -m examples/ple-fixture/model validate
syscribe -m examples/ple-fixture/model propagate --select Features::Camera=true
syscribe capabilities --json     # gate on this first: version and features
```
