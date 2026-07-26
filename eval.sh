#!/usr/bin/env bash
# Builds the bot, plays a match against an opponent on the server, and downloads the resulting
# match log to match.jsonl for analysis.
#
# Usage: ./eval.sh [opponent] [map]   (defaults: ripleybot2 hex)
set -euo pipefail

OPPONENT="${1:-ripleybot2}"
MAP="${2:-hex}"
BOT_BINARY="./target/debug/RipleyPlanetWarsBot"
API="https://planetwars.zeus.gent"

cargo build

echo "Playing $BOT_BINARY vs $OPPONENT on $MAP ..."
output=$(planetwars_client "$BOT_BINARY" "$OPPONENT" "$MAP" 2>&1)
echo "$output"

match_url=$(printf '%s\n' "$output" | grep -oE 'https://planetwars.zeus.gent/matches/[0-9]+' | tail -1)
match_id=$(printf '%s\n' "$match_url" | grep -oE '[0-9]+' | tail -1)
if [ -z "$match_id" ]; then
  echo "ERROR: could not find a match id in the client output" >&2
  exit 1
fi

echo "Match URL: $match_url"
echo "Downloading match $match_id ..."
curl -s "$API/api/matches/$match_id/log" > match.jsonl
echo "saved match $match_id to match.jsonl ($(wc -l < match.jsonl | tr -d ' ') lines)"
