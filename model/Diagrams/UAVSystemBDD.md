---
type: Diagram
name: UAVSystemBDD
diagramKind: BDD
svgMode: inline
subject: UAV::UAVSystem
shapes:
  s-uavsystem: {ref: "UAV::UAVSystem", kind: PartDef}
  s-airframe: {ref: "UAV::Airframe", kind: PartDef}
  s-propulsion: {ref: "UAV::Propulsion::PropulsionSystem", kind: PartDef}
  s-avionics: {ref: "UAV::Avionics::AvionicsBay", kind: PartDef}
  s-power: {ref: "UAV::Power::PowerSystem", kind: PartDef}
  s-payload: {ref: "UAV::Payload::PayloadBay", kind: PartDef}
  s-gcs: {ref: "GroundStation::GroundControlStation", kind: PartDef}
edges:
  e-airframe: {ref: "UAV::UAVSystem::airframe", source: s-uavsystem, target: s-airframe, kind: composition}
  e-propulsion: {ref: "UAV::UAVSystem", source: s-uavsystem, target: s-propulsion, kind: composition}
  e-avionics: {ref: "UAV::UAVSystem", source: s-uavsystem, target: s-avionics, kind: composition}
  e-power: {ref: "UAV::UAVSystem", source: s-uavsystem, target: s-power, kind: composition}
  e-payload: {ref: "UAV::UAVSystem", source: s-uavsystem, target: s-payload, kind: composition}
---

Block Definition Diagram showing the top-level structural decomposition of the UAVSystem into its constituent subsystem definitions.

```svg
<svg xmlns="http://www.w3.org/2000/svg" xmlns:sysml="urn:syscribe:1.0"
     width="760" height="400" viewBox="0 0 760 400">

  <!-- UAVSystem (root) -->
  <g id="s-uavsystem" sysml:ref="UAV::UAVSystem" transform="translate(280,20)">
    <use href="#sym-PartDef" width="200" height="60"/>
    <text class="elem-label" x="100" y="22" text-anchor="middle" font-size="10" fill="#555">«part def»</text>
    <text class="elem-name"  x="100" y="42" text-anchor="middle" font-size="13" font-weight="bold">UAVSystem</text>
  </g>

  <!-- Airframe -->
  <g id="s-airframe" sysml:ref="UAV::Airframe" transform="translate(20,160)">
    <use href="#sym-PartDef" width="120" height="50"/>
    <text class="elem-label" x="60" y="18" text-anchor="middle" font-size="9" fill="#555">«part def»</text>
    <text class="elem-name"  x="60" y="35" text-anchor="middle" font-size="11" font-weight="bold">Airframe</text>
  </g>

  <!-- PropulsionSystem -->
  <g id="s-propulsion" sysml:ref="UAV::Propulsion::PropulsionSystem" transform="translate(160,160)">
    <use href="#sym-PartDef" width="130" height="50"/>
    <text class="elem-label" x="65" y="18" text-anchor="middle" font-size="9" fill="#555">«part def»</text>
    <text class="elem-name"  x="65" y="35" text-anchor="middle" font-size="11" font-weight="bold">PropulsionSystem</text>
  </g>

  <!-- AvionicsBay -->
  <g id="s-avionics" sysml:ref="UAV::Avionics::AvionicsBay" transform="translate(310,160)">
    <use href="#sym-PartDef" width="120" height="50"/>
    <text class="elem-label" x="60" y="18" text-anchor="middle" font-size="9" fill="#555">«part def»</text>
    <text class="elem-name"  x="60" y="35" text-anchor="middle" font-size="11" font-weight="bold">AvionicsBay</text>
  </g>

  <!-- PowerSystem -->
  <g id="s-power" sysml:ref="UAV::Power::PowerSystem" transform="translate(450,160)">
    <use href="#sym-PartDef" width="120" height="50"/>
    <text class="elem-label" x="60" y="18" text-anchor="middle" font-size="9" fill="#555">«part def»</text>
    <text class="elem-name"  x="60" y="35" text-anchor="middle" font-size="11" font-weight="bold">PowerSystem</text>
  </g>

  <!-- PayloadBay -->
  <g id="s-payload" sysml:ref="UAV::Payload::PayloadBay" transform="translate(590,160)">
    <use href="#sym-PartDef" width="120" height="50"/>
    <text class="elem-label" x="60" y="18" text-anchor="middle" font-size="9" fill="#555">«part def»</text>
    <text class="elem-name"  x="60" y="35" text-anchor="middle" font-size="11" font-weight="bold">PayloadBay</text>
  </g>

  <!-- Composition arrows from UAVSystem (bottom center ~380,80) to each child top -->
  <!-- diamond-tail at UAVSystem, open arrowhead at child -->
  <line x1="380" y1="80" x2="80"  y2="160" stroke="#333" stroke-width="1.5" marker-end="url(#arrow-composition)" marker-start="url(#arrow-composition)"/>
  <line x1="380" y1="80" x2="225" y2="160" stroke="#333" stroke-width="1.5" marker-end="url(#arrow-composition)"/>
  <line x1="380" y1="80" x2="370" y2="160" stroke="#333" stroke-width="1.5" marker-end="url(#arrow-composition)"/>
  <line x1="380" y1="80" x2="510" y2="160" stroke="#333" stroke-width="1.5" marker-end="url(#arrow-composition)"/>
  <line x1="380" y1="80" x2="650" y2="160" stroke="#333" stroke-width="1.5" marker-end="url(#arrow-composition)"/>
</svg>
```
