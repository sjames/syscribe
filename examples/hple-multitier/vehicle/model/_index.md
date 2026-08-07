---
type: Package
name: Vehicle
---

Top tier of the worked example: a vehicle product line that consolidates the `battery-pack` line
(`[repos.battery_pack]` below, resolved by `../../battery-pack`) — which itself consolidates the
`battery-cell` line — via `subConfigurations:` at both levels
(`ADR-SYS-HPLE-001`: "subConfigurations at any tier"). See the top-level `README.md` for the full
worked scenario and the exact commands to reproduce every finding described here.
