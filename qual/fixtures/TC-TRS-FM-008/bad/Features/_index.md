---
type: FeatureModel
name: Features
featureTree:
  - {name: Protection, id: FEAT-Q8-PROT, mandatory: true, groupKind: optional}
  - {name: Protection.Afci, id: FEAT-Q8-AFCI, groupKind: optional}
  - name: Protection.Rsd
    id: FEAT-Q8-RSD
    groupKind: optional
    requires: [Protection.Afcy]
  - {name: Mppt, id: FEAT-Q8-MPPT, mandatory: true, groupKind: optional}
  - name: Mppt.GlobalScan
    id: FEAT-Q8-GSCAN
    groupKind: optional
    parameters:
      - {name: scanPeriodS, type: ScalarValues::Integer, default: 600, isRequired: true, bindingTime: load}
---
