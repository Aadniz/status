#!/bin/bash

#
# Script to communicate with named pipes
#

NO_OUTPUT_TIMEOUT=60000  # Milliseconds
OUTPUT_TIMEOUT=400  # Milliseconds


IN_PIPE=/tmp/status_in_pipe
OUT_PIPE=/tmp/status_out_pipe

if [ ! -p "$IN_PIPE" ]; then
    echo "$IN_PIPE doesn't exist, is the application running?"
    exit 2
fi
if [ ! -p "$OUT_PIPE" ]; then
    echo "$OUT_PIPE doesn't exist, is the application running?"
    exit 2
fi


timeout $(( OUTPUT_TIMEOUT / 1000 )) cat "$OUT_PIPE" & echo "$*" > "$IN_PIPE"

