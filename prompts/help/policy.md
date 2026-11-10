# policy — the project's agent policy, and how this invocation is treated

## SYNOPSIS
    syscribe -m <root> policy [--json]

## DESCRIPTION
Prints the `[agents]` policy of `.syscribe.toml`: the rules an automated editor must respect, in plain
words, any entries that were ignored because they were malformed (`W762`), and whether the current
invocation would be treated as an agent or as a person. With no `[agents]` table it says so; the policy
is opt-in.

    [agents]
    protected_paths     = ["model/Safety/**"]       # tools may not write these (repository-relative globs)
    human_only_status   = { Requirement = ["approved"], ADR = ["accepted"] }
    human_only_answers  = true                      # PlanningItem questions are answered by a person (unless `answerBy: any`)
    require_before_done = ["validate", "evidence"]  # before an agent sets a PlanningItem done
    require_by          = true                      # every model-writing command names its actor
    sidecar_writers     = ["ingest-results"]        # commands that may write `.syscribe/**` although it is protected
    actors              = ["agent-*"]               # git authors the audit counts as agents
    audit_limit         = 200                       # recent commits `validate` inspects

**Who is an agent.** A change is an agent's when it names an actor (`--by`, or an MCP tool call — MCP
writes always count) and is not declared a person's with `--as-human`. The policy applies to agents; a
person edits the file or runs the command with `--as-human` (recorded as a `Syscribe-As: human` trailer
when committed). `--as-human` is a declaration, not a proof: this is a guard rail, not a security boundary.

**What is enforced.** A refused change exits 1 with the rule named and how a person can proceed — also
under `--dry-run`, so a dry run reports what a real run would be refused:
- `protected_paths`: no command writes a matching path (a `move` is checked across every file it rewrites);
  protecting `.syscribe/**` (the recorded evidence) also stops `ingest-results`, so when ingest is automated
  name it in `sidecar_writers = ["ingest-results"]`: that command alone may then write the sidecar (not
  `.syscribe/scripts/`), while every other agent edit of recorded verdicts stays refused with exit 1;
- `human_only_status`: `set <id> status=<value>` and the MCP write tools refuse a listed status;
- `require_before_done`: setting a PlanningItem `done` needs resolving `evidence` (no `E719`) and/or that the
  change introduce no validation error;
- `require_by`: a model-writing command with neither `--by` nor `--as-human` is refused.
- `human_only_answers`: an agent may not answer a PlanningItem question — set its `answer`, `answeredBy` or
  `answeredAt`, move it to `answered`, create it answered, or lift the reservation (`set … question.answer`,
  the MCP `question`/`update_element`/`apply_changes` tools, any writer). A question's own `answerBy: human | any`
  wins over this default, and `answerBy: human` reserves a question even with no `[agents]` table. An agent
  still records what it needs decided with `question.add`; `questions --json` shows `answerBy` per question.

**What is audited.** `validate` reads the last `audit_limit` commits that touched the model and reports
violations that reached the files by another route: `W760` an agent-made commit changed a protected path;
`W761` one set a human-only status. A commit is agent-made when it carries a `Syscribe-Actor` trailer (and no
`Syscribe-As: human`) or its author matches `actors`. `W762` reports a malformed `[agents]` entry. All are
warnings: gate them with `validate --deny W760,W761`. `W323` (a PlanningItem finding) reports a reserved question
whose answer an agent wrote: `answeredBy` matches `actors`, or the commit that last changed the answer is
agent-made by the same rule. An answer git cannot attribute (uncommitted, not versioned) raises nothing.

## OPTIONS
    --json    Machine-readable output (schemaVersion 1.0).

## EXAMPLES
    syscribe -m model/ policy
    syscribe -m model/ --by agent-1 policy --json

## SEE ALSO
    agent-guide, validate, log, set
