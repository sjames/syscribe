tc_TRS_PLANITEM_009() {
    local F="$1"; local M="$F/TC-TRS-PLANITEM-009/model"
    local out; local rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "template PlanningItem prints a ready-to-fill skeleton"
    out=$("$SYSCRIBE" -m "$M" template PlanningItem 2>&1); rc=$?
    printf '%s' "$out" | grep -q 'type: PlanningItem' \
        && pass "skeleton declares type: PlanningItem" \
        || fail "skeleton missing 'type: PlanningItem': $out"
    printf '%s' "$out" | grep -qE 'id: PI-' \
        && pass "skeleton seeds a PI-* id" \
        || fail "skeleton missing a PI-* id: $out"
    printf '%s' "$out" | grep -q 'status:' \
        && pass "skeleton includes a status field" \
        || fail "skeleton missing a status field: $out"
    [ "$rc" -eq 0 ] && pass "command exits 0" || fail "expected exit 0, got $rc"

    _scn "template is case-insensitive on the type name"
    out=$("$SYSCRIBE" -m "$M" template planningitem 2>&1)
    printf '%s' "$out" | grep -q 'type: PlanningItem' \
        && pass "lowercase 'planningitem' still resolves" \
        || fail "lowercase 'planningitem' did not resolve: $out"

    _scn "PlanningItem is listed as a known native type"
    out=$("$SYSCRIBE" -m "$M" template NotAType 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep 'Native elements' | grep -q 'PlanningItem' \
        && pass "Native elements line lists PlanningItem" \
        || fail "Native elements line does not list PlanningItem: $out"
    [ "$rc" -ne 0 ] && pass "command exits non-zero for an unknown type" \
        || fail "expected non-zero exit for an unknown type"
}
