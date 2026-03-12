#!/bin/bash
BIN_PATH="${HOME}/opt/bin"
mv $BIN_PATH/cryostorage_host $BIN_PATH/cryostorage_host_old
lxterminal -e "curl -o $BIN_PATH/cryostorage_host https://drive.switch.ch/index.php/s/cZMKWzbJnSFui3i/download; chmod +x $BIN_PATH/cryostorage_host"
