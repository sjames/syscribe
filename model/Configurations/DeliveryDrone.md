---
type: Configuration
id: CONF-UAV-DELIVERY-001
title: "Delivery drone — redundant hex with cargo module and satcom"
status: approved
featureModel: Features
features:
  Features::Propulsion: true
  Features::Propulsion::Quad: false
  Features::Propulsion::Hex: true
  Features::Payload: true
  Features::Payload::Survey: false
  Features::Payload::Mapping: false
  Features::Payload::Delivery: true
  Features::DataLink: true
  Features::DataLink::LoRa: false
  Features::DataLink::Cellular: false
  Features::DataLink::Satcom: true
  Features::DualFlightController: true
parameterBindings:
  Features::Payload::Delivery::payloadCapacityKg: 3.0
---

Logistics delivery product: a redundant hexacopter carrying a 3 kg-capacity cargo
release module, communicating over a satellite terminal for global-range
delivery missions.
