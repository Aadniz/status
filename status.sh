#!/bin/bash

#
# Script to communicate with named pipes
#

NO_OUTPUT_TIMEOUT=60  # Seconds
OUTPUT_TIMEOUT=0.1  # Seconds

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

# Send input to the IN_PIPE in the background
echo "$*" > "$IN_PIPE" &
ECHO_PID=$!

# Start a background process that will kill echo after NO_OUTPUT_TIMEOUT seconds
(sleep $NO_OUTPUT_TIMEOUT; kill $ECHO_PID 2>/dev/null) &
SLEEP_PID=$!

# Wait for echo to finish
wait $ECHO_PID

# If echo finished before the timeout, kill the sleep process
if kill -0 $SLEEP_PID 2>/dev/null; then
    kill $SLEEP_PID 2>/dev/null
else
    echo "Writing to pipe timed out after $NO_OUTPUT_TIMEOUT seconds."
    exit 124
fi

# Try to read from the OUT_PIPE with NO_OUTPUT_TIMEOUT, if we get something then keep trying with OUTPUT_TIMEOUT
timeout=$NO_OUTPUT_TIMEOUT
while true; do
    IFS= read -r -t $timeout line <"$OUT_PIPE"
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 0 ]; then
      echo "$line"
      timeout=$OUTPUT_TIMEOUT
    else
      if [ $timeout == $NO_OUTPUT_TIMEOUT ]; then
        echo "No output from application within $NO_OUTPUT_TIMEOUT seconds."
        exit 124
      fi
      break
    fi
done