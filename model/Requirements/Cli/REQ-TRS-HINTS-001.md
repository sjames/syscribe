---
type: Requirement
id: REQ-TRS-HINTS-001
name: "mistakes are answered with the nearest valid choice and the command that does the job"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - validation
---

The tool shall point to the dedicated command or the nearest valid value when a user mistypes, or edits by hand what a command maintains (GH #439).

## Behavior

- An unknown command lists the three nearest commands with their purpose; an unknown option, `set` field, `set status=` value or `template` type is answered with `Did you mean '…'?`. Only the message changes; exit codes are unchanged.
- `syscribe explain <code>` (and `help <code>`) prints a finding's catalogue explanation and the commands it points to, without a model; an unknown code exits `1`.
- `validate --hints` adds informational findings, off by default and never affecting the exit status unless selected with `--deny`:
  - `I910` — `claimedBy` set without `claimedAt` (`claim` always writes both), naming `claim` and `release`;
  - `I911` — the same `evidence:` entry listed twice;
  - `I912` — `status` outside the type's vocabulary, naming the nearest valid value and the `set` command that applies it.
- The `status:` vocabulary is defined once (`syscribe_model::hints::status_vocabulary`) and shared by `set` and the hints.
