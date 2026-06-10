tc_TRS_PLAN_006() {
    local F="$1"; local M="$F/TC-TRS-PLAN-005/model"   # shared fixture

    SCENARIO_NAME="matrix --plan restricts rows to in-scope requirements"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$M" matrix --plan TP-NAV-001 2>/dev/null || true)
    printf '%s' "$out" | grep -q "REQ-NAV-002" && pass "in-scope leaf is a row" || fail "in-scope leaf missing"
    printf '%s' "$out" | grep -q "REQ-SAFE-001" && fail "out-of-scope requirement leaked into rows" \
        || pass "out-of-scope requirement excluded"

    SCENARIO_NAME="verification-depth --plan is scoped and exits 0"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" verification-depth --plan TP-NAV-001 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "verification-depth --plan exit 0" || fail "verification-depth --plan exit $rc"
    printf '%s' "$out" | grep -q "REQ-SAFE-001" && fail "out-of-scope requirement leaked" \
        || pass "verification-depth scoped to plan"

    SCENARIO_NAME="--plan composes with --config"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" matrix --plan TP-NAV-001 --config CONF-DELIVERY-001 2>/dev/null || true)
    printf '%s' "$out" | grep -q "1 configurations" && pass "config lens reduces to one column" \
        || fail "composition did not reduce columns"

    SCENARIO_NAME="an unknown plan id exits 1"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" matrix --plan TP-NOPE-001 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "unknown plan exits 1" || fail "unknown plan exit $rc (expected 1)"

    # --- audit --plan: scoped verdict, full-model validation (GH #40) ----------
    SCENARIO_NAME="audit --plan scopes the verdict to the plan (no escaping-ref artifacts)"
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    local A="$F/TC-TRS-PLAN-006/auditscope"
    "$SYSCRIBE" -m "$A" audit >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 2 ] && pass "whole-model audit FAILs (out-of-scope E102 exists)" \
        || fail "whole-model audit exit $rc (expected 2)"
    "$SYSCRIBE" -m "$A" audit --plan TP-SCOPE-001 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "audit --plan excludes the out-of-scope error (PASS)" \
        || fail "audit --plan TP-SCOPE-001 exit $rc (expected 0)"
    "$SYSCRIBE" -m "$A" audit --plan TP-BADSCOPE-001 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 2 ] && pass "audit --plan counts an in-scope error (FAIL)" \
        || fail "audit --plan TP-BADSCOPE-001 exit $rc (expected 2)"

    # The previously-artifacting fixture (in-scope req with an out-of-scope
    # breakdownAdr ADR) now audits clean under --plan (no phantom E311).
    "$SYSCRIBE" -m "$M" audit --plan TP-NAV-001 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "audit --plan clean when refs escape the scope (no E311 artifact)" \
        || fail "audit --plan TP-NAV-001 exit $rc (expected 0)"
}
