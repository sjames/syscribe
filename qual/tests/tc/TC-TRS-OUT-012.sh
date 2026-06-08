tc_TRS_OUT_012() {
    local F="$1"
    local M="$F/TC-TRS-OUT-012/model"   # model emitting W300 on a SIL-4 and a non-SIL leaf req

    # Helper: run `validate --profile ...`, capture exit code (no subcommand-less run).
    _prof_run() {
        SCENARIO_NAME="$1"; shift
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" validate "$@" 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    }
    _assert_exit() {
        [ "$SCENARIO_EXIT" -eq "$1" ] && pass "exit code $1" || fail "exit code $SCENARIO_EXIT (expected $1)"
    }

    _prof_run "warnings without a profile exit 0"
    _assert_exit 0

    _prof_run "--profile all300 promotes both W300 (exit 2)" --profile all300
    _assert_exit 2

    _prof_run "--profile safety promotes the SIL-4 W300 (exit 2)" --profile safety
    _assert_exit 2

    _prof_run "--profile none (scope matches nothing) stays clean (exit 0)" --profile none
    _assert_exit 0

    _prof_run "--profile nonexistent is an error (exit 1)" --profile nonexistent
    _assert_exit 1
}
