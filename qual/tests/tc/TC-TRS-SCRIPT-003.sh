tc_TRS_SCRIPT_003() {
    local F="$1"
    local M="$F/TC-TRS-SCRIPT-003"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    local OUT ERR rc=0
    OUT=$("$SYSCRIBE" -m "$M" scripts run inspect 2>/tmp/tc_script_003.err) || rc=$?
    ERR=$(cat /tmp/tc_script_003.err)

    _scn "iterate and read element getters"
    [ "$rc" -eq 0 ] && pass "inspect exit 0" || fail "inspect exit $rc"
    printf '%s' "$OUT" | grep -qF "REQ REQ-FIX-BRK-001 status=approved domain=system" \
        && pass "id/status/field(reqDomain) read" || fail "getter/field read wrong"
    printf '%s' "$OUT" | grep -qF "type=Requirement" \
        && pass "e.type read" || fail "e.type read wrong"
    printf '%s' "$OUT" | grep -qF "title=Service brake shall stop the vehicle" \
        && pass "e.title read" || fail "e.title read wrong"
    printf '%s' "$OUT" | grep -qF "tags=2" \
        && pass "e.tags read as array" || fail "e.tags read wrong"
    printf '%s' "$OUT" | grep -qF "doc_has_shall=true" \
        && pass "e.doc (markdown body) read" || fail "e.doc read wrong"

    _scn "find resolves by id and by qualified name"
    printf '%s' "$OUT" | grep -qF "find_id_qname=REQ-FIX-BRK-001/REQ-FIX-BRK-001" \
        && pass "find by id and qname resolve same element" || fail "find by id/qname mismatch"
    printf '%s' "$OUT" | grep -qF "unknown_is_unit=true" \
        && pass "unknown find returns unit ()" || fail "unknown find not unit"

    _scn "custom fields and absent fields"
    printf '%s' "$OUT" | grep -qF "supplier=Bosch" \
        && pass "custom_fields[supplier] read" || fail "custom_fields read wrong"
    printf '%s' "$OUT" | grep -qF "mg_origin=stakeholder" \
        && pass "custom_fields[mg_origin] read" || fail "mg_ custom field read wrong"
    printf '%s' "$OUT" | grep -qF "missing_is_unit=true" \
        && pass "e.field(absent) returns unit ()" || fail "absent field not unit"

    _scn "computed reverse index"
    printf '%s' "$OUT" | grep -qF "verified_by=1" \
        && pass "e.verified_by populated" || fail "verified_by wrong"
    printf '%s' "$OUT" | grep -qF "verified_by_0=TC-FIX-BRK-001" \
        && pass "verified_by contains the verifying test case" || fail "verified_by content wrong"

    _scn "stdout and stderr output"
    printf '%s' "$OUT" | grep -qF "inspect complete: " \
        && pass "command returned string printed to stdout" || fail "returned string missing"
    printf '%s' "$ERR" | grep -qF "inspect: starting" \
        && pass "eprint wrote to stderr" || fail "eprint not on stderr"
    printf '%s' "$OUT" | grep -qF "inspect: starting" \
        && fail "eprint text leaked to stdout" || pass "eprint not on stdout"

    rm -f /tmp/tc_script_003.err
}
