---
type: Package
name: BatteryPack
---

Middle tier of the worked example: a battery-pack product line built by consolidating the
`battery-cell` line (`[repos.battery_cell]` below, resolved by `../../battery-cell`) via
`subConfigurations:`, and itself consolidated further by the `vehicle` tier above it. Neither this
tier's `Configuration` nor any of its `FeatureDef`s carry any field naming, or capable of naming,
whoever eventually consolidates it (`REQ-TRS-HPLE-005`) — this package would be entirely unchanged
if no one ever imported it, or if ten different vehicle programs each imported it independently.
