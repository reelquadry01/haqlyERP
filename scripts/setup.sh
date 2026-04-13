#!/usr/bin/env bash
set -euo pipefail

HAQLY_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "=== HAQLY ERP Development Setup ==="
echo "Project root: $HAQLY_ROOT"
echo ""

# ---- Rust ----
if command -v rustup &>/dev/null; then
    echo "[OK] Rust toolchain found: $(rustc --version)"
else
    echo "[INSTALL] Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "[OK] Rust installed: $(rustc --version)"
fi

# ---- Node.js ----
if command -v node &>/dev/null; then
    echo "[OK] Node.js found: $(node --version)"
else
    echo "[INSTALL] Installing Node.js 20 LTS..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
    echo "[OK] Node.js installed: $(node --version)"
fi

# ---- Python ----
if command -v python3 &>/dev/null && python3 -c "import sys; exit(0 if sys.version_info >= (3, 12) else 1)"; then
    echo "[OK] Python found: $(python3 --version)"
else
    echo "[INSTALL] Installing Python 3.12..."
    sudo add-apt-repository -y ppa:deadsnakes/ppa
    sudo apt-get update
    sudo apt-get install -y python3.12 python3.12-venv python3.12-dev
    echo "[OK] Python installed: $(python3.12 --version)"
fi

echo ""
echo "=== Installing project dependencies ==="

# ---- Frontend deps ----
echo "[FRONTEND] Installing npm packages..."
cd "$HAQLY_ROOT/src-web"
npm install
echo "[OK] Frontend dependencies installed"

# ---- Backend build check ----
echo "[BACKEND] Checking Rust build..."
cd "$HAQLY_ROOT/src-server"
cargo check
echo "[OK] Backend compiles"

# ---- Tauri build check ----
echo "[TAURI] Checking Tauri build..."
cd "$HAQLY_ROOT/src-tauri"
cargo check
echo "[OK] Tauri shell compiles"

# ---- AI sidecar deps ----
echo "[AI] Installing Python dependencies..."
cd "$HAQLY_ROOT/src-ai"
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt
pip install -e .
deactivate
echo "[OK] AI engine dependencies installed"

# ---- .env ----
if [ ! -f "$HAQLY_ROOT/.env" ]; then
    echo "[CONFIG] Creating .env from .env.example..."
    cp "$HAQLY_ROOT/.env.example" "$HAQLY_ROOT/.env"
    echo "[OK] .env created. REVIEW AND UPDATE VALUES BEFORE RUNNING."
else
    echo "[OK] .env already exists"
fi

# ---- Docker / PostgreSQL ----
if command -v docker &>/dev/null; then
    echo "[DOCKER] Starting PostgreSQL..."
    cd "$HAQLY_ROOT"
    docker compose up -d postgres
    echo "[OK] Waiting for PostgreSQL to be ready..."
    until docker compose exec -T postgres pg_isready -U haqly &>/dev/null; do
        sleep 1
    done
    echo "[OK] PostgreSQL is ready"
else
    echo "[WARN] Docker not found. Start PostgreSQL manually and set DATABASE_URL in .env"
fi

# ---- Migrations ----
if command -v sqlx &>/dev/null; then
    echo "[DB] Running migrations..."
    cd "$HAQLY_ROOT/src-server"
    source "$HAQLY_ROOT/.env" 2>/dev/null || true
    sqlx migrate run --source migrations
    echo "[OK] Migrations applied"
else
    echo "[DB] Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
    cd "$HAQLY_ROOT/src-server"
    source "$HAQLY_ROOT/.env" 2>/dev/null || true
    sqlx migrate run --source migrations
    echo "[OK] Migrations applied"
fi

# ---- Frontend build ----
echo "[FRONTEND] Building Next.js static export..."
cd "$HAQLY_ROOT/src-web"
npm run build
echo "[OK] Frontend built"

echo ""
echo "=== Setup Complete ==="
echo ""
echo "To start developing:"
echo "  1. Backend:  cd src-server && cargo run"
echo "  2. Frontend: cd src-web && npm run dev"
echo "  3. AI:       cd src-ai && source .venv/bin/activate && uvicorn main:app --port 8200"
echo "  4. Desktop:  cargo tauri dev"
echo ""
echo "Or use Docker Compose:"
echo "  docker compose up"
