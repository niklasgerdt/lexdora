SHELL := /bin/bash

.PHONY: dev db-up db-down reset-db

# Start local development: brings up Postgres in Docker and runs the app in watch mode
dev:
	bash scripts/dev.sh

# Only start the database (background) using docker compose
db-up:
	docker compose up -d postgres

# Stop all compose services
db-down:
	docker compose down

# Reset the database from scratch (destroys data!)
reset-db:
	bash scripts/reset_db.sh
