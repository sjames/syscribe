---
type: Configuration
id: CONF-GTI-IN-M2-001
name: "India 10 kW, 2 MPPT, basic"
status: approved
featureModel: Features
features:
  Features::Region: true
  Features::Region::US: false
  Features::Region::IN: true
  Features::PvInput: true
  Features::PvInput::Mppt2: true
  Features::PvInput::Mppt3: false
  Features::Mppt: true
  Features::Mppt::GlobalScan: false
  Features::Mppt::StringMonitoring: false
  Features::Protection: true
  Features::Protection::Afci: false
  Features::Protection::RapidShutdown: false
  Features::Comms: true
  Features::Comms::SunSpecModbus: true
  Features::Comms::Ieee2030_5: false
  Features::GridSupport: true
  Features::GridSupport::VoltVar: false
  Features::GridSupport::FreqWatt: false
  Features::GridSupport::RemoteCurtailment: false
  Features::GridProfile: true
parameterBindings:
  Features::GridProfile.nominalFrequencyHz: 50
  Features::GridProfile.nominalLineVoltageV: 415
  Features::GridProfile.reconnectDelayS: 60
  Features::GridProfile.islandClearS: 2
  Features::GridProfile.overVoltagePu: 1.1
  Features::GridProfile.underVoltagePu: 0.85
  Features::GridProfile.overFreqHz: 50.5
  Features::GridProfile.underFreqHz: 47.5
  Features::GridProfile.reconnectVoltageMaxPu: 1.1
  Features::GridProfile.reconnectVoltageMinPu: 0.9
  Features::GridProfile.reconnectFreqMaxHz: 50.5
  Features::GridProfile.reconnectFreqMinHz: 49.5
  Features::GridProfile.clearingMs: 200
---

India entry model.
