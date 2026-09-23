tc_TRS_SET_001() {
    local F="$1"; local FX="$F/TC-TRS-SET-001"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    local tmp M out err rc
    tmp=$(mktemp -d)
    # fresh <name> — a scratch copy of the model (set writes files in place)
    fresh() { M="$tmp/$1"; cp -r "$FX/model" "$M"; }
    local REQ_REL="Reqs/REQ-SET-001.md" ORIG="$FX/model/Reqs/REQ-SET-001.md"

    # 1. an out-of-enum status value is refused and writes nothing
    _scn "an out-of-enum status value is refused and writes nothing"
    fresh s1
    err=$("$SYSCRIBE" -m "$M" set REQ-SET-001 status=bogus 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "exit code $rc (non-zero)" || fail "exit code 0 (expected non-zero)"
    printf '%s' "$err" | grep -qF "not a valid status for Requirement. Allowed: draft, review, approved, implemented, verified" \
        && pass "refusal names the Requirement status enum" || fail "refusal does not list the allowed values: $err"
    cmp -s "$ORIG" "$M/$REQ_REL" && pass "file unchanged" || fail "file modified by a refused set"

    # 2. a valid status value writes only that line
    _scn "a valid status value writes only that line"
    fresh s2
    out=$("$SYSCRIBE" -m "$M" set REQ-SET-001 status=approved 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0): $out"
    sed 's/^status: draft$/status: approved/' "$ORIG" > "$tmp/expected.md"
    cmp -s "$tmp/expected.md" "$M/$REQ_REL" \
        && pass "file is byte-identical to the original except the status: line" \
        || fail "file differs beyond the status: line: $(diff "$tmp/expected.md" "$M/$REQ_REL" || true)"
    grep -qxF 'name: "A requirement: with a colon"' "$M/$REQ_REL" \
        && pass "quoted name: field keeps its double quotes" || fail "name: quoting not preserved"
    grep -qxF "tags: ['alpha', \"beta\"]" "$M/$REQ_REL" \
        && pass "mixed-quote flow list preserved verbatim" || fail "tags: flow list not preserved"
    [ "$(sed -n '5p' "$M/$REQ_REL")" = "status: approved" ] \
        && pass "status: updated in place (line 5, field order preserved)" || fail "status: not updated in place"

    # 3. the target resolves by qualified name or stable id
    _scn "the target resolves by qualified name or stable id"
    fresh s3q
    "$SYSCRIBE" -m "$M" set Reqs::REQ-SET-001 status=approved >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && grep -qxF 'status: approved' "$M/$REQ_REL" \
        && pass "resolved and written by qualified name Reqs::REQ-SET-001" || fail "qualified-name form failed (exit $rc)"
    fresh s3i
    "$SYSCRIBE" -m "$M" set REQ-SET-001 status=approved >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && grep -qxF 'status: approved' "$M/$REQ_REL" \
        && pass "resolved and written by stable id REQ-SET-001" || fail "stable-id form failed (exit $rc)"
    fresh s3n
    err=$("$SYSCRIBE" -m "$M" set Nope::REQ-SET-001 status=approved 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && printf '%s' "$err" | grep -qF 'Element not found: Nope::REQ-SET-001' && cmp -s "$ORIG" "$M/$REQ_REL" \
        && pass "a non-resolving qualified name is refused and writes nothing" || fail "non-resolving target not refused (exit $rc): $err"

    # 4. marking a PlanningItem done warns on an under-verified achieves requirement but still writes
    _scn "marking a PlanningItem done warns on an under-verified achieves requirement but still writes"
    fresh s4
    err=$("$SYSCRIBE" -m "$M" set PI-SET-005 status=done 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "exit code 0 (warning is non-blocking)" || fail "exit code $rc (expected 0): $err"
    printf '%s' "$err" | grep -E '^warning:' | grep -qF "achieves 'REQ-SET-012' which has no active TestCase" \
        && pass "warning on stderr names the under-verified achieves requirement REQ-SET-012" \
        || fail "no W310-style warning naming REQ-SET-012: $err"
    grep -qxF 'status: done' "$M/Planning/PI-SET-005.md" \
        && pass "status: done written despite the warning" || fail "status: done not written"
    # negative side: an achieves requirement that meets the bar raises no warning
    err=$("$SYSCRIBE" -m "$M" set PI-SET-006 status=done 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && [ -z "$err" ] && grep -qxF 'status: done' "$M/Planning/PI-SET-006.md" \
        && pass "no warning when the achieves requirement has an active TestCase" \
        || fail "unexpected warning/failure for a verified achieves target (exit $rc): $err"

    # 5. --dry-run previews without writing
    _scn "--dry-run previews without writing"
    fresh s5
    out=$("$SYSCRIBE" -m "$M" set REQ-SET-001 status=approved --dry-run 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0)"
    printf '%s\n' "$out" | grep -qxF -- '--- a/Reqs/REQ-SET-001.md' \
        && printf '%s\n' "$out" | grep -qxF -- '+++ b/Reqs/REQ-SET-001.md' \
        && printf '%s\n' "$out" | grep -qE '^@@ -[0-9]+(,[0-9]+)? \+[0-9]+(,[0-9]+)? @@' \
        && pass "unified diff headers and hunk printed" || fail "no unified diff headers: $out"
    printf '%s\n' "$out" | grep -qxF -- '-status: draft' && printf '%s\n' "$out" | grep -qxF -- '+status: approved' \
        && pass "diff shows -status: draft / +status: approved" || fail "diff missing the status change: $out"
    cmp -s "$ORIG" "$M/$REQ_REL" && pass "file unchanged" || fail "--dry-run wrote the file"

    rm -rf "$tmp"
}
