tc_TRS_BT_002() {
    local F="$1"; local B="$F/TC-TRS-BT-002"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "a stage placed between two defaults"
    out=$("$SYSCRIBE" -m "$B/deploy" binding-times --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.bindingTimes[].name]==["compile","deploy","load","runtime"] and (.bindingTimes[1].binder=="installer")' >/dev/null \
        && pass "order is compile, deploy, load, runtime and the binder is kept" || fail "merged order wrong: $out"
    out=$("$SYSCRIBE" -m "$B/deploy" validate 2>&1 || true)
    printf '%s' "$out" | grep -qE 'E230|W750' && fail "finding raised for a valid declared stage" \
        || pass "a parameter may use the declared stage"

    _scn "an unknown stage is E230 and the message lists the effective vocabulary"
    out=$("$SYSCRIBE" -m "$B/deploybogus" validate 2>&1 || true)
    printf '%s' "$out" | grep 'E230' | grep -q 'compile, deploy, load, runtime' \
        && pass "E230 names the declared stage" || fail "E230 missing or without the vocabulary: $out"

    _scn "a stage with no placement"
    out=$("$SYSCRIBE" -m "$B/unplaced" validate 2>&1 || true)
    printf '%s' "$out" | grep 'W750' | grep -q 'loose' \
        && pass "W750 raised for an unplaceable stage" || fail "W750 not raised: $out"
}
