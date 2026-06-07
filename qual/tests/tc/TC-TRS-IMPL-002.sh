tc_TRS_IMPL_002() {
    local F="$1"; local B="$F/TC-TRS-IMPL-001/model"
    SCENARIO_NAME="links shows the implementation path"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local lk; lk=$("$SYSCRIBE" -m "$B" links Arch::Sched 2>/dev/null || true)
    printf '%s' "$lk" | grep -qiF "implementedBy" && printf '%s' "$lk" | grep -qF "src/exists/" \
        && pass "links shows implementedBy path" || fail "links missing implementedBy path"

    SCENARIO_NAME="refs on a module path reports the owning element"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local rf; rf=$("$SYSCRIBE" -m "$B" refs "src/exists/" 2>/dev/null || true)
    printf '%s' "$rf" | grep -qF "Arch::Sched" \
        && pass "refs reports the owning architecture element" || fail "refs did not report the owner"

    SCENARIO_NAME="spec fields lists implementedBy"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" spec fields 2>/dev/null | grep -qF "implementedBy" \
        && pass "spec fields lists implementedBy" || fail "spec fields missing implementedBy"
}
