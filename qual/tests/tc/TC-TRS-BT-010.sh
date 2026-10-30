tc_TRS_BT_010() {
    local F="$1"
    # The single-stage behaviour is exactly what TC-TRS-PARAM-004 pins; run it unchanged.
    source "$(dirname "${BASH_SOURCE[0]}")/TC-TRS-PARAM-004.sh"
    tc_TRS_PARAM_004 "$F"
}
