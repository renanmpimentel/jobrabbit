# jobRabbit — dev shortcuts (everything via Docker; the host needs no Rust/Node).

.DEFAULT_GOAL := up
.PHONY: up app web-build web-install web-dev build test run tui snapshot release fmt clean shell prepare

# Containers run as root, so anything they write into the bind-mounted repo
# (dist/, web-ui/dist/, formatted sources) would land root-owned on the host.
# `prepare` pre-creates the volume mountpoints as the host user, and FIX_OWN
# hands written files back after each container run.
HOST_UID := $(shell id -u)
HOST_GID := $(shell id -g)
FIX_OWN   = chown -R $(HOST_UID):$(HOST_GID)

prepare:
	@mkdir -p dist target web-ui/node_modules

up: prepare   ## Default: rebuild the frontend + binary and launch the web UI on the host
	./scripts/build-release.sh
	./dist/jobrabbit

app: prepare  ## Full release flow: test, build the frontend + binary and open the web UI
	@echo "🐇 jobRabbit — preparing everything..."
	docker compose run --rm dev cargo test
	./scripts/build-release.sh
	@echo "🐇 opening the web UI (run it on your desktop, with claude + Chrome)..."
	./dist/jobrabbit

web-install: prepare  ## Install the frontend dependencies (web-ui/)
	docker compose run --rm web sh -c "npm install --no-audit --no-fund && $(FIX_OWN) /app/web-ui/package-lock.json"

web-build: prepare    ## Build the frontend (web-ui/ → web-ui/dist)
	docker compose run --rm web sh -c "npm run build && $(FIX_OWN) /app/web-ui/dist"

web-dev: prepare      ## Vite dev server (HMR) on :5173, proxies /api → backend :8787
	docker compose run --rm --service-ports web npm run dev

build: prepare        ## Compile (debug) in the container
	docker compose run --rm dev cargo build

test: prepare         ## Run the test suite
	docker compose run --rm dev cargo test

snapshot: prepare     ## Render the TUI screens as text (preview without a TTY)
	docker compose run --rm dev cargo run -- --tui --snapshot

run: web-build ## Run the web UI (rebuilds the embedded frontend first; needs claude + Chrome on the host)
	docker compose run --rm --service-ports dev cargo run -- --web

tui: prepare          ## Run the classic TUI (needs a TTY)
	docker compose run --rm dev cargo run -- --tui

release: prepare      ## Build the release binary for the host (./dist/jobrabbit)
	./scripts/build-release.sh

fmt: prepare          ## Format the code
	docker compose run --rm dev sh -c "cargo fmt && $(FIX_OWN) /app/src"

clean: prepare        ## Clean build artifacts
	docker compose run --rm dev cargo clean

shell: prepare        ## Open a shell in the dev container
	docker compose run --rm dev bash
