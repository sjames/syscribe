// Behavior alongside structure: `state def`/`action def` coexist in the same
// `Propulsion` package as the structural content in Structure.sysml /
// Interfaces.sysml. Since REQ-TRS-SYSMLV2-018/-019 they are part of the fixed
// mapped set and become real StateDef/ActionDef elements (originally this file
// demonstrated the coverage boundary, when they contributed nothing). See the
// README.
package Propulsion {

state def RotorHealthState;

action def MonitorRotorHealth;

}
