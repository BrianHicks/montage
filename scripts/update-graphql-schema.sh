#!/usr/bin/env bash
set -euo pipefail

cd $(git rev-parse --show-toplevel)

CLIENT_FILE=$(pwd)/montage_client/schema.graphql

SCHEMA="$(cargo run show-graphql-schema)"

printf '# NOTE: don''t change this by hand! Instead, run `%s` to get updates\n%s' "$0" "$SCHEMA" > "$CLIENT_FILE"
