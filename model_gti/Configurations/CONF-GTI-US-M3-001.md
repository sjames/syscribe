---
type: Configuration
id: CONF-GTI-US-M3-001
name: "US 10 kW, 3 MPPT, full smart-inverter"
status: approved
featureModel: Features
features:
  Features::Region: true
  Features::Region::US: true
  Features::Region::IN: false
  Features::PvInput: true
  Features::PvInput::Mppt2: false
  Features::PvInput::Mppt3: true
  Features::Mppt: true
  Features::Mppt::GlobalScan: true
  Features::Mppt::StringMonitoring: true
  Features::Protection: true
  Features::Protection::Afci: true
  Features::Protection::RapidShutdown: true
  Features::Comms: true
  Features::Comms::SunSpecModbus: true
  Features::Comms::Ieee2030_5: true
  Features::GridSupport: true
  Features::GridSupport::VoltVar: true
  Features::GridSupport::FreqWatt: true
  Features::GridSupport::RemoteCurtailment: true
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
  Features::Mppt::GlobalScan.scanPeriodS: 600
---

US flagship: three trackers with global scan and string monitoring; arc-fault and rapid shutdown (forced by the US constraint); IEEE 2030.5 plus SunSpec; volt-var, freq-watt and curtailment.
