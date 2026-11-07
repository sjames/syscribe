# questions — every PlanningItem's open decisions in one list

## SYNOPSIS
    syscribe -m <root> questions [--open] [--status open|answered|deferred] [--item <PI>] [--json]

## DESCRIPTION
Lists the inline `questions:` of every `PlanningItem` — the decisions awaiting an answer — with
the owning item (`PI-X#Q1`), status, text, the recommendation and the answer. A question is
detected from the field, never inferred from prose, and is private to its item.

An entry is `{id, text, recommended?, status, answer?, answeredBy?, answeredAt?, answerBy?}` with `status`
`open | answered | deferred`. `answerBy: human | any` says who may answer it: `human` reserves the
decision to a person (an owner), so an agent's `question.answer` is refused (`human_only_answers`) and an
answer an agent wrote anyway is `W323`. With no `answerBy`, the default is `any`, or `human` when
`.syscribe.toml` sets `[agents] human_only_answers = true`; a question's own `answerBy` always wins.
Reserved questions are marked *(for a person)* in the table. Record and resolve them with `set`:

    syscribe -m <root> set <PI> question.add text="Which UART?" recommended=UART2
    syscribe -m <root> set <PI> question.answer Q1 text=UART2 by=alice
    syscribe -m <root> set <PI> question.defer Q1

`list PlanningItem --open-questions` keeps the items with an open question; `show <PI>` prints
its questions. `validate` checks them (`E724`–`E727`) and warns when an item is `done` with an
open one (`W317`); `deferred` is the explicit way to leave one unanswered. An open question does
not make an item `blocked` — `blockedBy:` stays for dependencies.

## OPTIONS
    --open            Only open questions (same as --status open).
    --status <s>      Only questions with this status.
    --item <PI>       Only this item's questions (id or qualified name); unknown or not a PlanningItem exits 1.
    --json            Machine-readable output (schemaVersion 1.0). Each question also carries `answerBy`
                      (`human` | `any`, the default applied) and, when git records the commit that last
                      changed its answer, `answerProvenance: {by, agent, at, commit}` — `by` is the
                      `Syscribe-Actor` (else the author), `agent` whether that commit was an agent's.

## EXAMPLES
    # against the bundled PlanningItem example (examples/planning-item/model/)
    syscribe -m examples/planning-item/model/ questions --open
    syscribe -m examples/planning-item/model/ questions --item PI-RTH-DOCS-001 --json
    syscribe -m examples/planning-item/model/ set PI-RTH-DOCS-001 question.add text="Which locale?" --dry-run
    syscribe -m examples/planning-item/model/ set PI-RTH-DOCS-001 question.answer Q1 text=percent by=alice --dry-run
    syscribe -m examples/planning-item/model/ set PI-RTH-DOCS-001 question.defer Q1 --dry-run

## SEE ALSO
    set, steps, list, show, validate
