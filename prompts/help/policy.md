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
    sidecar_audit       = "exempt"                  # or "command": how `validate` audits commits of that sidecar
    actors              = ["agent-*"]               # git authors the audit counts as agents
    audit_limit         = 200                       # recent commits `validate` inspects

**Who is an agent.** A change is an agent's when it names an actor (`--by`, or an MCP tool call — MCP
writes always count) and is not declared a person's with `--as-human`. The policy applies to agents; a
person edits the file or runs the command with `--as-human` (recorded as a `Syscribe-As: human` trailer
when committed). `--as-human` is a declaration, not a proof: this is a guard rail, not a security boundary.

**What is enforced.** A refused change exits 1 with the rule named and how a person can proceed — also
under `--dry-run`, so a dry run reports what a real run would be refused:
- `protected_paths`: no command writes a matching path (a `move` is checked across every file it rewrites);
  protecting `.syscribe/**` (the recorded evidence) also stops `ingest-results`: see **Protecting recorded
  evidence** below for how to let an automated ingest through;
- `human_only_status`: `set <id> status=<value>` and the MCP write tools refuse a listed status;
- `require_before_done`: setting a PlanningItem `done` needs resolving `evidence` (no `E719`) and/or that the
  change introduce no validation error;
- `require_by`: a model-writing command with neither `--by` nor `--as-human` is refused.
- `human_only_answers`: an agent may not answer a PlanningItem question — set its `answer`, `answeredBy` or
  `answeredAt`, move it to `answered`, create it answered, or lift the reservation (`set … question.answer`,
  the MCP `question`/`update_element`/`apply_changes` tools, any writer). A question's own `answerBy: human | any`
  wins over this default, and `answerBy: human` reserves a question even with no `[agents]` table. An agent
  still records what it needs decided with `question.add`; `questions --json` shows `answerBy` per question.

**Protecting recorded evidence.** The verdicts `ingest-results` records live in `.syscribe/` (`results.json`,
`results-history.json`, …). Two different wishes are easy to confuse, and each is one setting in the `[agents]`
table of `.syscribe.toml` (a person edits this file; an agent does not change the policy that binds it — if you
are blocked, say which setting you need, or record the question):

| You want | Set |
|---|---|
| Agents may not hand-edit recorded verdicts, and CI may not ingest either (rare) | `protected_paths = [".syscribe/**"]` |
| Agents may not hand-edit recorded verdicts, but an automated ingest must work | the line above **plus** `sidecar_writers = ["ingest-results"]` |
| ... and `validate` should also catch a hand-edited sidecar committed by an agent | the two above **plus** `sidecar_audit = "command"` |
| No protection | nothing: leave `.syscribe/` out of `protected_paths` |

Copy-paste, the common case (CI ingests; nobody hand-edits):

    [agents]
    protected_paths = [".syscribe/**"]
    sidecar_writers = ["ingest-results"]

and the stricter one (the pipeline commits through `ingest-results --commit`):

    [agents]
    protected_paths = [".syscribe/**"]
    sidecar_writers = ["ingest-results"]
    sidecar_audit   = "command"

How `sidecar_writers` works: only the commands it names may write under `.syscribe/` (not `.syscribe/scripts/`,
which is executable tooling and is never exempt); every other agent write there is refused, exit 1, with the rule
named. `--dry-run` writes nothing, the run history (`--run`) included, and a protected history is refused before
the sidecar is written. It applies to a change that names its actor (`--by`), like the rest of the policy.

How `sidecar_audit` works: it only matters when `sidecar_writers` is set, because `validate` reads git history and
git cannot tell an `ingest-results` write from a hand edit.
- `"exempt"` (the default): a commit of a `.syscribe/` file raises no `W760`. Simple, and right when you do not need
  to catch a hand-edited sidecar. The cost: a verdict edited by hand and committed by an agent is not caught.
- `"command"`: such a commit raises `W760` unless its `Syscribe-Command` trailer names a listed writer. `--commit`
  writes that trailer, so `ingest-results ... --by <you> --commit` is clean and a plain `git commit` of the
  sidecar is reported. The cost: a pipeline that ingests and then commits with its own `git commit` is reported
  too, so it must commit through `--commit` (or use `"exempt"`). The trailer is text in a commit message, so a
  person can fake it: this is a guard rail, not proof.
- Any other value is `W762` and `"exempt"` applies.

What you will see, and what to do (as an agent, you do not edit `.syscribe.toml` yourself):

| Message | Meaning | Who does what |
|---|---|---|
| `Cannot write results sidecar: refused by [agents] protected_paths ...` and a hint naming `sidecar_writers` | the sidecar is protected and no writer is named | a person adds `sidecar_writers = ["ingest-results"]`, or you stop and report it |
| the same, ending "only x may write it" | writers are named but not this command | use the named command, or a person adds yours |
| `W760 ... sidecar_audit = "command"` on `.syscribe/results.json` | a commit of the sidecar was not made by a listed writer | commit the ingest with `--commit`, or a person sets `sidecar_audit = "exempt"` |
| `W762 ... sidecar_audit must be "exempt" or "command"` | a typo in the value | a person fixes it |

**What is audited.** `validate` reads the last `audit_limit` commits that touched the model and reports
violations that reached the files by another route: `W760` an agent-made commit changed a protected path
(the evidence sidecar is treated as `sidecar_audit` says when `sidecar_writers` is set: `exempt` by default, which
also hides a hand-edited sidecar committed by an agent, or `command`); `W761` one set a human-only status. A commit is agent-made when it carries a `Syscribe-Actor` trailer (and no
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
