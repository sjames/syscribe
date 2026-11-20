# done — whether a PlanningItem may be called done, and exactly what is missing

## SYNOPSIS
    syscribe -m <root> done <PI-id> [--json]

## DESCRIPTION
The one check every agent harness runs before it stops (GH #485). It judges the model as it would be with the item
`done` and reports each check:

- `steps` — a composite's child items are all done;
- `questions` — none of the item's questions is open (one for a person is marked);
- `blockers` — every `blockedBy:` is met;
- `evidence` — the item has the evidence a done leaf needs (`E719`) and the requirements it achieves have the
  verification `W310` asks for;
- `validate` — the model has no errors;
- `gates` — no `[gates]` stereotype condition or coverage threshold is tripped;
- `trace-code` — every code unit is named or baselined (skipped without a `[traceCode]` table);
- `status` — whether this invocation may set `done`. When `[agents] human_only_status` reserves it for a person and
  the invocation is an agent's (`--by`), the result is a handoff: everything holds, but a person sets the status.

The command never writes. When everything holds it prints the `set <PI> status=done` command to run.

## OPTIONS
    --json   `{schemaVersion, item, ready, needsPerson, missing, checks: [{check, result, detail}]}`;
             `result` is pass, fail, skip or handoff.

Exit `0` ready, `2` something is missing, `1` an unknown item or a usage error.

## EXAMPLES
    # against the bundled PlanningItem example (examples/planning-item/model/)
    syscribe -m examples/planning-item/model/ done PI-RTH-DOCS-001
    syscribe -m examples/planning-item/model/ --by agent-7 done PI-RTH-DOCS-001 --json

## SEE ALSO
    next, set, validate, steps, questions, policy
