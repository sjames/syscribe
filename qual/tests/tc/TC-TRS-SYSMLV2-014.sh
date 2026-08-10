tc_TRS_SYSMLV2_014() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-014/model"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. an interface def's doc-comment directives lift shortName and
    #    implementedBy, and strip the directive lines from doc text while
    #    keeping surrounding prose.
    _scn "an interface def's doc-comment directives lift shortName and implementedBy"
    local eout; eout=$("$SYSCRIBE" -m "$M" export --ndjson 2>&1)
    local iface_fm; iface_fm=$(printf '%s' "$eout" | jq -c --arg q "SysML2::Demo::IPowerInterface" 'select(.qname==$q) | .frontmatter')
    [ "$(printf '%s' "$iface_fm" | jq -r '.shortName')" = "power-if" ] \
        && pass "IPowerInterface.shortName lifted to power-if" \
        || fail "IPowerInterface.shortName was: $iface_fm"
    [ "$(printf '%s' "$iface_fm" | jq -r '.implementedBy[0]')" = "aidl/interfaces/car/power/IPowerInterface.aidl" ] \
        && pass "IPowerInterface.implementedBy lifted" \
        || fail "IPowerInterface.implementedBy was: $iface_fm"

    out=$("$SYSCRIBE" -m "$M" show SysML2::Demo::IPowerInterface 2>&1)
    printf '%s' "$out" | grep -q 'Real interface documentation\.' \
        && pass "surrounding prose survives in doc text" \
        || fail "prose missing from doc text: $out"
    printf '%s' "$out" | grep -q '@SyscribeShortName' \
        && fail "directive line leaked into doc text: $out" \
        || pass "@SyscribeShortName directive line stripped from doc text"
    printf '%s' "$out" | grep -q '@SyscribeImplementedBy' \
        && fail "directive line leaked into doc text: $out" \
        || pass "@SyscribeImplementedBy directive line stripped from doc text"

    # 2. implementedBy lifted via a directive drives W023 (path doesn't
    #    exist on disk in this fixture).
    _scn "implementedBy lifted via a directive drives W023"
    local vout; vout=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)
    printf '%s' "$vout" | grep 'Model.sysml' | grep -q 'W023' \
        && pass "W023 raised for the lifted implementedBy path" \
        || fail "W023 not raised: $vout"

    # 3. a port def's doc-comment directives lift domain and asilLevel
    _scn "a port def's doc-comment directives lift domain and asilLevel"
    local port_fm; port_fm=$(printf '%s' "$eout" | jq -c --arg q "SysML2::Demo::PowerPort" 'select(.qname==$q) | .frontmatter')
    [ "$(printf '%s' "$port_fm" | jq -r '.domain')" = "hardware" ] \
        && pass "PowerPort.domain lifted to hardware" \
        || fail "PowerPort.domain was: $port_fm"
    [ "$(printf '%s' "$port_fm" | jq -r '.asilLevel')" = "D" ] \
        && pass "PowerPort.asilLevel lifted to D" \
        || fail "PowerPort.asilLevel was: $port_fm"

    # 4. a connection def's doc-comment directive lifts shortName, and the
    #    doc-only-directive block leaves no doc text at all.
    _scn "a connection def's doc-comment directive lifts shortName"
    local link_fm; link_fm=$(printf '%s' "$eout" | jq -c --arg q "SysML2::Demo::PowerLink" 'select(.qname==$q) | .frontmatter')
    [ "$(printf '%s' "$link_fm" | jq -r '.shortName')" = "power-link" ] \
        && pass "PowerLink.shortName lifted to power-link" \
        || fail "PowerLink.shortName was: $link_fm"
    local link_out; link_out=$("$SYSCRIBE" -m "$M" show SysML2::Demo::PowerLink 2>&1)
    printf '%s' "$link_out" | grep -q '## Documentation' \
        && fail "PowerLink unexpectedly has a Documentation section: $link_out" \
        || pass "PowerLink has no Documentation section (doc block was directive-only)"

    # 5. an interface def with no doc comment at all is unaffected
    _scn "an interface def with no doc comment is unaffected"
    local plain_fm; plain_fm=$(printf '%s' "$eout" | jq -c --arg q "SysML2::Demo::IPlain" 'select(.qname==$q) | .frontmatter')
    [ "$(printf '%s' "$plain_fm" | jq -r 'has("shortName")')" = "false" ] \
        && [ "$(printf '%s' "$plain_fm" | jq -r 'has("implementedBy")')" = "false" ] \
        && [ "$(printf '%s' "$plain_fm" | jq -r 'has("domain")')" = "false" ] \
        && pass "IPlain has no Syscribe-lifted fields (no regression)" \
        || fail "IPlain unexpectedly carries a lifted field: $plain_fm"
}
