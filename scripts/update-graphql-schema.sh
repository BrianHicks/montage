#!/usr/bin/env bash
set -euo pipefail

cd $(git rev-parse --show-toplevel)

CLIENT_FILE=src/client/schema.graphql

if test -f "$CLIENT_FILE"; then rm "$CLIENT_FILE"; fi

printf '# NOTE: don''t change this by hand! Instead, run `%s` to get updates\n' "$0" > "$CLIENT_FILE"
cargo run show-graphql-schema >> "$CLIENT_FILE"
