---
type: ADR
id: ADR-SYS-AGENTPOLICY-001
name: "The project's agent policy is declared in .syscribe.toml, enforced by the writers, and audited from git"
status: accepted
tags:
  - cli
  - agents
  - git
---

## Context

Conventions for automated editors — a planning item must not be `done` without evidence, `approved` is a
person's call, some paths are not for tools, certain checks must pass before work is called finished — were
enforced only by convention (GH #442). Nothing in the tool knew them, so a script or agent could break one
with a valid command. `ADR-SYS-PROVENANCE-001` gave every change an optional actor (`--by`) and put the record
in git; this decision builds the rules on top of that.

## Decision

1. **An opt-in `[agents]` table in `.syscribe.toml`** holds the rules: `protected_paths` (repository-relative
   globs), `human_only_status` (element type → statuses), `require_before_done` (`validate` and/or `evidence`),
   `require_by`, plus `actors` (git-author globs the audit counts as agents) and `audit_limit`. No table, no
   policy. A malformed entry is ignored on its own and reported (`W762`); a status that its type does not have
   is a malformed entry, so a typo cannot silently protect nothing.
2. **The policy applies to an agent.** A change is an agent's when it names an actor (`--by`) and is not
   declared a person's with `--as-human`. Over MCP every client is an agent and there is no `--as-human`: a
   person uses the CLI or edits the file. With `require_by`, a model-writing command naming neither is refused.
   `--as-human` is a declaration, recorded as a `Syscribe-As: human` trailer when committed — an honest label
   for who acted, not an access control.
3. **Refusals are exit 1 and say what to do.** Each names the rule and how a person proceeds. (`ADR-SYS-
   PROVENANCE-001` already decided there is no separate exit code for a refusal.) A refusal also happens under
   `--dry-run`, so a dry run reports what a real run would be refused.
4. **Protected paths are enforced twice.** The single-element writers check up front for a clear message; the
   write layer (`syscribe_model::fsio`) holds a guard — covering file writes, removals and directory creation —
   that every other write goes through as a backstop. A
   `move` is checked across every file it would rewrite, because renaming an unprotected element can edit a
   protected file that refers to it.
5. **`require_before_done` is judged on a candidate copy.** `evidence` needs the item to raise no `E719` once
   done; `validate` needs the change to introduce no validation error. Both reuse the validator rather than
   restating its rules.
6. **`validate` audits git for violations that bypassed the writers.** It reads the last `audit_limit` commits
   that touched the model; a commit is agent-made when it carries a `Syscribe-Actor` trailer (and no
   `Syscribe-As: human`) or its author matches `actors`. An agent-made commit that changed a protected path is
   `W760`; one that set a human-only status is `W761`. They are warnings, gated with `--deny`.
7. **The policy is shown where agents read.** `syscribe policy`, a `policy` section of `agent-guide`, and a line
   in the MCP `initialize` instructions state it in plain words.

## Rejected alternatives

- **A review-record requirement for human-only statuses** ("set without a matching review record"). It would add
  a structure to maintain; the git history already says who set a status, and an agent-made commit setting one
  is exactly what `W761` reports.
- **Treating "no `--by`" as an agent.** Most uses of the CLI are a person at a terminal; they would be refused
  by default. Naming an actor is how a change identifies itself as automated.
- **An `--as-human` for MCP.** An agent could always claim it, which would make the policy meaningless for the
  one channel that is entirely agents.
- **Enforcing only in the writers, or only in the audit.** The writers prevent; the audit finds what they could
  not see (a hand edit by an agent, a commit made outside the tool). Neither alone covers both.
- **Failing `validate` on a violation.** A past commit cannot be un-made by the person reading the report, so
  these are warnings a project chooses to gate.

## Consequences

- The policy is a guard rail, not a security boundary: a person can omit `--by`, an agent can pass
  `--as-human`, and history can be rewritten. It stops the mistakes automation makes, not a determined actor.
- Without `require_by`, an agent that forgets `--by` is not bound by the rules — the cost of leaving a person's
  unadorned command alone. `agent-guide` and the MCP instructions therefore tell agents to always pass `--by`,
  and `require_by` closes the gap for a project that wants it closed. With `require_by`, the requirement is
  enforced in the write layer as well as up front, so a writing command nobody listed is still refused.
- The audit sees only commits that touched the model directory, and only the last `audit_limit` of them. It
  cannot attribute uncommitted edits.
- A `protected_paths` glob is matched against the repository-relative path and the model-relative path, so
  either spelling works.
