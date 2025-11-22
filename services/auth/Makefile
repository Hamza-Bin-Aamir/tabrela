.PHONY: help build run test test-unit test-integration test-system test-coverage clean dev migrate docker-build docker-run

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the project in release mode
	cargo build --release

run: ## Run the service
	cargo run

dev: ## Run the service in development mode with auto-reload
	cargo watch -x run

test: ## Run all tests
	cargo test

test-unit: ## Run unit tests only
	cargo test --lib

test-integration: ## Run integration tests (requires test database)
	@echo "Setting up test database..."
	-dropdb auth_test 2>/dev/null || true
	createdb auth_test
	DATABASE_URL=postgresql://postgres:postgres@localhost/auth_test sqlx migrate run
	cargo test --test integration_test -- --ignored

test-system: ## Run system tests (requires running server)
	@echo "Starting server in background..."
	cargo run & echo $$! > .server.pid
	@sleep 3
	@echo "Running system tests..."
	-cargo test --test system_test -- --ignored
	@echo "Stopping server..."
	-kill `cat .server.pid` 2>/dev/null || true
	-rm .server.pid

test-coverage: ## Generate test coverage report
	cargo tarpaulin --out Html --output-dir coverage --exclude-files tests/*

migrate: ## Run database migrations
	sqlx migrate run

migrate-revert: ## Revert last database migration
	sqlx migrate revert

migrate-info: ## Show migration status
	sqlx migrate info

clean: ## Clean build artifacts
	cargo clean
	rm -rf coverage/
	rm -f .server.pid

fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt -- --check

lint: ## Run clippy linter
	cargo clippy -- -D warnings

audit: ## Check for security vulnerabilities
	cargo audit

check: fmt-check lint test ## Run all checks (format, lint, test)

doc: ## Generate documentation
	cargo doc --no-deps --open

install-tools: ## Install development tools
	cargo install cargo-watch
	cargo install cargo-tarpaulin
	cargo install sqlx-cli --no-default-features --features postgres
	cargo install cargo-audit

docker-build: ## Build Docker image
	docker build -t auth-service:latest .

docker-run: ## Run Docker container
	docker run -p 8081:8081 --env-file .env auth-service:latest

docker-compose-up: ## Start services with docker-compose
	docker-compose up -d

docker-compose-down: ## Stop services with docker-compose
	docker-compose down

setup: ## Initial project setup
	@echo "Setting up development environment..."
	cp .env.example .env
	@echo "Creating database..."
	-createdb auth_db 2>/dev/null || echo "Database already exists"
	@echo "Running migrations..."
	make migrate
	@echo "Installing tools..."
	make install-tools
	@echo ""
	@echo "Setup complete! Edit .env file and run 'make dev' to start developing."

db-reset: ## Reset database (WARNING: deletes all data)
	@echo "⚠️  WARNING: This will delete all data!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		dropdb auth_db; \
		createdb auth_db; \
		make migrate; \
		echo "Database reset complete"; \
	fi

benchmark: ## Run benchmarks
	cargo bench

watch-test: ## Run tests in watch mode
	cargo watch -x test
