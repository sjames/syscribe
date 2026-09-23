tc_TRS_LINKTYPE_003() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-003"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local out
    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)
    _scn "permitted source and target validate cleanly"
    has "$out" "GoodCtl.md" "E63[34]" && fail "unexpected E633/E634 on GoodCtl" || pass "GoodCtl clean"
    _scn "a forbidden source type raises E633"
    has "$out" "REQ-LT3-002.md" "E633" && pass "E633 raised" || fail "E633 not raised"
    _scn "a forbidden target type raises E634"
    has "$out" "BadTargetCtl.md" "E634" && pass "E634 raised" || fail "E634 not raised"
}
