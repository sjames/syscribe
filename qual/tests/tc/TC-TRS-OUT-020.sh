tc_TRS_OUT_020() {
    local F="$1"
    local M="$F/TC-TRS-OUT-020/proj"

    # Well-formed XML (xmllint if present).
    _flush_scenario
    SCENARIO_NAME="output is well-formed ReqIF XML"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    local xml; xml=$("$SYSCRIBE" -m "$M" export-reqif --include-tests 2>/dev/null)
    if command -v xmllint >/dev/null 2>&1; then
        printf '%s' "$xml" | xmllint --noout - 2>/dev/null && pass "well-formed (xmllint)" || fail "malformed XML"
    else
        grep -q "</REQ-IF>" <<<"$xml" && pass "has REQ-IF root (xmllint absent)" || fail "no REQ-IF root"
    fi

    run_re() {
        _flush_scenario
        SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        shift
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" export-reqif "$@" 2>/dev/null); SCENARIO_EXIT=$?
    }

    run_re "a SPEC-OBJECT per requirement"
    assert_output_contains "SO-REQ-RIF-000"
    assert_output_contains "SO-REQ-RIF-LEAF-001"

    run_re "package hierarchy as nested SPEC-HIERARCHY"
    assert_output_contains "<SPEC-HIERARCHY"

    run_re "derivedFrom becomes DERIVED_FROM"
    assert_output_contains "REL_DERIVED"

    run_re "--include-tests adds VERIFIED_BY" --include-tests
    assert_output_contains "REL_VERIFIED"

    # --zip writes a readable .reqifz
    _flush_scenario
    SCENARIO_NAME="--zip writes a readable .reqifz"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out="${TMPDIR:-/tmp}/reqif-$$"
    "$SYSCRIBE" -m "$M" export-reqif --zip --output "$out" >/dev/null 2>&1
    if command -v unzip >/dev/null 2>&1; then
        unzip -l "$out.reqifz" 2>/dev/null | grep -q "content.reqif" && pass ".reqifz contains content.reqif" || fail ".reqifz unreadable"
    else
        [ -s "$out.reqifz" ] && pass ".reqifz written (unzip absent)" || fail ".reqifz not written"
    fi
    rm -f "$out.reqifz"
}
