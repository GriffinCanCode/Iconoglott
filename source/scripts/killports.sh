#!/bin/bash
# Kill processes on backend ports

PORTS=(8765)

for port in "${PORTS[@]}"; do
    pid=$(lsof -ti:$port 2>/dev/null)
    if [ -n "$pid" ]; then
        echo "Killing process on port $port (PID: $pid)"
        kill -9 $pid 2>/dev/null
    fi
done

