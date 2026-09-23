tc_TRS_INGEST_002() {
    local F="$1"; local FX="$F/TC-TRS-INGEST-002"
    local LOGS="$FX/logs"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # row <trace-output> <tc-id> — the "Verified by" table row for that TestCase
    row() { printf '%s\n' "$1" | grep -E "^\| $2( \[[a-z]+\])? \|" || true; }

    local tmp M out r
    tmp=$(mktemp -d)
    # fresh <name> — a scratch copy of the model (ingest writes .syscribe/results.json)
    fresh() { M="$tmp/$1"; cp -r "$FX/model" "$M"; }
    # ingest_sl <log> — session-log ingest into the current scratch model
    ingest_sl() { "$SYSCRIBE" -m "$M" ingest-results --format session-log "$LOGS/$1" >/dev/null 2>&1 || true; }

    # 1. a TestCase with no testFunctions is unannotated before any ingest
    _scn "a TestCase with no testFunctions is unannotated before any ingest"
    fresh s1
    out=$("$SYSCRIBE" -m "$M" trace REQ-SL-001 2>&1 || true)
    r=$(row "$out" TC-SL-001)
    [ -e "$M/.syscribe/results.json" ] && fail "precondition: a sidecar already exists" || pass "no results sidecar present"
    printf '%s' "$r" | grep -qF '| TC-SL-001 | Manually verified via a live session | L5 | 2 |' \
        && pass "TC-SL-001 listed with no verdict annotation" || fail "TC-SL-001 row wrong/annotated: '$r'"

    # 2. every scenario recorded pass annotates the TestCase pass
    _scn "every scenario recorded pass annotates the TestCase pass"
    fresh s2; ingest_sl all_pass.json
    out=$("$SYSCRIBE" -m "$M" trace REQ-SL-001 2>&1 || true)
    r=$(row "$out" TC-SL-001)
    printf '%s' "$r" | grep -qF '| TC-SL-001 [pass] |' \
        && pass "TC-SL-001 annotated [pass]" || fail "TC-SL-001 not annotated [pass]: '$r'"

    # 3. any scenario recorded fail annotates the whole TestCase fail
    _scn "any scenario recorded fail annotates the whole TestCase fail"
    fresh s3; ingest_sl one_fail.json
    out=$("$SYSCRIBE" -m "$M" trace REQ-SL-001 2>&1 || true)
    r=$(row "$out" TC-SL-001)
    printf '%s' "$r" | grep -qF '| TC-SL-001 [fail] |' \
        && pass "TC-SL-001 annotated [fail]" || fail "TC-SL-001 not annotated [fail]: '$r'"

    # 4. partial scenario coverage does not falsely read as pass
    _scn "partial scenario coverage does not falsely read as pass"
    fresh s4; ingest_sl partial.json
    jq -e '(.by_scenario | length) == 1' "$M/.syscribe/results.json" >/dev/null 2>&1 \
        && pass "precondition: exactly one of the two scenarios ingested" || fail "precondition: partial ingest did not land"
    out=$("$SYSCRIBE" -m "$M" trace REQ-SL-001 2>&1 || true)
    r=$(row "$out" TC-SL-001)
    [ -n "$r" ] && pass "TC-SL-001 still listed" || fail "TC-SL-001 missing from trace: $out"
    printf '%s' "$r" | grep -qF '[pass]' && fail "partial coverage annotated [pass]: '$r'" || pass "not annotated [pass]"
    printf '%s' "$r" | grep -qF '[fail]' && fail "partial coverage annotated [fail]: '$r'" || pass "not annotated [fail]"

    # 5. a TestCase with testFunctions ignores any session-log data
    _scn "a TestCase with testFunctions ignores any session-log data"
    # (a) Sidecar holding both a passing function verdict and a failing
    #     session-log verdict for the same TestCase: only testFunctions count.
    fresh s5a; mkdir -p "$M/.syscribe"; cp "$LOGS/mixed_results.json" "$M/.syscribe/results.json"
    out=$("$SYSCRIBE" -m "$M" trace REQ-SL-002 2>&1 || true)
    r=$(row "$out" TC-SL-002)
    printf '%s' "$r" | grep -qF '| TC-SL-002 [pass] |' \
        && pass "with function=pass and scenario=fail both present, TC-SL-002 is [pass] (testFunctions only)" \
        || fail "session-log data leaked into a testFunctions-scored verdict: '$r'"
    # (b) The literal CLI sequence: score via cargo-json, then also ingest
    #     session-log data for that TestCase. The verdict must stay [pass].
    #     (Product 0.39.0: each ingest-results call rewrites the whole sidecar,
    #     so the session-log ingest drops the cargo-json by_leaf verdicts and
    #     TC-SL-002 degrades to [unknown] — REQ-TRS-INGEST-001 requires the
    #     session-log verdicts to be persisted "alongside" existing ones.)
    fresh s5b
    "$SYSCRIBE" -m "$M" ingest-results --format cargo-json "$LOGS/cargo.json" >/dev/null 2>&1 || true
    out=$("$SYSCRIBE" -m "$M" trace REQ-SL-002 2>&1 || true)
    r=$(row "$out" TC-SL-002)
    printf '%s' "$r" | grep -qF '| TC-SL-002 [pass] |' \
        && pass "precondition: cargo-json ingest scores TC-SL-002 [pass]" || fail "precondition: cargo-json ingest did not score [pass]: '$r'"
    ingest_sl tf_session_fail.json
    out=$("$SYSCRIBE" -m "$M" trace REQ-SL-002 2>&1 || true)
    r=$(row "$out" TC-SL-002)
    printf '%s' "$r" | grep -qF '[fail]' \
        && fail "session-log fail changed TC-SL-002 to [fail]: '$r'" || pass "session-log fail did not mark TC-SL-002 [fail]"
    printf '%s' "$r" | grep -qF '| TC-SL-002 [pass] |' \
        && pass "TC-SL-002 verdict unchanged ([pass]) after the session-log ingest" \
        || fail "TC-SL-002 verdict changed after a session-log ingest (was [pass], now '$r')"

    rm -rf "$tmp"
}
