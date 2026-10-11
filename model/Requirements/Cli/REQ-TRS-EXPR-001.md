---
type: Requirement
id: REQ-TRS-EXPR-001
name: "A deterministic typed expression language with units evaluates guards, constraints and calculations"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - execution
---

`syscribe-model` shall provide one expression language over typed, dimension-checked values, and `syscribe eval` shall evaluate it (GH #273, part of #272). Guards, effects, constraint expressions and calculation bodies of the execution engine are all written in it.

## Behavior

- **Values.** A number carries a dimension (the seven SI base exponents) and is stored in SI base units; a boolean; a string, which is also how an enumeration literal is held. A number literal may be followed by a unit (`50 ms`, `3.3V`, `1.5 kWh`); the unit supplies the scale and the dimension. Units are the SI symbols and common long names, with the SI prefixes (`mm`, `kHz`, `uV`, `µs`/`μs`, `kΩ`, `mbar`), plus `min`, `h`, `L`, `bar`, `rpm`, `deg`, `kph`, `in`, `ft`, `Wh`/`kWh`, `t`. A literal that overflows to infinity is a parse error.
- **Operators**, loosest first: `if c then a else b` (also usable as an operand: `1 + if c then 2 else 3`); `or`/`||`; `and`/`&&`; `not`/`!`; `== != < <= > >=`; `+ -`; `* / %`; unary `-`; calls and parentheses. `and`/`or` short-circuit, so `false and (1/0 > 0)` is `false`. Functions: `abs min max sqrt floor ceil round pow`.
- **Dimensions are checked.** `+`, `-`, `%` and every comparison need equal dimensions (`1 s + 1 V` and `1 s < 5` are errors); `*` and `/` combine them (`3 V * 2 A` is `6 W`); `sqrt` halves even exponents; `pow` takes an integer exponent. Booleans and strings compare only with their own kind.
- **Names.** An identifier, optionally dotted (`gps.fix`) or qualified (`Features::Display.sizeInch`), is looked up in the environment; an unbound name is an error naming it, never `false` or `0`. An unbound qualified name that has `::` and no `.` (`ArmStatus::armed`) is an enumeration literal; an unbound dotted one (`Features::Display.sizeInch`) is an error. A string and an enumeration literal compare equal when equal, or when one is qualified and its last segment equals the other.
- **Strings** are `'single'` or `"double"` quoted. Division or remainder by zero, `sqrt` of a negative, an overflowing or underflowing `pow` and any non-finite result are errors, not infinities. Two quantities are equal when they differ by at most 1e-9 relative (and at most 1e-15 in SI base units), so float noise around zero is not a difference. Numbers print with up to 15 significant digits.
- **Errors** are typed (parse, unbound name, dimension, type, domain) and name the offending token or name.
- `parse_value` reads a binding value (`50ms`, `-3.5 V`, `true`, `'x'`, `ArmStatus::armed`); every identifier an expression uses can be listed, in first-use order.
- **`syscribe eval '<expr>' [--set name=value]... [--unit <u>] [--json]`** prints the value (SI base units, with the unit symbol when the dimension has one, else the dimension), converted with `--unit`; `--json` prints `{expression, type, value, unit, dimension}`. A parse, name, dimension or type error exits 1 with the message on stderr. The command reads no model and is deterministic: no clock, randomness or environment.
