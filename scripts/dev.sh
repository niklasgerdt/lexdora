#!/usr/bin/env bash
set -euo pipefail

# Local dev convenience script
# - Starts PostgreSQL via docker compose
# - Waits until DB is healthy
# - Exports DATABASE_URL pointing to localhost
# - Runs the Rust app in watch mode (auto-restart on code changes)
#
# Usage:
#   Option A (no execute bit needed):
#     bash scripts/dev.sh
#   Option B (make it executable once, then run):
#     chmod +x scripts/dev.sh && ./scripts/dev.sh
#   Option C (via Makefile shortcut):
#     make dev
#
# Requirements:
#   - Docker + Docker Compose plugin
#   - Rust toolchain (cargo)
#   - cargo-watch (installed automatically if missing)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

echo "[dev] Using project root: $ROOT_DIR"

# Ensure docker compose is available
if ! command -v docker &>/dev/null; then
  echo "[dev] ERROR: docker is not installed or not on PATH" >&2
  exit 1
fi
if ! docker compose version &>/dev/null; then
  echo "[dev] ERROR: docker compose plugin is not available (Docker Desktop 2.20+ or Compose v2 required)" >&2
  exit 1
fi

echo "[dev] Bringing up PostgreSQL container (docker-compose.yml) ..."
docker compose up -d postgres

echo "[dev] Waiting for database to become healthy ..."
ATTEMPTS=120
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
  echo "[dev] ERROR: Database did not become healthy in time. Check 'docker logs dora-postgres'" >&2
  exit 1
fi

export DATABASE_URL="postgres://postgres:postgres@localhost:5432/dora"
export RUST_LOG="info"
echo "[dev] DATABASE_URL=$DATABASE_URL"

# Ensure cargo-watch exists (install if missing)
if ! command -v cargo-watch &>/dev/null; then
  echo "[dev] Installing cargo-watch (one-time) ..."
  cargo install cargo-watch
fi

echo "[dev] Starting the app with cargo-watch (restarts on code changes) ..."
echo "      Press Ctrl-C to stop. Database container will remain running."

# Watch typical sources; run the server on changes
exec cargo watch \
  -q \
  -w src \
  -w web \
  -w Cargo.toml \
  -x "run -- serve --bind 0.0.0.0:8080"
