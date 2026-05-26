---
type: Diagram
name: AvionicsBayIBD
diagramKind: IBD
svgMode: inline
subject: UAV::Avionics::AvionicsBay
shapes:
  s-boundary: {ref: "UAV::Avionics::AvionicsBay", kind: boundary}
  s-fc: {ref: "UAV::Avionics::FlightController", kind: Part, parent: s-boundary}
  s-imu: {ref: "UAV::Avionics::IMU", kind: Part, parent: s-boundary}
  s-gps: {ref: "UAV::Avionics::GPSReceiver", kind: Part, parent: s-boundary}
  s-fc-power: {ref: "UAV::Avionics::FlightController::powerIn", kind: Port}
  s-fc-ctrl: {ref: "UAV::Avionics::FlightController::controlOut", kind: Port}
  s-fc-telem: {ref: "UAV::Avionics::FlightController::telemetryOut", kind: Port}
edges:
  e-power: {ref: "UAV::Avionics::AvionicsBay", source: s-fc-power, target: s-imu, kind: flowConnection}
  e-ctrl: {ref: "UAV::Avionics::AvionicsBay", source: s-fc-ctrl, target: s-gps, kind: flowConnection}
---

Internal Block Diagram showing the internal structure of the AvionicsBay, including part usages for FlightController, IMU, and GPSReceiver, with their interconnecting ports and flow connections.

```svg
<svg xmlns="http://www.w3.org/2000/svg" xmlns:sysml="urn:syscribe:1.0"
     width="640" height="360" viewBox="0 0 640 360">

  <!-- AvionicsBay boundary -->
  <g id="s-boundary" sysml:ref="UAV::Avionics::AvionicsBay">
    <use href="#sym-boundary" x="20" y="20" width="600" height="320"/>
    <text font-size="11" fill="#333" x="30" y="40">«part def» AvionicsBay</text>
  </g>

  <!-- FlightController block -->
  <g id="s-fc" sysml:ref="UAV::Avionics::FlightController" transform="translate(60,100)">
    <use href="#sym-PartDef" width="160" height="80"/>
    <text class="elem-label" x="80" y="22" text-anchor="middle" font-size="9" fill="#555">«part»</text>
    <text class="elem-name"  x="80" y="42" text-anchor="middle" font-size="11" font-weight="bold">FlightController</text>
  </g>

  <!-- IMU block -->
  <g id="s-imu" sysml:ref="UAV::Avionics::IMU" transform="translate(350,80)">
    <use href="#sym-PartDef" width="120" height="60"/>
    <text class="elem-label" x="60" y="18" text-anchor="middle" font-size="9" fill="#555">«part»</text>
    <text class="elem-name"  x="60" y="35" text-anchor="middle" font-size="11" font-weight="bold">IMU</text>
  </g>

  <!-- GPSReceiver block -->
  <g id="s-gps" sysml:ref="UAV::Avionics::GPSReceiver" transform="translate(350,220)">
    <use href="#sym-PartDef" width="140" height="60"/>
    <text class="elem-label" x="70" y="18" text-anchor="middle" font-size="9" fill="#555">«part»</text>
    <text class="elem-name"  x="70" y="35" text-anchor="middle" font-size="11" font-weight="bold">GPSReceiver</text>
  </g>

  <!-- Ports on FlightController -->
  <g id="s-fc-power" sysml:ref="UAV::Avionics::FlightController::powerIn">
    <use href="#sym-port" x="52" y="175" width="16" height="16"/>
    <text font-size="9" x="52" y="202" text-anchor="middle">powerIn</text>
  </g>
  <g id="s-fc-ctrl" sysml:ref="UAV::Avionics::FlightController::controlOut">
    <use href="#sym-port" x="212" y="120" width="16" height="16"/>
    <text font-size="9" x="240" y="131">controlOut</text>
  </g>
  <g id="s-fc-telem" sysml:ref="UAV::Avionics::FlightController::telemetryOut">
    <use href="#sym-port" x="212" y="155" width="16" height="16"/>
    <text font-size="9" x="240" y="166">telemetryOut</text>
  </g>

  <!-- Flow connection FC controlOut to GPSReceiver -->
  <line id="e-ctrl" sysml:ref="UAV::Avionics::AvionicsBay"
        x1="228" y1="128" x2="350" y2="250"
        stroke="#3a6ea5" stroke-width="1.5" stroke-dasharray="5,3"
        marker-end="url(#arrow-flow)"/>

  <!-- Flow connection FC powerIn from boundary edge (left) -->
  <line id="e-power" sysml:ref="UAV::Avionics::AvionicsBay"
        x1="20" y1="183" x2="60" y2="183"
        stroke="#3a6ea5" stroke-width="1.5"
        marker-end="url(#arrow-flow)"/>
</svg>
```
