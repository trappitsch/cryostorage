#!/usr/bin/bash

# Kill poststation (if running) and then start it
pkill poststation
lxterminal -e "poststation; bash" &

# hide taskbar if it is present
TBAR_CONF="${HOME}/.config/wf-panel-pi.ini"

if grep -q '^autohide=false' "$TBAR_CONF"; then
    sed -i \
	-e 's/^autohide=false$/autohide=true/' \
	"$TBAR_CONF"
fi

sleep 2s

# Run the cryostorage program using `cargo run --release`
CRYO_PATH="${HOME}/opt/bin"
lxterminal --working-directory=$CRYO_PATH -e ./cryostorage_host
