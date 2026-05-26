---
type: Diagram
name: FlightStatesMachineD
diagramKind: StateMachine
svgMode: inline
subject: Behavior::FlightStates
shapes:
  s-initial: {ref: "Behavior::FlightStates", kind: initial}
  s-disarmed: {ref: "Behavior::FlightStates::disarmed", kind: state}
  s-armed: {ref: "Behavior::FlightStates::armed", kind: state}
  s-takingOff: {ref: "Behavior::FlightStates::takingOff", kind: state}
  s-flying: {ref: "Behavior::FlightStates::flying", kind: state}
  s-landing: {ref: "Behavior::FlightStates::landing", kind: state}
  s-fault: {ref: "Behavior::FlightStates::fault", kind: state}
edges:
  e-init: {source: s-initial, target: s-disarmed, kind: transition}
  e-arm: {source: s-disarmed, target: s-armed, kind: transition}
  e-takeoff: {source: s-armed, target: s-takingOff, kind: transition}
  e-armfault: {source: s-armed, target: s-fault, kind: transition}
  e-fly: {source: s-takingOff, target: s-flying, kind: transition}
  e-tofault: {source: s-takingOff, target: s-fault, kind: transition}
  e-land: {source: s-flying, target: s-landing, kind: transition}
  e-flyfault: {source: s-flying, target: s-fault, kind: transition}
  e-disarm: {source: s-landing, target: s-disarmed, kind: transition}
  e-lndfault: {source: s-landing, target: s-fault, kind: transition}
  e-recover: {source: s-fault, target: s-disarmed, kind: transition}
---

State machine diagram for the UAV flight states lifecycle, showing normal operational transitions from disarmed through takeoff, flight, and landing, as well as fault transitions from any active state and recovery back to disarmed.

```svg
<svg xmlns="http://www.w3.org/2000/svg" xmlns:sysml="urn:syscribe:1.0"
     width="700" height="520" viewBox="0 0 700 520">

  <!-- Initial pseudo-state -->
  <g id="s-initial" sysml:ref="Behavior::FlightStates">
    <use href="#sym-initial" x="175" y="20" width="20" height="20"/>
  </g>

  <!-- disarmed -->
  <g id="s-disarmed" sysml:ref="Behavior::FlightStates::disarmed">
    <use href="#sym-state" x="120" y="60" width="140" height="44"/>
    <text x="190" y="87" text-anchor="middle" font-size="12">disarmed</text>
  </g>

  <!-- armed -->
  <g id="s-armed" sysml:ref="Behavior::FlightStates::armed">
    <use href="#sym-state" x="120" y="150" width="140" height="44"/>
    <text x="190" y="177" text-anchor="middle" font-size="12">armed</text>
  </g>

  <!-- takingOff -->
  <g id="s-takingOff" sysml:ref="Behavior::FlightStates::takingOff">
    <use href="#sym-state" x="120" y="240" width="140" height="44"/>
    <text x="190" y="267" text-anchor="middle" font-size="12">takingOff</text>
  </g>

  <!-- flying -->
  <g id="s-flying" sysml:ref="Behavior::FlightStates::flying">
    <use href="#sym-state" x="120" y="330" width="140" height="44"/>
    <text x="190" y="357" text-anchor="middle" font-size="12">flying</text>
  </g>

  <!-- landing -->
  <g id="s-landing" sysml:ref="Behavior::FlightStates::landing">
    <use href="#sym-state" x="120" y="420" width="140" height="44"/>
    <text x="190" y="447" text-anchor="middle" font-size="12">landing</text>
  </g>

  <!-- fault (right column) -->
  <g id="s-fault" sysml:ref="Behavior::FlightStates::fault">
    <use href="#sym-state" x="480" y="240" width="140" height="44"/>
    <text x="550" y="267" text-anchor="middle" font-size="12">fault</text>
  </g>

  <!-- Transition arrows -->
  <!-- initial to disarmed -->
  <line id="e-init" x1="185" y1="40" x2="185" y2="60"
        stroke="#333" stroke-width="1.5" marker-end="url(#arrow-open)"/>

  <!-- disarmed to armed: accept ControlCommand / guard armStatus -->
  <line id="e-arm" x1="190" y1="104" x2="190" y2="150"
        stroke="#333" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="195" y="132" font-size="9" fill="#555">ControlCommand [disarmed]</text>

  <!-- armed to takingOff -->
  <line id="e-takeoff" x1="190" y1="194" x2="190" y2="240"
        stroke="#333" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="195" y="222" font-size="9" fill="#555">ControlCommand [armed]</text>

  <!-- takingOff to flying -->
  <line id="e-fly" x1="190" y1="284" x2="190" y2="330"
        stroke="#333" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="195" y="312" font-size="9" fill="#555">[altitude >= target]</text>

  <!-- flying to landing -->
  <line id="e-land" x1="190" y1="374" x2="190" y2="420"
        stroke="#333" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="195" y="402" font-size="9" fill="#555">ControlCommand [RTH]</text>

  <!-- landing to disarmed (curved back left) -->
  <path id="e-disarm" d="M 120,442 Q 40,442 40,82 Q 40,82 120,82"
        fill="none" stroke="#333" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="4" y="262" font-size="9" fill="#555" transform="rotate(-90,22,262)">[altitude &lt;= 0.1]</text>

  <!-- fault to disarmed (curved right-side) -->
  <path id="e-recover" d="M 550,240 Q 550,100 260,82"
        fill="none" stroke="#888" stroke-width="1.5" stroke-dasharray="4,3"
        marker-end="url(#arrow-open)"/>
  <text x="420" y="135" font-size="9" fill="#888">ControlCommand [cleared]</text>

  <!-- armed to fault -->
  <path id="e-armfault" d="M 260,172 Q 400,172 480,262"
        fill="none" stroke="#c0392b" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="360" y="195" font-size="9" fill="#c0392b">[imu.accel &gt; 30]</text>

  <!-- takingOff to fault -->
  <line id="e-tofault" x1="260" y1="262" x2="480" y2="262"
        stroke="#c0392b" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="320" y="255" font-size="9" fill="#c0392b">[altitude stall]</text>

  <!-- flying to fault -->
  <path id="e-flyfault" d="M 260,352 Q 400,352 480,280"
        fill="none" stroke="#c0392b" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="340" y="375" font-size="9" fill="#c0392b">[battery &lt; 10%]</text>

  <!-- landing to fault -->
  <path id="e-lndfault" d="M 260,442 Q 500,442 550,284"
        fill="none" stroke="#c0392b" stroke-width="1.5" marker-end="url(#arrow-open)"/>
  <text x="440" y="458" font-size="9" fill="#c0392b">[descent rate &gt; 3m/s]</text>
</svg>
```
