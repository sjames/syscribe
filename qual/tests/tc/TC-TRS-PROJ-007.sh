tc_TRS_PROJ_007() {
    local F="$1"; local G="$F/TC-TRS-PROJ-007/gate"; local FAM="$F/TC-TRS-PROJ-005/family"
    local WDT="CONF-PROJ7-WDT-001" NOWDT="CONF-PROJ7-NOWDT-001"

    _p7_run() { # <model> <args...>: sets SCENARIO_OUTPUT / SCENARIO_EXIT
        local m="$1"; shift
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$m" validate "$@" 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    }
    _p7_exit() { # <expected> <label>
        [ "$SCENARIO_EXIT" -eq "$1" ] && pass "$2 → exit $1" || fail "$2 → exit $SCENARIO_EXIT (expected $1)"
    }

    SCENARIO_NAME="--all-configs honours the gating flags per variant"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _p7_run "$G" --all-configs;                          _p7_exit 0 "--all-configs (no gate)"
    _p7_run "$G" --all-configs --warnings-as-errors;     _p7_exit 2 "--all-configs --warnings-as-errors"
    _p7_run "$G" --all-configs --deny W015;              _p7_exit 2 "--all-configs --deny W015"
    _p7_run "$G" --all-configs --max-warnings 0;         _p7_exit 2 "--all-configs --max-warnings 0"
    _p7_run "$G" --all-configs --max-warnings 6;         _p7_exit 2 "--all-configs --max-warnings 6 (per-variant budget)"
    grep -F "| $NOWDT |" <<<"$SCENARIO_OUTPUT" | grep -qF "pass" \
        && pass "variant within the per-variant budget is marked pass" || fail "NoWdt variant not marked pass under --max-warnings 6"
    grep -F "| $WDT |" <<<"$SCENARIO_OUTPUT" | grep -qF "gate" \
        && pass "variant over the budget is marked gate" || fail "Wdt variant not marked gate under --max-warnings 6"
    _p7_run "$G" --all-configs --deny W999;              _p7_exit 0 "--all-configs --deny W999 (absent code)"
    _p7_run "$G" --all-configs --warnings-as-errors --json
    [ "$SCENARIO_EXIT" -eq 2 ] && jq -e 'length == 2 and all(.[]; .result == "gate")' <<<"$SCENARIO_OUTPUT" >/dev/null 2>&1 \
        && pass "--json reports result=gate per variant, exit 2" || fail "--json result/exit wrong (exit $SCENARIO_EXIT)"

    SCENARIO_NAME="--all-configs honours a scoped --profile per variant"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _p7_run "$G" --all-configs --profile wdt;            _p7_exit 2 "--all-configs --profile wdt"
    grep -F "| $WDT |" <<<"$SCENARIO_OUTPUT" | grep -qF "gate" \
        && pass "Wdt variant trips the scoped profile" || fail "Wdt variant not marked gate"
    grep -F "| $NOWDT |" <<<"$SCENARIO_OUTPUT" | grep -qF "pass" \
        && pass "NoWdt variant (no wdt-tagged element active) passes" || fail "NoWdt variant not marked pass"

    SCENARIO_NAME="errors dominate gates across variants"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _p7_run "$FAM" --all-configs --warnings-as-errors;   _p7_exit 1 "--all-configs --warnings-as-errors on a family with an erroring variant"

    SCENARIO_NAME="--config honours the gating flags and --profile"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _p7_run "$G" --config "$WDT";                        _p7_exit 0 "--config $WDT (no gate)"
    _p7_run "$G" --config "$WDT" --warnings-as-errors;   _p7_exit 2 "--config $WDT --warnings-as-errors"
    _p7_run "$G" --config "$WDT" --profile wdt;          _p7_exit 2 "--config $WDT --profile wdt"
    _p7_run "$G" --config "$NOWDT" --profile wdt;        _p7_exit 0 "--config $NOWDT --profile wdt"

    SCENARIO_NAME="--config honours --file"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _p7_run "$G" --config "$WDT" --file Reqs/Wdt.md
    grep -qF "Reqs/Wdt.md" <<<"$SCENARIO_OUTPUT" && pass "findings for the filtered file are reported" || fail "no findings for Reqs/Wdt.md"
    grep -qF "Reqs/Core.md" <<<"$SCENARIO_OUTPUT" && fail "--file did not filter (Core.md present)" || pass "other files' findings are filtered out"

    SCENARIO_NAME="usage errors exit 1"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local o e rc
    o=$("$SYSCRIBE" -m "$G" validate --config CONF-DOES-NOT-EXIST 2>/dev/null) && rc=0 || rc=$?
    e=$("$SYSCRIBE" -m "$G" validate --config CONF-DOES-NOT-EXIST 2>&1 >/dev/null || true)
    [ "$rc" -eq 1 ] && [ -z "$o" ] && [ -n "$e" ] \
        && pass "unresolvable --config → exit 1, stderr message, empty stdout" || fail "unresolvable --config → exit $rc, stdout ${#o} bytes"
    o=$("$SYSCRIBE" -m "$G" validate --profile nosuch 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && [ -z "$o" ] && pass "undefined --profile → exit 1, empty stdout" || fail "undefined --profile → exit $rc"
}
