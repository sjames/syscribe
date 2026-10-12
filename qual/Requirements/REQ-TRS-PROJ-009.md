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

- in the base model, check each machine expression that uses such a parameter with a stand-in value of the parameter's type, substituted the way a bound value is (the first `enumValues` entry, `false` for a Boolean, the empty string for a String, else the lower bound of `range:` or `0`, followed by the unit for a `|unit` placeholder), so the rest of the expression is still checked and no parse error is raised for the placeholder;
- report each such parameter exactly once as `W248` on its declaring `FeatureDef`, naming the parameter, the machines that use it, and that `validate --config` checks them at a bound value; not for a `bindingTime: runtime` parameter (it cannot be bound), a `derivedFrom:`/`bindTo:` parameter (it takes its value elsewhere), nor inside the `--config` lens (an unbound required parameter is `W017` there);
- under `validate --config <C>` and for each Configuration of `--all-configs`, check the machines at that configuration's own bound parameter values (Configurations that select the same features are told apart by their bindings), so a bound value that is not a valid expression operand is reported against the machine;
- check machines at bound values without changing what suspect-link and baseline hashing read, so the lens never makes a link suspect;
- leave a parameter with a `value:` or `default:` exactly as before.

**Source:** GH #328, found while building the grid-tie inverter model (`model_gti/`).

**Acceptance criteria:**

- A placeholder referring to a parameter with no `value:`/`default:` gives exactly one `W248` naming it and `--config`; no `E882`/`E884`/`W933` parse error is raised for that expression.
- With `--config` binding the parameter, the machine is checked at the bound value; `--all-configs` judges each of two Configurations with the same features at its own bindings.
- A parameter with a default keeps today's behaviour.
