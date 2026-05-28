---
type: PartDef
name: Track Circuit Interface
domain: hardware
features:
  - name: sectionLength
    typedBy: ScalarValues::Real
    unit: m
  - name: frequencyHz
    typedBy: ScalarValues::Real
    unit: Hz
---

The Track Circuit Interface is an audio-frequency track circuit input module. It detects the presence of a train on a section of track by monitoring the AC shunt current at the receiver end of the track circuit. When a train axle bridges the two running rails, the resulting low-resistance shunt reduces the receiver voltage below the detection threshold, indicating an occupied section.

The module outputs a vital "occupied/clear" binary signal to the vital processor. This output is fail-safe: a loss of receiver voltage for any reason (train presence, broken rail, loss of feed voltage, or cable fault) is reported as occupied, never as clear. The vital processor will not clear any signal whose route includes an occupied track section.

Track circuit frequency is configured per section to avoid mutual interference from adjacent sections. The module supports audio-frequency track circuits in the 83 Hz to 830 Hz range as specified by EN 50238.

The interface is designed to reject interference from traction harmonics and adjacent track circuits using narrow-band filters tuned to the section frequency.
