tc_TRS_BT_001() {
    local F="$1"; local B="$F/TC-TRS-BT-001"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "no project declaration"
    out=$("$SYSCRIBE" -m "$B/plain" binding-times --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.bindingTimes[].name]==["compile","load","runtime"]' >/dev/null \
        && pass "vocabulary is compile, load, runtime in that order" || fail "default vocabulary wrong: $out"

    _scn "the three default names stay valid in a refined project"
    out=$("$SYSCRIBE" -m "$B/refined" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E230' && fail "E230 raised for a built-in stage in a refined project" \
        || pass "no E230 for runtime in a project that declares deploy"
    out=$("$SYSCRIBE" -m "$B/refined" binding-times --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.bindingTimes[]|select(.builtin)|.name]==["compile","load","runtime"]' >/dev/null \
        && pass "the built-in stages keep their order" || fail "built-in order changed: $out"
}
