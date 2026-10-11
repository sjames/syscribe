---
type: Configuration
id: CONF-GTI-IN-M3-001
name: "India 10 kW, 3 MPPT, shaded-roof variant"
status: approved
featureModel: Features
features:
  Features::Region: true
  Features::Region::US: false
  Features::Region::IN: true
  Features::PvInput: true
  Features::PvInput::Mppt2: false
  Features::PvInput::Mppt3: true
  Features::Mppt: true
  Features::Mppt::GlobalScan: true
  Features::Mppt::StringMonitoring: true
  Features::Protection: true
  Features::Protection::Afci: false
  Features::Protection::RapidShutdown: false
  Features::Comms: true
  Features::Comms::SunSpecModbus: true
  Features::Comms::Ieee2030_5: false
  Features::GridSupport: true
  Features::GridSupport::VoltVar: false
  Features::GridSupport::FreqWatt: false
  Features::GridSupport::RemoteCurtailment: true
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
  Features::Mppt::GlobalScan.scanPeriodS: 300
---

India variant for partially shaded rooftops: three trackers, global scan every 5 min, string monitoring, Modbus and remote curtailment. No arc-fault or rapid shutdown (not mandated, see ADR-GTI-ASM-003).
