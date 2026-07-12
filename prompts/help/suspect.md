# suspect — content-baseline suspect-link detection

## SYNOPSIS
    syscribe -m <root> suspect list
    syscribe -m <root> suspect accept <source> <target>
    syscribe -m <root> suspect accept --all

## DESCRIPTION
A **suspect link** is a trace link whose target changed after the link was last
reviewed. Review captures a **baseline** — a BLAKE3 hash of a canonical
projection of the target — stored on the *source* element in `traceBaselines:`
(the element that holds the link, per §12.1). Validation recomputes the target's
projection hash and compares it to the stored baseline; a mismatch surfaces as
warning **W090**.

The projection hashes the markdown body plus the normative frontmatter fields and
**excludes** editorial/presentation fields (`name`, `displayOrder`, `extRef`,
diagram layout, and `traceBaselines` itself), so cosmetic edits do not raise a
suspect flag.

Detection is **opt-in and additive**: a link with no baseline is never flagged
during `validate`. `suspect list` still surfaces unbaselined links so coverage
gaps stay discoverable.

## COMMANDS
    list
        Report every suspect link (baselined, target changed) and every
        unbaselined link. Read-only; output is deterministic.

    accept <source> <target>
        Re-baseline one link: capture <target>'s current projection hash into
        <source>'s `traceBaselines` map. <source> and <target> may be a stable id
        or a qualified name. It is an error if <target> is not referenced by any
        trace link on <source>.

    accept --all
        Re-baseline every currently-suspect link across the model in one pass
        (the W090 set). This reflects review: it clears outstanding suspect flags.

    accept --all-unbaselined
        Onboarding: baseline every link that has NO baseline yet (and whose target
        resolves). Never overwrites an existing baseline, so it cannot clear an
        outstanding suspect flag. Idempotent. Mutually exclusive with --all.

## VALIDATION
W090 is a Warning: draft-visible, non-fatal by default, and gateable in CI with
`--deny W090` (exit code 2 when a suspect link exists).

## EXAMPLES
    syscribe -m model/ suspect list
    syscribe -m model/ suspect accept TC-SCHED-BITMAP-001 REQ-SCHED-BITMAP-001
    syscribe -m model/ suspect accept --all
    syscribe -m model/ validate --deny W090

## SEE ALSO
    validate, trace, links, repos
