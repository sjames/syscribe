# change-impact — which Configurations an edit would affect

## SYNOPSIS
    syscribe -m <root> change-impact <element> [--set <field>=<yaml>]...
                                     [--param <name> --param-set <field>=<yaml>]... [--json]

## DESCRIPTION
Answers "if I change this feature, constraint or parameter range, which products are affected?" before the edit is
made. The edit is applied to a **candidate copy** of the model by the guarded-write engine in dry-run mode, so
nothing on disk changes. For each Configuration the live and the candidate variants are compared:

- validity: its errors before and after, whether it **became invalid** or **became valid**, and the new and resolved
  errors (code, file, message);
- projection: the elements that **enter** or **leave** the variant.

The result also carries the unified diff of the edit, the element's `location` (file and line, ready to jump to),
and the model-wide findings the edit would add.

`--set field=value` sets a frontmatter field of the element (the value is YAML, so `requires=[Features::A]` and
`mandatory=true` work; `null` removes the key). `--param NAME --param-set field=value` edits one entry of the
element's `parameters:` list, for example a range: `--param motorKV --param-set range="900..1100"`.

The JSON (schema `change-impact`) is the payload of the MCP tool `change_impact` and the LSP request
`syscribe/changeImpact`. `diff --config A --config B --json` gains a `changes` list with the same `location`.

## OPTIONS
    --set <f>=<yaml>        field to set on the element; repeatable
    --param <name>          the parameter entry to edit
    --param-set <f>=<yaml>  field of that parameter to set; repeatable
    --json                  machine-readable output
An unknown element exits 1; a missing edit or a malformed value exits 2.

## EXAMPLES
    syscribe -m examples/staged-configuration/model/ change-impact FEAT-UAV-PROP --param motorKV --param-set range="900..1000"
    syscribe -m examples/staged-configuration/model/ change-impact FEAT-UAV-TELEM --set mandatory=true --json

## SEE ALSO
    diff, propagate, explain-feature, feature-check, residual
