tc_TRS_BT_006() {
    local F="$1"; local B="$F/TC-TRS-BT-006"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }

    _scn "a spanning window is accepted and its keys are recognised"
    out=$("$SYSCRIBE" -m "$B/span" validate 2>&1 || true)
    printf '%s' "$out" | grep -qE 'E229|E230|E750|W047' && fail "finding raised for a valid window: $(printf '%s' "$out" | grep -E 'E229|E230|E750|W047')" \
        || pass "no binding-time or unknown-field finding"

    _scn "an item with neither field is unspecified"
    out=$("$SYSCRIBE" -m "$B/none" validate 2>&1 || true)
    printf '%s' "$out" | grep -qE 'E229|E230|E750' && fail "binding-time finding on an unspecified item" \
        || pass "no binding-time finding"
}
