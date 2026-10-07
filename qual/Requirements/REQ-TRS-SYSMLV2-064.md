---
id: REQ-TRS-SYSMLV2-064
type: Requirement
name: "Standard-library recognition covers the full ScalarValues membership and the common ISQ/SI names including compound units"
status: verified
reqDomain: software
verificationMethod: test
---

The built-in \`ScalarValues\` inventory shall contain every primitive a SysML v2 source commonly names (\`Boolean\`, \`String\`, \`NumericalValue\`, \`Number\`, \`Complex\`, \`Real\`, \`Rational\`, \`Integer\`, \`Natural\`, \`ScalarValue\`) so none raises \`W043\`; the ISQ quantity and SI unit tables shall add torque, angular velocity/acceleration, density, volume flow and the \`deg\`/\`rpm\`/\`Nm\` units; and a compound unit expression built from table units with \`*\`, \`/\` and \`^\` (\`m/s\`, \`N*m\`, \`m^2\`, \`kg*m/s^2\`) shall have a derived dimension so \`W044\` checks it. Recognised references raise no \`E111\`/\`W043\`/\`W404\` finding in ingested submodels or in \`export-sysml\` output, which emits them verbatim. This is a small documented table, not a library import; unrecognised names keep their existing handling.

**Source:** `REQ-TRS-SYSMLV2-064` (product model), `ADR-SYS-SYSMLV2-001`.
