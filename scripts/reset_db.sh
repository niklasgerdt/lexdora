#!/usr/bin/env bash
set -euo pipefail

# Reset the local database to a clean state
# - Stops all services
# - Removes the PostgreSQL data volume
# - Brings the database back up (it will re-run /docker-entrypoint-initdb.d/)
#
# Usage:
#   bash scripts/reset_db.sh

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

echo "[reset-db] Stopping docker-compose services and removing volumes..."
docker compose down -v

echo "[reset-db] Bringing up a fresh PostgreSQL container..."
docker compose up -d postgres

echo "[reset-db] Waiting for database to become healthy..."
ATTEMPTS=60
SLEEP=1
READY=0
for i in $(seq 1 $ATTEMPTS); do
  # Prefer health status via inspect
  STATUS="$(docker inspect -f '{{.State.Health.Status}}' dora-postgres 2>/dev/null || echo "unknown")"
  if [[ "$STATUS" == "healthy" ]]; then
    READY=1; break
  fi
  # Fallback to pg_isready inside the container
  if docker exec dora-postgres pg_isready -U postgres -d dora &>/dev/null; then
    READY=1; break
  fi
  sleep "$SLEEP"
done

if [[ "$READY" -ne 1 ]]; then
  echo "[reset-db] ERROR: Database did not become healthy in time." >&2
  exit 1
fi

echo "[reset-db] Database is now fresh and ready at postgres://postgres:postgres@localhost:5432/dora"
