#!/usr/bin/env bash
# Build the release binary inside Docker and copy it to ./dist/jobrabbit, which
# (thanks to the `.:/app` bind-mount) appears on the host. Run the result on the
# host DESKTOP, where `claude`, Chrome, D-Bus and X11/Wayland live.
set -euo pipefail
cd "$(dirname "$0")/.."

# 1) Build the frontend (web-ui/ → web-ui/dist). cargo embeds that dist into the
#    binary via rust-embed, so it MUST happen before cargo build.
echo "▶ building the frontend (web-ui/)..."
docker compose run --rm web sh -c "npm install --no-audit --no-fund && npm run build"

# 2) Build the release binary (with the dist embedded) and copy it to ./dist.
#    cp to a temp file then atomic mv: avoids "Text file busy" (ETXTBSY) if an
#    instance of the binary is currently running.
echo "▶ building the binary (Rust, with the frontend embedded)..."
docker compose run --rm dev sh -c "mkdir -p /app/dist && cargo build --release && cp target/release/jobrabbit /app/dist/.jobrabbit.tmp && mv -f /app/dist/.jobrabbit.tmp /app/dist/jobrabbit"

echo
echo "✔ Binary (with the web UI embedded) at ./dist/jobrabbit"
echo "  Run it on the host (not the container):  ./dist/jobrabbit        # opens the web UI in your browser"
echo "  Classic TUI (fallback):                  ./dist/jobrabbit --tui"
echo "  Runtime deps (host):                     Google Chrome + an authenticated claude CLI (and libxcb1/libxss1/libdbus-1-3 for the TUI)"
