# Staged configuration

A UAV product line configured in three stages (`ADR-SYS-BT-001`, spec §9.7–§9.8):

| Configuration | Stage | Binds |
|---|---|---|
| `CONF-UAV-BUILD-001` | `compile` | the propulsion family; narrows `motorKV` to `900..1100` |
| `CONF-UAV-DEPLOY-001` | `load` (derived from build) | `motorKV`, `maxCurrentA`, the telemetry feature and its `mode` |
| `CONF-UAV-FLIGHT-001` | `runtime` (derived from deploy) | `Telemetry.rateHz` |

```bash
syscribe -m examples/staged-configuration/model/ validate
syscribe -m examples/staged-configuration/model/ binding-times
syscribe -m examples/staged-configuration/model/ residual --config CONF-UAV-BUILD-001
syscribe -m examples/staged-configuration/model/ configure CONF-UAV-DEPLOY-001
syscribe -m examples/staged-configuration/model/ feature-check --deep
syscribe -m examples/staged-configuration/model/ params export --config CONF-UAV-FLIGHT-001 --schema flight.schema.json
syscribe -m examples/staged-configuration/model/ params export --config CONF-UAV-BUILD-001   # fails: motorKV is not bound yet
```

`params export` writes the YAML file the running system reads its `load`- and `runtime`-bound
parameters from (and, with `--schema`, a JSON Schema for it); `params check <file> --config …`
validates such a file against the model and flags one generated from an older model as stale.
Exporting the build Configuration fails by design: `motorKV` is required and no stage has bound it.

`residual` on the build Configuration lists what deployment and flight still have to
decide; on the flight Configuration it lists nothing. `configure` on the deployment
Configuration shows where every value came from and at which stage it was bound.
