---
type: ItemDef
name: GPSFix
supertype: Items::Item
features:
  - name: latitude
    typedBy: ScalarValues::Real
  - name: longitude
    typedBy: ScalarValues::Real
  - name: altitudeM
    typedBy: ScalarValues::Real
    unit: SI::m
  - name: fixQuality
    typedBy: ScalarValues::Integer
---

A GPS position fix reported by the receiver to the flight controller. fixQuality: 0=invalid, 1=GPS, 2=DGPS.
