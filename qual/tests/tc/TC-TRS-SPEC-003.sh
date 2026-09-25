tc_TRS_SPEC_003() {
    local F="$1"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    # Any model will do — explain_finding reads only the embedded catalogue.
    local M="$F/TC-TRS-LINKTYPE-007/model" out
    local init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qual","version":"1"}}}'
    local codes=(E600 W610 W041 W042 E317 W045 E108 E231 E520 W047 W090 W404 W563 I010 W007 W010)
    local reqs=("$init") i=10 c
    for c in "${codes[@]}"; do
        reqs+=("{\"jsonrpc\":\"2.0\",\"id\":$i,\"method\":\"tools/call\",\"params\":{\"name\":\"explain_finding\",\"arguments\":{\"code\":\"$c\"}}}")
        i=$((i+1))
    done
    out=$(printf '%s\n' "${reqs[@]}" | timeout 20 "$SYSCRIBE" -m "$M" mcp --read-only 2>/dev/null) || true
    # expl <code> — the explanation text for <code> ("" when the call errored).
    expl() {
        local idx=10 k
        for k in "${codes[@]}"; do [ "$k" = "$1" ] && break; idx=$((idx+1)); done
        jq -sr --argjson id "$idx" '.[] | select(.id==$id) | .result
            | if .isError then "" else ((.content[0].text | fromjson? | .explanation) // "") end' <<<"$out" 2>/dev/null || true
    }

    _scn "three-column table rows explain the condition, not the severity"
    local e
    for c in E600 W610 W041 W042 E317 W045; do
        e=$(expl "$c")
        if [ -z "$e" ] || grep -qixE '(error|warning|info)' <<<"$e"; then
            fail "$c explanation is a severity word or empty: '$e'"
        else
            pass "$c explained: ${e:0:60}"
        fi
    done

    _scn "previously missing codes are explained"
    for c in E108 E231 E520 W047 W090 W404 W563 I010; do
        e=$(expl "$c")
        if [ -n "$e" ] && ! grep -qixE '(error|warning|info)' <<<"$e"; then
            pass "$c explained"
        else
            fail "$c has no explanation"
        fi
    done

    _scn "stale W007 and W010 rows are corrected"
    local w7 w10; w7=$(expl W007); w10=$(expl W010)
    if grep -qiE 'supertype|never used|never referenced' <<<"$w7" && ! grep -qiE '^Unrecogni' <<<"$w7"; then
        pass "W007 = unused definition"
    else
        fail "W007 stale: '$w7'"
    fi
    if grep -qiE 'test result|ingest' <<<"$w10" && ! grep -qi 'isRequired' <<<"$w10"; then
        pass "W010 = test-result ingestion"
    else
        fail "W010 stale: '$w10'"
    fi
}
