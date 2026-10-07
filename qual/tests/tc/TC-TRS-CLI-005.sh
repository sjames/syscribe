tc_TRS_CLI_005() {
    # The full set of dispatchable commands (must each have a man page).
    local CMDS="validate report audit show ls tree find extref list types untyped \
        connectivity export trace why who-verifies links refs matrix verification-depth \
        metrics cyber-risk co-analysis safety-case feature-check features feature why-active \
        configure diff template next-id check-ref path-for move scaffold-gherkin \
        ingest-results render diagram spec help"

    SCENARIO_NAME="help <cmd> and <cmd> --help print a man page for every command"
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    local missing=0
    for c in $CMDS; do
        # help <cmd>
        out=$("$SYSCRIBE" help "$c" 2>/dev/null); ec=$?
        printf '%s' "$out" | grep -qF "SYNOPSIS" && [ "$ec" -eq 0 ] || { echo "      help $c: no SYNOPSIS / exit $ec"; missing=1; }
        # <cmd> --help (no model dir)
        out2=$("$SYSCRIBE" "$c" --help 2>/dev/null); ec2=$?
        printf '%s' "$out2" | grep -qF "SYNOPSIS" && [ "$ec2" -eq 0 ] || { echo "      $c --help: no SYNOPSIS / exit $ec2"; missing=1; }
    done
    [ "$missing" -eq 0 ] && pass "all commands have a man page via help <cmd> and --help" \
        || fail "some commands lack a man page"

    SCENARIO_NAME="-h alias works"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" matrix -h 2>/dev/null | grep -qF "SYNOPSIS" && pass "matrix -h prints the page" || fail "-h alias failed"

    SCENARIO_NAME="help (no arg) prints an index"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    idx=$("$SYSCRIBE" help 2>/dev/null)
    printf '%s' "$idx" | grep -qF "validate" && printf '%s' "$idx" | grep -qF "audit" \
        && pass "index lists commands" || fail "index missing commands"

    SCENARIO_NAME="help <unknown> exits non-zero"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" help bogus-command >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "unknown command exits non-zero" || fail "unknown command did not fail"
}
