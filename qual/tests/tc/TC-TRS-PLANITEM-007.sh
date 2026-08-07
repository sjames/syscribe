tc_TRS_PLANITEM_007() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-007"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. a resolving blockedBy naming another PlanningItem validates cleanly
    _scn "a resolving blockedBy naming another PlanningItem validates cleanly"
    printf '%s' "$out" | grep 'ResolvingBlockedBy.md' | grep -qE 'E72[01]' \
        && fail "unexpected blockedBy error on ResolvingBlockedBy.md" \
        || pass "ResolvingBlockedBy.md raises no blockedBy error"

    # 2. a resolving blockedBy naming a non-PlanningItem validates cleanly
    _scn "a resolving blockedBy naming a non-PlanningItem validates cleanly"
    printf '%s' "$out" | grep 'NonPlanningItemBlocker.md' | grep -qE 'E72[01]' \
        && fail "unexpected blockedBy error on NonPlanningItemBlocker.md" \
        || pass "NonPlanningItemBlocker.md raises no blockedBy error (permissive resolution)"

    # 3. a dangling blockedBy is rejected
    _scn "a dangling blockedBy is rejected"
    printf '%s' "$out" | grep 'Dangling.md' | grep -q 'E720' \
        && pass "E720 raised for a dangling blockedBy" || fail "E720 not raised for a dangling blockedBy"

    # 4. a 2-node blockedBy cycle is detected gracefully
    _scn "a 2-node blockedBy cycle is detected gracefully"
    local cyc_out
    cyc_out=$("$SYSCRIBE" -m "$FX/cycle2" validate 2>&1 || true)
    printf '%s' "$cyc_out" | grep -q 'E721' \
        && pass "E721 raised for a 2-node blockedBy cycle" || fail "E721 not raised for a 2-node blockedBy cycle"

    # 5. a non-empty blockedBy while not status: blocked is a warning
    _scn "a non-empty blockedBy while not status: blocked is a warning"
    if printf '%s' "$out" | grep 'StaleBlocked.md' | grep -q 'W308'; then
        pass "W308 raised for a stale blockedBy"
    else
        fail "W308 not raised for a stale blockedBy"
    fi
    printf '%s' "$out" | grep 'StaleBlocked.md' | grep -qE 'E72[01]' \
        && fail "stale blockedBy must never escalate to an error" \
        || pass "stale blockedBy stays a warning, not an error"

    # 6. status: blocked with no blockedBy raises nothing
    _scn "status: blocked with no blockedBy raises nothing"
    printf '%s' "$out" | grep 'BlockedNoBlockers.md' | grep -qE 'E72[01]|W308' \
        && fail "unexpected blockedBy-related finding on BlockedNoBlockers.md" \
        || pass "BlockedNoBlockers.md (blocked, no blockedBy:) raises nothing"
}
