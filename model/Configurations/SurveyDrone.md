---
type: Configuration
id: CONF-UAV-SURVEY-001
title: "Survey drone — compact quad with EO camera and LoRa link"
status: approved
featureModel: Features
features:
  Features::Propulsion: true
  Features::Propulsion::Quad: true
  Features::Propulsion::Hex: false
  Features::Payload: true
  Features::Payload::Survey: true
  Features::Payload::Mapping: false
  Features::Payload::Delivery: false
  Features::DataLink: true
  Features::DataLink::LoRa: true
  Features::DataLink::Cellular: false
  Features::DataLink::Satcom: false
  Features::DualFlightController: false
---

Entry-level survey product: a lightweight quadcopter carrying an electro-optical
camera, communicating over a long-range LoRa link. No flight-controller
redundancy.
