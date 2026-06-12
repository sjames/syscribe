# scripts — run user-authored Rhai extension scripts

## SYNOPSIS
    syscribe -m <root> scripts list [--json]
    syscribe -m <root> scripts run <command> [--json]
    syscribe -m <root> scripts validate [--deny <codes>] [--max-warnings <n>] [--warnings-as-errors] [--json]

## DESCRIPTION
Extension scripts are user-authored `.rhai` files stored **outside** the model in
a configured scripts directory. They are *tooling*, not model content: they are
never parsed as model elements and are **not** run by the built-in `validate`
pass — the qualification boundary stays crisp.

Scripts run in a sandboxed, resource-limited, deterministic Rhai engine: no
filesystem, network, clock, randomness, environment, or process access; `eval` is
disabled; the module resolver is confined to the scripts directory; and a runaway
script is aborted by an operation budget rather than hanging. The only side
effect is text to stdout/stderr.

A script registers either a **command** (run with `scripts run`) or a **check**
(run with `scripts validate`):

    register_command(name, description, fn);   // fn(model) -> string
    register_check(name, description, fn);      // fn(model) emits finding(...)

A file may register many of either, or none (a pure, importable **library**).
A duplicate `name` across scripts is a load error.

## CONFIGURATION
The scripts directory is `[scripts] path` in `<model_root>/.syscribe.toml`,
resolved relative to the model root; the default is `.syscribe/scripts/`. The
directory is the Rhai module-import root, so a script can reuse a library:

    import "lib/helpers" as h;

An absent scripts directory is not an error — the model simply has no extensions.

## MODEL API (read-only)
    model.elements()                 // all elements
    model.elements_of_type("Type")   // elements of a given type
    model.find("id-or-qname")        // one element, or unit () if unknown

    e.qname  e.id  e.name  e.title  e.type  e.status  e.doc  e.tags
    e.supertype  e.typed_by  e.subsets  e.satisfies  e.verifies  e.derived_from
    e.verified_by  e.derived_children  e.refined_by  e.allocated_from   // computed
    e.field("key")        // any frontmatter field, or unit () if absent
    e.custom_fields       // a map of the element's custom_fields

    finding(element, code, severity, message)   // severity: error|warning|info
    print(value)          // stdout
    eprint(value)         // stderr

## SUBCOMMANDS
    list        Enumerate every registered command and check (name, kind,
                description, source file). `--json` for a machine-readable array.
    run         Invoke a command's function and print its returned string.
                `--json` if the command returns structured data. An unknown name
                exits non-zero; a check name reports that it is a check.
    validate    Run every registered check and print findings as `<check>/<code>`
                with the source script. Exit 0 clean, 1 on an error-severity
                finding, 2 on a tripped gate. Reuses `--deny`/`--max-warnings`/
                `--warnings-as-errors`. Independent of the built-in `validate`.

## EXAMPLES
    syscribe -m model/ scripts list
    syscribe -m model/ scripts run coverage
    syscribe -m model/ scripts validate --deny naming/NOASIL

## SEE ALSO
    validate, export
