# tasks — common jobs and the command for each

Run `syscribe help <command>` for any of these, `syscribe help <Type>` for an element type,
or `syscribe help --search <keyword>` to find a command by what it does.

## Create and edit safely
    Find the next free id            syscribe -m model/ next-id REQ-FX
    Start from a skeleton            syscribe template Requirement          (no model needed)
    Check a reference resolves       syscribe -m model/ check-ref <ref>
    Change status / one field        syscribe -m model/ set <id> status=approved [--dry-run]
    Add evidence to a planning item  syscribe -m model/ set <PI-id> evidence.add ref=<TC-id>
    Record the delivering commit     syscribe -m model/ set <PI-id> evidence.add commit=<sha>
    Commits that delivered an item   syscribe -m model/ log --item <PI-id>
    Rename or move, fixing refs      syscribe -m model/ move <src> <dest> [--dry-run]
    Claim / release shared work      syscribe -m model/ claim <PI-id> --by <agent>  /  release <PI-id>
    Record a decision you need       syscribe -m model/ set <PI-id> question.add text="Which UART?" [recommended=UART2]
    Reserve a decision to a person   answerBy: human on the question (or [agents] human_only_answers = true); see `help policy`
    Who may answer / who answered    syscribe -m model/ questions --json     (answerBy, answerProvenance)
    Anchor a requirement item        write `1. {#grant-keys} ...` in the REQ body (syscribe help Requirement)
    Cite a requirement item          write REQ-X#grant-keys in prose; check: check-ref REQ-X#grant-keys
    Cite a planning question         write PI-X#Q3 in prose; check: check-ref PI-X#Q3

## Check
    Validate the model               syscribe -m model/ validate
    Only what a change introduced    syscribe -m model/ validate --new-since <rev>
    Explain a finding code           syscribe explain E106
    Is everything traced and tested  syscribe -m model/ audit

## Understand a change
    What changed between revisions   syscribe -m model/ changes <revA> [<revB>]
    What a change affects            syscribe -m model/ impact <id>
    Trace one requirement            syscribe -m model/ trace <REQ-id>
    Who verifies a requirement       syscribe -m model/ who-verifies <REQ-id>
    Who cites one requirement item   syscribe -m model/ refs REQ-X#grant-keys   (or impact REQ-X#grant-keys)
    Baseline an item-level link      syscribe -m model/ suspect accept <src-id> REQ-X#grant-keys

## Evidence and variants
    Unplanned tests (no item names)  syscribe -m model/ coverage --json     (testCasesWithoutPlanningEvidence; W319 with [planning])
    Record test results              syscribe -m model/ ingest-results <file>
    Requirement verdicts             syscribe -m model/ requirement-coverage --config <CONF>
    Compare two variants             syscribe -m model/ diff --config <A> --config <B>
    Validate every variant (CI)      syscribe -m model/ validate --all-configs

## Learn the tool
    What can I do with a type        syscribe help PlanningItem
    Which command does X             syscribe help --search evidence
    Format reference                 syscribe spec
