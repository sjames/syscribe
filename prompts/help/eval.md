# eval — evaluate an expression of the execution engine's language

## SYNOPSIS
    syscribe eval '<expression>' [--set name=value]... [--unit <unit>] [--json]

## DESCRIPTION
Evaluates one expression in the typed, dimension-checked language that guards,
constraints and calculations of the execution engine are written in. It reads no
model and is deterministic.

    syscribe eval '50 ms + 1 s'                     # 1.05 s
    syscribe eval '3 V * 2 A'                       # 6 W
    syscribe eval 'altitude >= 30 m' --set altitude=42m
    syscribe eval 'mode == Mode::auto' --set mode=auto

Values are numbers with a dimension (stored in SI base units), booleans and strings
(also enumeration literals). A number may be followed by a unit: SI symbols and
long names with prefixes (`mm`, `kHz`, `uV`), `min`, `h`, `L`, `bar`, `rpm`, `deg`,
`kph`, `in`, `ft`, `Wh`/`kWh`, `t`.

Operators, loosest first: `if c then a else b`; `or` `||`; `and` `&&`; `not` `!`;
`== != < <= > >=`; `+ -`; `* / %`; unary `-`. `and`/`or` short-circuit. Functions:
`abs min max sqrt floor ceil round pow`, and `exp ln sin cos tanh` over a
dimensionless argument (radians for `sin`/`cos`; `ln` needs a positive one). `+`, `-`, `%` and comparisons need equal
dimensions (`1 s + 1 V` is an error); `*` and `/` combine them. `if c then a else b`
can also be an operand. Equality is tolerant of float noise (relative 1e-9, absolute
1e-15 in SI base units). Numbers print with up to 15 significant digits.

Temperatures: `degC` and `degF` are absolute readings (`23.4 degC` is `296.55 K`) and `delta_degC`, `delta_degF`,
`delta_K` are differences. The minus of `-20 degC` belongs to the number, so a reading below freezing is written
as it reads (`-20 degC` is `253.15 K`, `-40 degC == -40 degF`); a reading below absolute zero (`-300 degC`) is an
error, as is negating a reading expression (`-(20 degC)`: a reading has no negative). reading - reading is a
difference, reading +/- difference is a reading; adding two readings or scaling one is an error.

Names are looked up among the `--set` bindings; a dotted name (`gps.fix`) or a
qualified one (`Features::Display.sizeInch`) is one name. An unbound name is an
error. An unbound `A::b` without a dot is an enumeration literal.

## OPTIONS
    --set name=value   Bind a name: `3`, `50ms`, `-3.5 V`, `true`, `'text'`, `Enum::lit`.
    --unit <unit>      Express a numeric result in <unit> (`ms`, `kWh`); the dimension must match.
    --json             {expression, type, value, unit, dimension}.

Exit 1 with a message on stderr for a parse, name, dimension, type or domain error
(division by zero, `sqrt` of a negative, `ln` of a non-positive, a non-finite result); nothing on stdout.
