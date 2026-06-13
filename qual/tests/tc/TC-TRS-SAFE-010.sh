tc_TRS_SAFE_010() {
    local F="$1"; local M="$F/TC-TRS-SAFE-010/model"

    SCENARIO_NAME="safety-case appends [unknown] footnote when no results sidecar loaded"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" safety-case 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "verdicts unknown" \
        && pass "footnote present when no sidecar" || fail "footnote missing when no sidecar"

    SCENARIO_NAME="safety-case --json includes verdictsUnknown: true when no sidecar"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    json=$("$SYSCRIBE" -m "$M" safety-case --json 2>/dev/null || true)
    printf '%s' "$json" | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d.get('verdictsUnknown') is True, 'verdictsUnknown not True'
print('ok')
" 2>/dev/null && pass "verdictsUnknown: true in JSON" || fail "verdictsUnknown missing or false in JSON"

    SCENARIO_NAME="safety-case shows [unknown] verdict on TestCase leaf"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qF "[unknown]" \
        && pass "TestCase leaf shows [unknown] verdict" || fail "TestCase leaf missing [unknown] verdict"

    SCENARIO_NAME="safety-case text includes the ingest-results tip command"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qF "ingest-results" \
        && pass "footnote mentions ingest-results" || fail "footnote missing ingest-results command"
}
