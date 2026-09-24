tc_TRS_PLANITEM_011() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-011"
    local tmp M out err rc before after

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # claim/release mutate files: always work on a scratch copy of the fixture.
    tmp=$(mktemp -d)
    cp -r "$FX/model" "$tmp/model"
    M="$tmp/model"
    local UNCL="$M/Planning/Unclaimed.md" CLMD="$M/Planning/ClaimedTodo.md"
    local DONE="$M/Planning/Done.md" BLKD="$M/Planning/ClaimedBlocked.md"
    local ISO='[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z'

    # 8 (run first, while the fixture is pristine). --dry-run writes nothing
    _scn "--dry-run previews without writing for both commands"
    before=$(cat "$UNCL")
    out=$("$SYSCRIBE" -m "$M" claim PI-P11-UNCLAIMED-001 --by agent-1 --dry-run 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "claim --dry-run exits 0" || fail "claim --dry-run exit $rc"
    printf '%s' "$out" | grep -q '^+claimedBy: agent-1' \
        && pass "claim --dry-run prints a diff adding claimedBy" || fail "claim --dry-run printed no +claimedBy diff line"
    [ "$(cat "$UNCL")" = "$before" ] && pass "claim --dry-run left Unclaimed.md unchanged" \
        || fail "claim --dry-run wrote to Unclaimed.md"
    before=$(cat "$BLKD")
    out=$("$SYSCRIBE" -m "$M" release PI-P11-BLOCKED-001 --dry-run 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "release --dry-run exits 0" || fail "release --dry-run exit $rc"
    printf '%s' "$out" | grep -q '^-claimedBy: agent-1' \
        && pass "release --dry-run prints a diff removing claimedBy" || fail "release --dry-run printed no -claimedBy diff line"
    [ "$(cat "$BLKD")" = "$before" ] && pass "release --dry-run left ClaimedBlocked.md unchanged" \
        || fail "release --dry-run wrote to ClaimedBlocked.md"

    # 6. release on an unclaimed item is a no-op
    _scn "release on an unclaimed item is a no-op"
    before=$(cat "$UNCL")
    out=$("$SYSCRIBE" -m "$M" release PI-P11-UNCLAIMED-001 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "release on unclaimed item exits 0" || fail "release on unclaimed item exit $rc"
    printf '%s' "$out" | grep -q 'nothing to release' \
        && pass "prints 'nothing to release'" || fail "no 'nothing to release' message"
    [ "$(cat "$UNCL")" = "$before" ] && pass "Unclaimed.md not rewritten" || fail "Unclaimed.md was modified"

    # 1. claim sets claimedBy and claimedAt
    _scn "claim sets claimedBy and claimedAt"
    grep -q '^claimed' "$UNCL" && fail "precondition: Unclaimed.md already has claim fields" \
        || pass "precondition: Unclaimed.md has no claim fields"
    out=$("$SYSCRIBE" -m "$M" claim PI-P11-UNCLAIMED-001 --by agent-1 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "claim exits 0" || fail "claim exit $rc"
    grep -qx 'claimedBy: agent-1' "$UNCL" && pass "claimedBy: agent-1 written" || fail "claimedBy not written"
    grep -qE "^claimedAt: \"?$ISO\"?\$" "$UNCL" && pass "claimedAt written as an ISO-8601 UTC timestamp" \
        || fail "claimedAt not written (or not ISO-8601)"

    # 7. claimedBy visible in show and list --json
    _scn "claimedBy is visible in show and list --json"
    out=$("$SYSCRIBE" -m "$M" show PI-P11-UNCLAIMED-001 2>&1 || true)
    printf '%s' "$out" | grep -qE '^\| \*\*claimedBy\*\* \| agent-1 \|' \
        && pass "show surfaces claimedBy = agent-1" || fail "show does not surface claimedBy"
    out=$("$SYSCRIBE" -m "$M" list PlanningItem --json 2>/dev/null || true)
    local jv
    jv=$(printf '%s' "$out" | python3 -c 'import json,sys
d=json.load(sys.stdin)
print(next((e.get("claimedBy") for e in d if e.get("id")=="PI-P11-UNCLAIMED-001"),"<missing>"))' 2>/dev/null || echo "<badjson>")
    [ "$jv" = "agent-1" ] && pass "list --json has claimedBy = agent-1 for PI-P11-UNCLAIMED-001" \
        || fail "list --json claimedBy = '$jv' (expected agent-1)"
    jv=$(printf '%s' "$out" | python3 -c 'import json,sys
d=json.load(sys.stdin)
print(next((e.get("claimedBy") for e in d if e.get("id")=="PI-P11-DONE-001"),"<missing>"))' 2>/dev/null || echo "<badjson>")
    [ "$jv" = "None" ] && pass "list --json claimedBy is null for the unclaimed PI-P11-DONE-001" \
        || fail "list --json claimedBy for PI-P11-DONE-001 = '$jv' (expected null)"

    # 2. claim refuses when already claimed by someone else
    _scn "claim refuses when already claimed by someone else"
    before=$(cat "$CLMD")
    err=$("$SYSCRIBE" -m "$M" claim PI-P11-CLAIMED-001 --by agent-2 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "claim by agent-2 exits non-zero ($rc)" || fail "claim by agent-2 exited 0"
    printf '%s' "$err" | grep -q "already claimed by 'agent-1'" \
        && pass "error names the current claimant agent-1" || fail "error does not name the current claimant"
    [ "$(cat "$CLMD")" = "$before" ] && pass "ClaimedTodo.md unchanged" || fail "ClaimedTodo.md was modified"
    grep -qx 'claimedBy: agent-1' "$CLMD" && pass "claimedBy still agent-1" || fail "claimedBy changed"

    # 3. re-claiming with the same agent is allowed
    _scn "re-claiming with the same agent is allowed"
    out=$("$SYSCRIBE" -m "$M" claim PI-P11-CLAIMED-001 --by agent-1 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "re-claim by agent-1 exits 0" || fail "re-claim by agent-1 exit $rc"
    grep -qx 'claimedBy: agent-1' "$CLMD" && pass "claimedBy still agent-1" || fail "claimedBy lost after re-claim"
    grep -q '2026-01-01T00:00:00Z' "$CLMD" \
        && fail "claimedAt not refreshed by re-claim" || pass "claimedAt refreshed (stale 2026-01-01 value replaced)"
    [ "$(grep -c '^claimedBy:' "$CLMD")" -eq 1 ] && pass "no duplicated claimedBy key" || fail "claimedBy key duplicated"

    # 4. claim refuses on an already-done item
    _scn "claim refuses on an already-done item"
    before=$(cat "$DONE")
    err=$("$SYSCRIBE" -m "$M" claim PI-P11-DONE-001 --by agent-1 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "claim on done item exits non-zero ($rc)" || fail "claim on done item exited 0"
    printf '%s' "$err" | grep -q 'nothing to claim' \
        && pass "error says 'nothing to claim'" || fail "error does not say 'nothing to claim'"
    [ "$(cat "$DONE")" = "$before" ] && pass "Done.md unchanged" || fail "Done.md was modified"

    # 5. release clears both fields regardless of status
    _scn "release clears both fields regardless of status"
    out=$("$SYSCRIBE" -m "$M" release PI-P11-BLOCKED-001 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "release exits 0" || fail "release exit $rc"
    grep -q '^claimedBy:' "$BLKD" && fail "claimedBy still present" || pass "claimedBy removed"
    grep -q '^claimedAt:' "$BLKD" && fail "claimedAt still present" || pass "claimedAt removed"
    grep -qx 'status: blocked' "$BLKD" && pass "status: blocked unchanged" || fail "status changed by release"

    rm -rf "$tmp"
}
