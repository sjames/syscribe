---
id: REQ-TRS-PROJ-009
type: Requirement
name: Tool shall report a machine parameter with no default once, check its expressions with a stand-in, and check them at the bound value under --config
status: draft
reqDomain: software
verificationMethod: test
---

A state machine can use a feature parameter in an expression (`{{Features::F.param}}`). `validate` checks the base model at each parameter's `value:`/`default:`. A parameter with neither, usually an `isRequired: true` profile value, used to leave the placeholder written and raise a parse error (`E882`/`E884`/`W933` "unexpected character '{'") that did not point at the cause.

The tool **shall**:

- in the base model, check each machine expression that uses such a parameter with a stand-in literal of the parameter's type (the first `enumValues` entry, `false` for a Boolean, `""` for a String, else `0`), so the rest of the expression is still checked and no parse error is raised for the placeholder;
- report each such parameter exactly once as `W248` on its declaring `FeatureDef`, naming the parameter, the machines that use it, and that `validate --config` checks them at a bound value;
- under `validate --config <C>` (and `--all-configs`), check the machines at the configuration's bound parameter values, so a bound value that is not a valid expression operand is reported against the machine;
- leave a parameter with a `value:` or `default:` exactly as before.

**Source:** GH #328, found while building the grid-tie inverter model (`model_gti/`).

**Acceptance criteria:**

- A placeholder referring to a parameter with no `value:`/`default:` gives exactly one `W248` naming it and `--config`; no `E882`/`E884`/`W933` parse error is raised for that expression.
- With `--config` binding the parameter, the machine is checked at the bound value.
- A parameter with a default keeps today's behaviour.
