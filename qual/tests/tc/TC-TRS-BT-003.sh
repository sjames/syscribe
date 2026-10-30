tc_TRS_BT_003() {
    local F="$1"; local B="$F/TC-TRS-BT-003"
    local out n
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "a cyclic order"
    out=$("$SYSCRIBE" -m "$B/cyclic" validate 2>&1 || true)
    n=$(printf '%s' "$out" | grep -c 'W750' || true)
    [ "$n" -eq 2 ] && pass "each cyclic entry is reported once" || fail "expected 2 W750 (a and b), got $n: $out"
    out=$("$SYSCRIBE" -m "$B/cyclic" binding-times --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.bindingTimes[].name]==["compile","load","runtime"]' >/dev/null \
        && pass "the effective order is the default" || fail "order changed: $out"

    _scn "two stages in the same position"
    out=$("$SYSCRIBE" -m "$B/samegap" validate 2>&1 || true)
    printf '%s' "$out" | grep 'W750' | grep -q "same position as 'b'" && printf '%s' "$out" | grep 'W750' | grep -q "same position as 'a'" \
        && pass "each warning names the other stage" || fail "ambiguity not reported both ways: $out"
    out=$("$SYSCRIBE" -m "$B/samegap" binding-times --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.bindingTimes[].name]==["compile","load","runtime"]' >/dev/null \
        && pass "neither stage is placed" || fail "a stage was placed: $out"
    out=$("$SYSCRIBE" -m "$B/ordered" binding-times --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.bindingTimes[].name]==["compile","a","b","load","runtime"]' >/dev/null \
        && pass "naming one as the other's neighbour places both" || fail "ordered pair not placed: $out"

    _scn "an unknown key"
    out=$("$SYSCRIBE" -m "$B/unknownkey" validate 2>&1 || true)
    printf '%s' "$out" | grep 'W750' | grep -q 'mystery' \
        && pass "W750 names the unknown key" || fail "W750 not raised: $out"
    out=$("$SYSCRIBE" -m "$B/unknownkey" binding-times --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.bindingTimes[].name]|index("x")==null' >/dev/null \
        && pass "the entry is ignored whole" || fail "entry kept: $out"

    _scn "a bad entry does not discard the rest of the vocabulary"
    out=$("$SYSCRIBE" -m "$B/partial" binding-times --json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '[.bindingTimes[].name]==["compile","deploy","load","runtime"]' >/dev/null \
        && pass "the valid stage survives" || fail "valid stage lost: $out"
    out=$("$SYSCRIBE" -m "$B/partial" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E230' && fail "E230 for the surviving stage" || pass "its use is still valid"
}
