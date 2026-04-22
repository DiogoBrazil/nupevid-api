#!/usr/bin/env bash
set -euo pipefail
set -a
source .env
set +a
exec env DATABASE_URL="$DATABASE_TEST_URL" cargo nextest run "$@"
