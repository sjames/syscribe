tc_TRS_SM_007() {
    local F="$1"; local B="$F/TC-TRS-SM-007"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out hits

    out=$("$SYSCRIBE" -m "$B/incomplete" validate 2>/dev/null) || true
    hits=$(grep -F "| W929 |" <<<"$out" || true)

    _scn "a top-level transition without source raises W929"
    grep -qF "'resume'" <<<"$hits" && grep -qF "no \`source" <<<"$hits" \
        && pass "W929 names the resume transition's missing source" || fail "W929 missing for sourceless top-level transition: $hits"

    _scn "a transition without target raises W929"
    grep -qF "'halt'" <<<"$hits" && grep -qF "no \`target" <<<"$hits" \
        && pass "W929 names the halt transition's missing target" || fail "W929 missing for targetless transition: $hits"
    local n; n=$(grep -c . <<<"$hits" || true)
    [ "$n" = "2" ] && pass "exactly two W929 findings" || fail "expected 2 W929 findings, got $n: $hits"

    _scn "a nested transition with an implicit source raises no W929"
    out=$("$SYSCRIBE" -m "$B/clean" validate 2>/dev/null) || true
    grep -qF "| W929 |" <<<"$out" && fail "W929 raised on the well-formed machine" || pass "no W929 on the well-formed machine"

    _scn "W929 is draft-suppressed and gateable"
    out=$("$SYSCRIBE" -m "$B/draft" validate 2>/dev/null) || true
    grep -qF "| W929 |" <<<"$out" && fail "W929 raised on a draft machine" || pass "W929 draft-suppressed"
    local rc=0
    "$SYSCRIBE" -m "$B/incomplete" validate --deny W929 >/dev/null 2>&1 || rc=$?
    [ "$rc" -ne 0 ] && pass "--deny W929 exits non-zero" || fail "--deny W929 exited 0"
}
