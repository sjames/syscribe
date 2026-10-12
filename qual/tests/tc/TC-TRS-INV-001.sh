tc_TRS_INV_001() {
    local F="$1"
    local BASE="$F/TC-TRS-INV-001/model"
    local W

    SCENARIO_NAME="within is reached and kept"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE" simulate TC-SYS-001 2>&1 || true)
    assert_output_contains "is not reached within 2 s (by t = 2 s)"
    assert_output_contains "PASS   TC-SYS-001  reached at 2.2 s passes within 2.5 s"
    assert_output_contains "was reached at t = 1 s but no longer holds at t = 1.5 s"

    SCENARIO_NAME="always holds at every settled instant"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_output_contains "\`m.a == 0\` does not hold at t = 1.5 s"

    SCENARIO_NAME="degraded clauses"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_output_contains "held in a degraded clause: 'the safe state': Except under FM-SYS-001"
    assert_output_contains "no applicable degraded clause for 'the safe state'"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE" inject TC-SYS-001 --report-only 2>&1 || true)
    assert_output_contains "SURVIVED (DEGRADED) FM-SYS-001"
    assert_output_contains "3 survived (2 only in a degraded safe state), 0 defeated"

    SCENARIO_NAME="malformed clauses are E886"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    sed -i 's/^  Where the safe state is m.a == 0 and m.b == 0$/  Except under FM-SYS-003: m.a == 0\n  Where the safe state is m.a == 0 and m.b == 0/; s/Then within 1 s the safe state holds$/Then within 1 s the closed state holds/' "$W/TC-SYS-001.md"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" validate 2>&1 || true)
    assert_output_contains "belongs to the \`Where\` before it"
    assert_output_contains "there is no \`Where the closed state is <expression>\`"
    rm -rf "$W"
}
