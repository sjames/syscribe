tc_TRS_SYSMLV2_008() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-008/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. all four annotations lift onto a part def
    _scn "all four annotations lift onto a part def"
    local out; out=$("$SYSCRIBE" -m "$M" export 2>/dev/null)
    local qn="SysML2::Demo::AllFieldsPart"
    local domain; domain=$(printf '%s' "$out" | jq -r --arg q "$qn" '.elements[] | select(.qname==$q) | .frontmatter.domain')
    [ "$domain" = "software" ] && pass "domain lifted to software" || fail "domain='$domain' (expected software)"
    local asil; asil=$(printf '%s' "$out" | jq -r --arg q "$qn" '.elements[] | select(.qname==$q) | .frontmatter.asilLevel')
    [ "$asil" = "B" ] && pass "asilLevel lifted to B" || fail "asilLevel='$asil' (expected B)"
    local sn; sn=$(printf '%s' "$out" | jq -r --arg q "$qn" '.elements[] | select(.qname==$q) | .frontmatter.shortName')
    [ "$sn" = "all-fields-part" ] && pass "shortName lifted to all-fields-part" || fail "shortName='$sn' (expected all-fields-part)"
    local ib; ib=$(printf '%s' "$out" | jq -r --arg q "$qn" '.elements[] | select(.qname==$q) | .frontmatter.implementedBy[0]')
    [ "$ib" = "services/all-fields-part/" ] && pass "implementedBy lifted to services/all-fields-part/" \
        || fail "implementedBy='$ib' (expected services/all-fields-part/)"

    # 2. @SyscribeIntegrity with both asil and sil raises the existing W006
    _scn "@SyscribeIntegrity with both asil and sil raises the existing W006 mutual-exclusion warning"
    local vout; vout=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)
    local w006_count; w006_count=$(printf '%s' "$vout" | grep -c 'W006' || true)
    [ "$w006_count" -eq 1 ] && pass "exactly one W006 raised" || fail "W006 count=$w006_count (expected 1)"

    # 3. a part def with no annotation carries no lifted fields
    _scn "a part def with no annotation carries no lifted fields"
    local plain; plain=$(printf '%s' "$out" | jq -c --arg q "SysML2::Demo::PlainPart" \
        '.elements[] | select(.qname==$q) | .frontmatter | {domain, asilLevel, silLevel, plLevel, shortName, implementedBy}')
    local nonnull; nonnull=$(printf '%s' "$plain" | jq '[.[] | select(. != null)] | length')
    [ "$nonnull" -eq 0 ] && pass "PlainPart carries no lifted fields" || fail "PlainPart unexpectedly carries lifted fields: $plain"
}
