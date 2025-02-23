#!/usr/bin/env bash

# Execute multiple curl requests in parallel
# Each curl runs in its own background process.

echo "Starting parallel curl requests..."

curl -s http://127.0.0.1:7878/sleep -o /dev/null &
curl -s http://127.0.0.1:7878/sleep -o /dev/null &
curl -s http://127.0.0.1:7878/notfound -o /dev/null &
curl -s http://127.0.0.1:7878/temp -o /dev/null &
curl -s http://127.0.0.1:7878/test -o /dev/null &
curl -s http://127.0.0.1:7878/ -o /dev/null &
# Wait for all background tasks to complete
wait

echo "All curl requests have finished."
