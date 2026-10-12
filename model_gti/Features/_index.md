---
type: FeatureModel
name: Features
featureTree:
  - name: Region
    id: FEAT-GTI-REGION
    mandatory: true
    groupKind: alternative
    doc: "Grid-code region of sale; selects the grid profile."
  - name: Region.US
    id: FEAT-GTI-US
    groupKind: optional
    doc: "United States: IEEE 1547-2018, UL 1741 SB, NEC 690."
    requires: [FEAT-GTI-AFCI, FEAT-GTI-RSD, FEAT-GTI-VVAR, FEAT-GTI-FWATT]
  - name: Region.IN
    id: FEAT-GTI-IN
    groupKind: optional
    doc: "India: CEA connectivity regulations, BIS IS 16221 / IS 16169."
  - name: PvInput
    id: FEAT-GTI-PV
    mandatory: true
    groupKind: alternative
    doc: "Number of independent MPPT trackers (two string inputs each)."
  - name: PvInput.Mppt2
    id: FEAT-GTI-MPPT2
    groupKind: optional
  - name: PvInput.Mppt3
    id: FEAT-GTI-MPPT3
    groupKind: optional
  - name: Mppt
    id: FEAT-GTI-MPPT
    mandatory: true
    groupKind: optional
    doc: "Tracking behaviour common to all variants."
  - name: Mppt.GlobalScan
    id: FEAT-GTI-GSCAN
    groupKind: optional
    doc: "Periodic full-range P-V sweep to escape local maxima under partial shading."
    parameters:
      - name: scanPeriodS
        type: ScalarValues::Integer
        range: "60..=3600"
        default: 600
        isRequired: true
        bindingTime: load
  - name: Mppt.StringMonitoring
    id: FEAT-GTI-STRMON
    groupKind: optional
    doc: "Per-string current monitoring and under-performance detection."
  - name: Protection
    id: FEAT-GTI-PROT
    mandatory: true
    groupKind: optional
  - name: Protection.Afci
    id: FEAT-GTI-AFCI
    groupKind: optional
    doc: "DC series arc-fault detection and interruption (NEC 690.11 / UL 1699B)."
  - name: Protection.RapidShutdown
    id: FEAT-GTI-RSD
    groupKind: optional
    doc: "Rapid shutdown of array conductors (NEC 690.12)."
  - name: Comms
    id: FEAT-GTI-COMMS
    mandatory: true
    groupKind: or
  - name: Comms.SunSpecModbus
    id: FEAT-GTI-MODBUS
    groupKind: optional
  - name: Comms.Ieee2030_5
    id: FEAT-GTI-2030
    groupKind: optional
    doc: "IEEE 2030.5 (SEP 2.0) utility communications, California Rule 21."
  - name: GridSupport
    id: FEAT-GTI-GSUP
    mandatory: true
    groupKind: optional
    doc: "Grid-support functions beyond trip limits."
  - name: GridSupport.VoltVar
    id: FEAT-GTI-VVAR
    groupKind: optional
  - name: GridSupport.FreqWatt
    id: FEAT-GTI-FWATT
    groupKind: optional
  - name: GridSupport.RemoteCurtailment
    id: FEAT-GTI-CURT
    groupKind: optional
  - name: GridProfile
    id: FEAT-GTI-PROFILE
    groupKind: mandatory
    doc: "Regulatory limits for the selected region. Values are review inputs, see the regulatory matrix in Analysis."
    parameters:
      - {name: nominalFrequencyHz, type: ScalarValues::Integer, range: "50..=60", isRequired: true, bindingTime: load}
      - {name: nominalLineVoltageV, type: ScalarValues::Integer, range: "200..=480", isRequired: true, bindingTime: load}
      - {name: reconnectDelayS, type: ScalarValues::Integer, range: "10..=600", default: 300, isRequired: true, bindingTime: load}
      - {name: islandClearS, type: ScalarValues::Integer, range: "1..=5", isRequired: true, bindingTime: load}
      - {name: overVoltagePu, type: ScalarValues::Real, range: "1.0..=1.5", default: 1.2, isRequired: true, bindingTime: load}
      - {name: underVoltagePu, type: ScalarValues::Real, range: "0.1..=1.0", default: 0.45, isRequired: true, bindingTime: load}
      - {name: overFreqHz, type: ScalarValues::Real, range: "40.0..=70.0", default: 62.0, isRequired: true, bindingTime: load}
      - {name: underFreqHz, type: ScalarValues::Real, range: "40.0..=70.0", default: 56.5, isRequired: true, bindingTime: load}
      - {name: reconnectVoltageMaxPu, type: ScalarValues::Real, range: "1.0..=1.5", default: 1.05, isRequired: true, bindingTime: load}
      - {name: reconnectVoltageMinPu, type: ScalarValues::Real, range: "0.5..=1.0", default: 0.917, isRequired: true, bindingTime: load}
      - {name: reconnectFreqMaxHz, type: ScalarValues::Real, range: "40.0..=70.0", default: 60.1, isRequired: true, bindingTime: load}
      - {name: reconnectFreqMinHz, type: ScalarValues::Real, range: "40.0..=70.0", default: 59.5, isRequired: true, bindingTime: load}
      - {name: clearingMs, type: ScalarValues::Integer, range: "10..=5000", default: 160, isRequired: true, bindingTime: load}
constraints: []
---
