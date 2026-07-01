# jobRabbit — dev shortcuts (everything via Docker; the host needs no Rust/Node).

.DEFAULT_GOAL := up
.PHONY: up app web-build web-install web-dev build test run tui snapshot release fmt clean shell

up: run       ## Default: rebuild the frontend + run the web UI (the everyday command)

app:          ## Full release flow: test, build the frontend + binary and open the web UI
	@echo "🐇 jobRabbit — preparing everything..."
	docker compose run --rm dev cargo test
	./scripts/build-release.sh
	@echo "🐇 opening the web UI (run it on your desktop, with claude + Chrome)..."
	./dist/jobrabbit

web-install:  ## Install the frontend dependencies (web-ui/)
	docker compose run --rm web npm install --no-audit --no-fund

web-build:    ## Build the frontend (web-ui/ → web-ui/dist)
	docker compose run --rm web npm run build

web-dev:      ## Vite dev server (HMR) on :5173, proxies /api → backend :8787
	docker compose run --rm --service-ports web npm run dev

build:        ## Compile (debug) in the container
	docker compose run --rm dev cargo build

test:         ## Run the test suite
	docker compose run --rm dev cargo test

snapshot:     ## Render the TUI screens as text (preview without a TTY)
	docker compose run --rm dev cargo run -- --tui --snapshot

run: web-build ## Run the web UI (rebuilds the embedded frontend first; needs claude + Chrome on the host)
	docker compose run --rm --service-ports dev cargo run -- --web

tui:          ## Run the classic TUI (needs a TTY)
	docker compose run --rm dev cargo run -- --tui

release:      ## Build the release binary for the host (./dist/jobrabbit)
	./scripts/build-release.sh

fmt:          ## Format the code
	docker compose run --rm dev cargo fmt

clean:        ## Clean build artifacts
	docker compose run --rm dev cargo clean

shell:        ## Open a shell in the dev container
	docker compose run --rm dev bash
