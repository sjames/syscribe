tc_TRS_RISK_001() {
    local F="$1"
    local BASE="$F/TC-TRS-RISK-001/model"
    local W A B

    SCENARIO_NAME="a recorded campaign is listed and deterministic"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" inject TC-SYS-001 --record --report-only >/dev/null 2>&1 || true
    A=$(grep -v recordedAtUnix "$W/.syscribe/campaigns.json" 2>/dev/null)
    "$SYSCRIBE" -m "$W" inject TC-SYS-001 --record --report-only >/dev/null 2>&1 || true
    B=$(grep -v recordedAtUnix "$W/.syscribe/campaigns.json" 2>/dev/null)
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" results campaigns 2>&1 || true)
    assert_output_contains "TC-SYS-001"
    assert_output_contains "1 survived, 2 defeated"
    if [ -n "$A" ] && [ "$A" = "$B" ]; then SCENARIO_OUTPUT="identical"; else SCENARIO_OUTPUT="differ"; fi
    assert_output_contains "identical"

    SCENARIO_NAME="defeats are tied to decisions"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local H
    H=$(grep -o '"compositeHash": "[0-9a-f]*"' "$W/.syscribe/campaigns.json" | head -1 | sed 's/.*: "\(.\{12\}\).*/\1/')
    sed -i "s/^expires: 2099-12-31$/expires: 2099-12-31\ncampaignHash: $H/" "$W/RISK-SYS-001.md"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" validate 2>&1 || true)
    assert_count "W940" 1
    assert_output_contains "FM-TX-004"
    assert_no_code "W942"
    sed -i 's/value: "7"/value: "0"/' "$W/TxM.md"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" validate 2>&1 || true)
    assert_count "W942" 1
    assert_count "W944" 1
    sed -i 's/value: "0"/value: "7"/' "$W/TxM.md"

    SCENARIO_NAME="the safety case shows the campaign"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" safety-case 2>&1 || true)
    assert_output_contains "fault-injection campaign TC-SYS-001: 3 sets, 1 survived, 2 defeated (1 explained)"
    assert_output_contains "RISK-SYS-001"
    rm -rf "$W"
}
