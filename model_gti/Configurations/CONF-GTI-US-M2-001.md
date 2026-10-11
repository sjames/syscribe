---
type: Configuration
id: CONF-GTI-US-M2-001
name: "US 10 kW, 2 MPPT, minimum compliant"
status: approved
featureModel: Features
features:
  Features::Region: true
  Features::Region::US: true
  Features::Region::IN: false
  Features::PvInput: true
  Features::PvInput::Mppt2: true
  Features::PvInput::Mppt3: false
  Features::Mppt: true
  Features::Mppt::GlobalScan: false
  Features::Mppt::StringMonitoring: false
  Features::Protection: true
  Features::Protection::Afci: true
  Features::Protection::RapidShutdown: true
  Features::Comms: true
  Features::Comms::SunSpecModbus: false
  Features::Comms::Ieee2030_5: true
  Features::GridSupport: true
  Features::GridSupport::VoltVar: true
  Features::GridSupport::FreqWatt: true
  Features::GridSupport::RemoteCurtailment: false
  Features::GridProfile: true
parameterBindings:
  Features::GridProfile.nominalFrequencyHz: 60
  Features::GridProfile.nominalLineVoltageV: 208
  Features::GridProfile.reconnectDelayS: 300
  Features::GridProfile.islandClearS: 2
  Features::GridProfile.overVoltagePu: 1.2
  Features::GridProfile.underVoltagePu: 0.45
  Features::GridProfile.overFreqHz: 62.0
  Features::GridProfile.underFreqHz: 56.5
  Features::GridProfile.reconnectVoltageMaxPu: 1.05
  Features::GridProfile.reconnectVoltageMinPu: 0.917
  Features::GridProfile.reconnectFreqMaxHz: 60.1
  Features::GridProfile.reconnectFreqMinHz: 59.5
  Features::GridProfile.clearingMs: 160
---

US entry model: two trackers, no global scan, only the functions the US constraint forces.
