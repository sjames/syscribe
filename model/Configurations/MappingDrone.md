---
type: Configuration
id: CONF-UAV-MAPPING-001
title: "Mapping drone — redundant hex with LiDAR and cellular link"
status: approved
featureModel: Features
features:
  Features::Propulsion: true
  Features::Propulsion::Quad: false
  Features::Propulsion::Hex: true
  Features::Payload: true
  Features::Payload::Survey: false
  Features::Payload::Mapping: true
  Features::Payload::Delivery: false
  Features::DataLink: true
  Features::DataLink::LoRa: false
  Features::DataLink::Cellular: true
  Features::DataLink::Satcom: false
  Features::DualFlightController: true
---

Professional mapping product: a redundant hexacopter (dual flight controller)
carrying a LiDAR scanner, communicating over a 4G cellular modem for
beyond-line-of-sight operation.
