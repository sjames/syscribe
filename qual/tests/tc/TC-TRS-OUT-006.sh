tc_TRS_OUT_006() {
    local F="$1"
    local M="$F/TC-TRS-VAL-003/W005"   # model emitting one W005 warning, no errors
    local E="$F/TC-TRS-VAL-001/E005"   # model containing an E005 error

    # Helper: run `validate` with extra args, capture exit code.
    _gate_run() {
        SCENARIO_NAME="$1"; shift
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$1" validate "${@:2}" 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    }
    _assert_exit() {
        [ "$SCENARIO_EXIT" -eq "$1" ] && pass "exit code $1" || fail "exit code $SCENARIO_EXIT (expected $1)"
    }

    _gate_run "warnings without gate exit 0" "$M"
    _assert_exit 0

    _gate_run "--deny W005 trips gate (exit 2)" "$M" --deny W005
    _assert_exit 2

    _gate_run "--deny W999 (absent code) stays clean" "$M" --deny W999
    _assert_exit 0

    _gate_run "--max-warnings 0 trips gate (exit 2)" "$M" --max-warnings 0
    _assert_exit 2

    _gate_run "--warnings-as-errors trips gate (exit 2)" "$M" --warnings-as-errors
    _assert_exit 2

    _gate_run "errors dominate gating flags (exit 1)" "$E" --warnings-as-errors
    _assert_exit 1
}
