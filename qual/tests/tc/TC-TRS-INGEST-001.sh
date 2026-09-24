tc_TRS_INGEST_001() {
    local F="$1"; local FX="$F/TC-TRS-INGEST-001"
    local LOGS="$FX/logs"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # Every ingest runs against a fresh scratch copy of the model — the command
    # writes <model>/.syscribe/results.json, so the fixture is never mutated.
    local tmp M side out err rc before after
    tmp=$(mktemp -d); M="$tmp/model"; side="$M/.syscribe/results.json"
    cp -r "$FX/model" "$M"

    # 1. a well-formed session-log file ingests successfully
    _scn "a well-formed session-log file ingests successfully"
    out=$("$SYSCRIBE" -m "$M" ingest-results --format session-log "$LOGS/good.json" 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0): $out"
    printf '%s' "$out" | grep -qF '(session-log): 1 pass, 1 fail, 0 ignored' \
        && pass "summary reports 1 pass, 1 fail from session-log" || fail "unexpected summary: $out"
    if [ -f "$side" ]; then
        pass "sidecar written at .syscribe/results.json"
        jq -e '.format == "session-log"
               and (.by_scenario | length) == 2
               and .by_scenario["TC-SL-001::An empty allowlist denies everything"] == "pass"
               and .by_scenario["TC-SL-001::A configured allowlist permits listed commands"] == "fail"' \
            "$side" >/dev/null 2>&1 \
            && pass "sidecar carries one verdict per (testCase, scenario) pair: pass / fail" \
            || fail "sidecar by_scenario content wrong: $(cat "$side")"
    else
        fail "sidecar not written"
    fi
    # The good sidecar from scenario 1 is now the pre-existing sidecar that
    # every following failing ingest must leave byte-identical.
    before=$(cat "$side" 2>/dev/null || true)

    # 2. a record with empty steps fails ingestion
    _scn "a record with empty steps fails ingestion"
    err=$("$SYSCRIBE" -m "$M" ingest-results --format session-log "$LOGS/empty_steps.json" 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "exit code $rc (non-zero)" || fail "exit code 0 (expected non-zero)"
    printf '%s' "$err" | grep -qF "record 1 ('TC-SL-001' / 'A configured allowlist permits listed commands') has empty or missing steps" \
        && pass "error names the offending record (index 1, its testCase and scenario) and the steps defect" \
        || fail "error does not name the offending record: $err"
    printf '%s' "$err" | grep -qF 'record 0' \
        && fail "error blames the valid record 0" || pass "valid record 0 is not blamed"
    after=$(cat "$side" 2>/dev/null || true)
    [ -n "$before" ] && [ "$before" = "$after" ] \
        && pass "existing sidecar unchanged" || fail "existing sidecar was modified by a failed ingest"

    # 3. a record with a missing steps field fails ingestion
    _scn "a record with a missing steps field fails ingestion"
    err=$("$SYSCRIBE" -m "$M" ingest-results --format session-log "$LOGS/missing_steps.json" 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "exit code $rc (non-zero)" || fail "exit code 0 (expected non-zero)"
    printf '%s' "$err" | grep -qF "record 1 ('TC-SL-001' / 'A configured allowlist permits listed commands') has empty or missing steps" \
        && pass "error names the offending record and the missing steps" \
        || fail "error does not name the offending record: $err"
    after=$(cat "$side" 2>/dev/null || true)
    [ "$before" = "$after" ] && pass "existing sidecar unchanged" || fail "existing sidecar was modified by a failed ingest"

    # 4. a record with an unrecognized result value fails ingestion
    _scn "a record with an unrecognized result value fails ingestion"
    err=$("$SYSCRIBE" -m "$M" ingest-results --format session-log "$LOGS/bad_result.json" 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "exit code $rc (non-zero)" || fail "exit code 0 (expected non-zero)"
    printf '%s' "$err" | grep -qF "has unrecognized result 'maybe'" \
        && pass "error names the unrecognized value 'maybe'" || fail "error does not name the unrecognized value: $err"
    printf '%s' "$err" | grep -qF "record 0 ('TC-SL-001' / 'An empty allowlist denies everything')" \
        && pass "error names the offending record" || fail "error does not name the offending record: $err"
    after=$(cat "$side" 2>/dev/null || true)
    [ "$before" = "$after" ] && pass "existing sidecar unchanged" || fail "existing sidecar was modified by a failed ingest"

    # 5. an empty input array fails ingestion
    _scn "an empty input array fails ingestion"
    err=$("$SYSCRIBE" -m "$M" ingest-results --format session-log "$LOGS/empty_array.json" 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "exit code $rc (non-zero)" || fail "exit code 0 (expected non-zero)"
    printf '%s' "$err" | grep -qF 'no records found (empty array)' \
        && pass "error reports the empty array" || fail "error does not report the empty array: $err"
    after=$(cat "$side" 2>/dev/null || true)
    [ "$before" = "$after" ] && pass "existing sidecar not replaced by an empty result set" \
        || fail "existing sidecar was replaced: $after"
    # And with no sidecar present, nothing at all is written.
    local M2="$tmp/model_fresh"; cp -r "$FX/model" "$M2"
    "$SYSCRIBE" -m "$M2" ingest-results --format session-log "$LOGS/empty_array.json" >/dev/null 2>&1 || true
    [ -e "$M2/.syscribe/results.json" ] \
        && fail "an empty result-set sidecar was written" || pass "no sidecar written for an empty array"

    # 6. --format session-log is never inferred from the file extension
    _scn "--format session-log is never inferred from the file extension"
    local M3="$tmp/model_noformat"; cp -r "$FX/model" "$M3"
    out=$("$SYSCRIBE" -m "$M3" ingest-results "$LOGS/good.json" 2>&1) || true
    printf '%s' "$out" | grep -qF '(session-log)' \
        && fail "format was inferred as session-log: $out" || pass "format not reported as session-log"
    if [ -f "$M3/.syscribe/results.json" ]; then
        jq -e '(.format != "session-log") and ((.by_scenario // {}) | length) == 0' \
            "$M3/.syscribe/results.json" >/dev/null 2>&1 \
            && pass "no session-log scenario verdicts recorded without --format" \
            || fail "session-log verdicts recorded without --format: $(cat "$M3/.syscribe/results.json")"
    else
        pass "no session-log scenario verdicts recorded without --format (no sidecar)"
    fi

    rm -rf "$tmp"
}
