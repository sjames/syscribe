tc_TRS_TYPE_024() {
    local F="$1"; local B="$F/TC-TRS-TYPE-024"
    local out rc c

    SCENARIO_NAME="references through the mount point resolve"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B/mount" validate 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "mounted composition validates (exit 0)" || fail "mounted composition exit $rc"
    for c in E512 E102 E103 E104 E105 E110 E111 E112 E113 E114 E503 E514; do
        if grep -qF "| $c |" <<<"$out"; then
            fail "unexpected $c: $(grep -F "| $c |" <<<"$out" | head -1 | cut -c1-200)"
        else
            pass "no $c"
        fi
    done

    SCENARIO_NAME="a mounted reference naming nothing in the peer is E512"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B/mount-bad" validate 2>&1 || true)
    grep -E "^\| E512 \|" <<<"$out" | grep -qF "Integration::Brakes::REQ-NOPE-001" && pass "E512 for a dangling mounted verifies" || fail "no E512 for a dangling mounted verifies"
    grep -E "^\| E512 \|" <<<"$out" | grep -qF "Integration::Brakes::NoSuchDef" && pass "E512 for a dangling mounted supertype" || fail "no E512 for a dangling mounted supertype"

    SCENARIO_NAME="a stable id exported by two peers is E515"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B/dup" validate 2>&1) && rc=0 || rc=$?
    local line
    line=$(grep -E "^\| E515 \|" <<<"$out" || true)
    if grep -qF "REQ-BRK-001" <<<"$line" && grep -qF "'brakes'" <<<"$line" && grep -qF "'clone'" <<<"$line"; then
        pass "E515 names the id and both peer aliases"
    else
        fail "E515 missing for a peer-vs-peer duplicate id"
    fi
    [ "$rc" -ne 0 ] && pass "validate exits non-zero on the duplicate" || fail "validate exited 0 despite E515"

    SCENARIO_NAME="the local-vs-peer E515 and the repo checks are unchanged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local T="$F/TC-TRS-TYPE-021"
    out=$("$SYSCRIBE" -m "$T/clean" validate 2>&1 || true)
    grep -qE "E51[0-9]|W510" <<<"$out" && fail "TC-TRS-TYPE-021 clean composition now reports E51x/W510" || pass "TC-TRS-TYPE-021 clean composition still clean"
    out=$("$SYSCRIBE" -m "$T/e515" validate 2>&1 || true)
    grep -qF "| E515 |" <<<"$out" && pass "local-vs-peer E515 still raised" || fail "local-vs-peer E515 lost"
}
