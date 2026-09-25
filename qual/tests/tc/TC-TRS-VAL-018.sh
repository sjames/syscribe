tc_TRS_VAL_018() {
    local F="$1"; local B="$F/TC-TRS-VAL-018"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out hits

    _scn "a ConfirmationMeasure status outside the documented set raises E924"
    out=$("$SYSCRIBE" -m "$B/cm" validate 2>/dev/null) || true
    hits=$(grep -F "| E924 |" <<<"$out" || true)
    grep -qF "CM-VAL-001.md" <<<"$hits" && pass "E924 on CM-VAL-001" || fail "E924 not raised for CM-VAL-001"
    grep -qF "'approved'" <<<"$hits" && pass "E924 names the value" || fail "E924 does not name 'approved': $hits"
    grep -qF "CM-VAL-002.md" <<<"$hits" && fail "E924 wrongly fired for status: completed" || pass "no E924 for completed"

    _scn "a Security Level outside 1-4 raises E925"
    out=$("$SYSCRIBE" -m "$B/sl" validate 2>/dev/null) || true
    hits=$(grep -F "| E925 |" <<<"$out" || true)
    grep -qF "ZN-VAL-001.md" <<<"$hits" && grep -qF "achievedSL 7" <<<"$hits" \
        && pass "E925 for Zone achievedSL 7" || fail "E925 missing for Zone achievedSL 7: $hits"
    grep -qF "CD-VAL-001.md" <<<"$hits" && grep -qF "achievedSL 0" <<<"$hits" \
        && pass "E925 for Conduit achievedSL 0" || fail "E925 missing for Conduit achievedSL 0: $hits"
    grep -qF "Pump.md" <<<"$hits" && grep -qF "targetSL 5" <<<"$hits" \
        && pass "E925 for PartDef targetSL 5" || fail "E925 missing for PartDef targetSL 5: $hits"
    grep -qF "ZN-VAL-002.md" <<<"$hits" && fail "E925 wrongly fired for in-range zone" || pass "no E925 for levels 1-4"

    _scn "a Zone or Conduit status outside the documented set raises E926"
    out=$("$SYSCRIBE" -m "$B/zstatus" validate 2>/dev/null) || true
    hits=$(grep -F "| E926 |" <<<"$out" || true)
    grep -qF "ZN-VAL-003.md" <<<"$hits" && grep -qF "'active'" <<<"$hits" \
        && pass "E926 for Zone status active" || fail "E926 missing for Zone status active: $hits"
    grep -qF "CD-VAL-002.md" <<<"$hits" && grep -qF "'done'" <<<"$hits" \
        && pass "E926 for Conduit status done" || fail "E926 missing for Conduit status done: $hits"
    grep -qF "ZN-VAL-004.md" <<<"$hits" && fail "E926 wrongly fired for status deprecated" || pass "no E926 for deprecated"

    _scn "a clean Zone/Conduit model raises none of E924-E926"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-TYPE-020/clean" validate 2>/dev/null) || true
    grep -qE "\| E92[456] \|" <<<"$out" && fail "E924-E926 raised on the clean model" || pass "clean model raises none"
}
