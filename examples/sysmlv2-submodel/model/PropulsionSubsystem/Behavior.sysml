// Coverage-boundary demonstration (REQ-TRS-SYSMLV2-007): `state def`/`action def`
// are real, legally-parsed SysML v2 constructs, but behavior modeling is outside
// this feature's fixed mapped-element set. They parse without error and coexist
// in the same `Propulsion` package as the mapped structural content in
// Structure.sysml/Interfaces.sysml, but contribute zero RawElements to the graph
// — parse-broad, map-narrow. See the README for where to look (or rather, where
// *not* to find anything) to confirm this.
package Propulsion {

state def RotorHealthState;

action def MonitorRotorHealth;

}
