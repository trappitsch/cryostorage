#!/usr/bin/bash
CONFIG="${HOME}/.config/wf-panel-pi.ini"

if [[ ! -f "$CONFIG" ]]; then
    echo "Config file not found: $CONFIG"
    exit 1
fi

if grep -q '^autohide=true' "$CONFIG"; then
    sed -i \
        -e 's/^autohide=true$/autohide=false/' \
	"$CONFIG"
else
    sed -i \
        -e 's/^autohide=false$/autohide=true/' \
	"$CONFIG"
fi
