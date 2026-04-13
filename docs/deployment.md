# HAQLY ERP — Deployment Guide

**Author:** Quadri Atharu  
**Version:** 0.1.0  
**Date:** 2026-04-13

---

## 1. Overview

HAQLY ERP supports two deployment shapes:
1. **Tauri Desktop Application** — Primary deployment for end users.
2. **Docker Compose** — For development, testing, and server-based deployments.

---

## 2. Prerequisites

| Software | Version | Purpose |
|---|---|---|
| Rust | 1.80+ | Backend server, Tauri shell |
| Node.js | 20 LTS | Frontend build |
| Python | 3.12+ | AI sidecar |
| PostgreSQL | 16+ | Database |
| Tauri CLI | 2.0+ | Desktop build |
| Docker | 24+ | Container runtime |
| Docker Compose | 2.20+ | Multi-container orchestration |

---

## 3. Docker Deployment

### 3.1 Docker Compose Configuration

The `docker-compose.yml` at the project root orchestrates all services:

```yaml
version: "3.9"

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: haqly_erp
      POSTGRES_USER: haqly
      POSTGRES_PASSWORD: ${DB_PASSWORD:-haqly_dev}
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U haqly"]
      interval: 5s
      timeout: 5s
      retries: 5

  api:
    build:
      context: ./src-server
      dockerfile: Dockerfile
    environment:
      DATABASE_URL: postgresql://haqly:${DB_PASSWORD:-haqly_dev}@postgres:5432/haqly_erp
      JWT_SECRET: ${JWT_SECRET:-dev_secret_change_in_production}
      AI_SIDECAR_URL: http://ai:8200
      NRS_API_KEY: ${NRS_API_KEY:-}
      NRS_BASE_URL: ${NRS_BASE_URL:-https://sandbox.api.nrs.ng/v1}
      UPLOAD_DIR: /data/uploads
      LOG_LEVEL: info
    ports:
      - "8100:8100"
    volumes:
      - uploads:/data/uploads
    depends_on:
      postgres:
        condition: service_healthy

  web:
    build:
      context: ./src-web
      dockerfile: Dockerfile
    ports:
      - "3001:3001"
    depends_on:
      - api

  ai:
    build:
      context: ./src-ai
      dockerfile: Dockerfile
    environment:
      AI_LLM_PROVIDER: ${AI_LLM_PROVIDER:-openai}
      AI_LLM_MODEL: ${AI_LLM_MODEL:-gpt-4o-mini}
      AI_LLM_API_KEY: ${AI_LLM_API_KEY:-}
      AI_MODELS_DIR: /data/ai/models
      AI_DATASETS_DIR: /data/ai/datasets
    ports:
      - "8200:8200"
    volumes:
      - ai_models:/data/ai/models
      - ai_datasets:/data/ai/datasets
    depends_on:
      - api

volumes:
  pgdata:
  uploads:
  ai_models:
  ai_datasets:
```

### 3.2 Starting Services

```bash
# Start all services
docker compose up -d

# View logs
docker compose logs -f api

# Stop all services
docker compose down

# Reset database
docker compose down -v
docker compose up -d
```

### 3.3 Backend Dockerfile (src-server/Dockerfile)

```dockerfile
FROM rust:1.80-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/haqly-server /usr/local/bin/
EXPOSE 8100
CMD ["haqly-server"]
```

### 3.4 Frontend Dockerfile (src-web/Dockerfile)

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/out /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 3001
CMD ["nginx", "-g", "daemon off;"]
```

### 3.5 AI Sidecar Dockerfile (src-ai/Dockerfile)

```dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 8200
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8200"]
```

---

## 4. Tauri Desktop Build

### 4.1 Prerequisites by Platform

**Windows:**
- Visual Studio Build Tools 2022 (C++ workload)
- WebView2 (installed by default on Windows 11)
- Rust toolchain: `rustup target add x86_64-pc-windows-msvc`

**macOS:**
- Xcode Command Line Tools: `xcode-select --install`
- Rust toolchain: `rustup target add aarch64-apple-darwin` (Apple Silicon) or `x86_64-apple-darwin` (Intel)

**Linux (Ubuntu):**
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
rustup target add x86_64-unknown-linux-gnu
```

### 4.2 Building the Desktop App

```bash
# Install Tauri CLI
cargo install tauri-cli

# Build for current platform
cargo tauri build

# Build for specific target
cargo tauri build --target x86_64-pc-windows-msvc
cargo tauri build --target aarch64-apple-darwin
cargo tauri build --target x86_64-unknown-linux-gnu
```

### 4.3 Build Artifacts

| Platform | Output Location | File |
|---|---|---|
| Windows | `src-tauri/target/release/bundle/nsis/` | `HAQLY ERP_0.1.0_x64-setup.exe` |
| Windows | `src-tauri/target/release/bundle/msi/` | `HAQLY ERP_0.1.0_x64_en-US.msi` |
| macOS | `src-tauri/target/release/bundle/dmg/` | `HAQLY ERP_0.1.0_aarch64.dmg` |
| Linux | `src-tauri/target/release/bundle/appimage/` | `haqly-erp_0.1.0_amd64.AppImage` |
| Linux | `src-tauri/target/release/bundle/deb/` | `haqly-erp_0.1.0_amd64.deb` |

### 4.4 Sidecar Bundling

The Axum server binary and Python AI engine are bundled as Tauri sidecars:

In `tauri.conf.json`:
```json
{
  "externalBin": [
    "binaries/haqly-server",
    "binaries/ai-engine"
  ]
}
```

Each sidecar binary must be placed at:
```
src-tauri/binaries/haqly-server-x86_64-pc-windows-msvc.exe
src-tauri/binaries/ai-engine-x86_64-pc-windows-msvc.exe
```

The target triple suffix is required by Tauri's sidecar convention.

---

## 5. PostgreSQL Setup

### 5.1 Database Creation

```sql
CREATE DATABASE haqly_erp
  WITH ENCODING = 'UTF8'
  LC_COLLATE = 'en_US.utf8'
  LC_CTYPE = 'en_US.utf8'
  TEMPLATE = template0;

CREATE USER haqly WITH PASSWORD 'change_me_in_production';
GRANT ALL PRIVILEGES ON DATABASE haqly_erp TO haqly;
```

### 5.2 Running Migrations

```bash
# Using sqlx CLI
sqlx database create --database-url postgresql://haqly:password@localhost:5432/haqly_erp
sqlx migrate run --source src-server/migrations --database-url postgresql://haqly:password@localhost:5432/haqly_erp
```

### 5.3 Production Considerations

- Enable SSL: `sslmode=verify-full` in connection string.
- Set `work_mem = 64MB` for complex report queries.
- Set `shared_buffers = 256MB` (or 25% of available RAM).
- Configure `pg_stat_statements` for query monitoring.
- Schedule daily `VACUUM ANALYZE` via cron.
- Set up WAL archiving for point-in-time recovery.

---

## 6. Sidecar Management

### 6.1 Startup Sequence

1. Tauri launches the main window.
2. Tauri spawns the `haqly-server` sidecar on port 8100.
3. The webview loads `http://localhost:8100` or the static export.
4. If AI features are enabled, Tauri spawns the `ai-engine` sidecar on port 8200.
5. The Axum server connects to PostgreSQL.
6. The Axum server performs migration check on startup.

### 6.2 Health Monitoring

The Tauri app monitors sidecar health every 30 seconds:
```rust
async fn check_sidecar_health(port: u16) -> bool {
    reqwest::get(format!("http://localhost:{}/health", port))
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
```

If a sidecar becomes unhealthy:
1. Retry 3 times with 5-second intervals.
2. If still unhealthy, show an error banner in the UI.
3. Offer a "Restart Services" button.
4. Log the event for diagnostics.

### 6.3 Shutdown

On window close:
1. Tauri sends SIGTERM to both sidecars.
2. Waits up to 10 seconds for graceful shutdown.
3. If processes don't stop, sends SIGKILL.

---

## 7. Environment Configuration

### 7.1 .env File

Copy `.env.example` to `.env` and fill in values:

```bash
# Database
DATABASE_URL=postgresql://haqly:haqly_dev@localhost:5432/haqly_erp

# Auth
JWT_SECRET=your-256-bit-secret-change-in-production
JWT_EXPIRY_HOURS=8

# AI Sidecar
AI_SIDECAR_URL=http://localhost:8200
AI_LLM_PROVIDER=openai
AI_LLM_MODEL=gpt-4o-mini
AI_LLM_API_KEY=sk-...

# E-Invoicing
NRS_API_KEY=your-nrs-api-key
NRS_BASE_URL=https://sandbox.api.nrs.ng/v1

# Storage
UPLOAD_DIR=~/.haqly/uploads

# Logging
LOG_LEVEL=info
RUST_LOG=haqly_server=info,sqlx=warn
```

### 7.2 Production Overrides

In production, override via environment variables (not `.env` file):
- `DATABASE_URL` — Production PostgreSQL connection string with SSL.
- `JWT_SECRET` — Cryptographically random 256-bit key.
- `NRS_API_KEY` — Production NRS key (not sandbox).
- `NRS_BASE_URL` — `https://api.nrs.ng/v1` (production endpoint).
- `LOG_LEVEL` — `warn` or `error` in production.

---

## 8. Update Mechanism

Tauri's built-in updater handles desktop application updates:

In `tauri.conf.json`:
```json
{
  "updater": {
    "active": true,
    "endpoints": ["https://releases.haqly.com/updates/{{target}}/{{arch}}/{{current_version}}"],
    "pubkey": "PUBLIC_KEY_HERE"
  }
}
```

Update flow:
1. Tauri checks the endpoint on launch and every 6 hours.
2. If an update is available, the user is prompted.
3. The update is downloaded and verified against the public key.
4. On restart, the new version is installed.

---

## 9. Backup Strategy

| Component | Method | Frequency |
|---|---|---|
| PostgreSQL | `pg_dump` + WAL archiving | Daily full, continuous WAL |
| Uploaded Files | Filesystem backup (rsync/S3 sync) | Daily |
| AI Models | Versioned in `~/.haqly/ai/models/` with rollback | On change |
| Configuration | `.env` file backup | On change |

---

## 10. Troubleshooting

| Issue | Cause | Resolution |
|---|---|---|
| "Cannot connect to database" | PostgreSQL not running | Start PostgreSQL, check DATABASE_URL |
| "Sidecar failed to start" | Port 8100 in use | Kill existing process: `lsof -i :8100` or `netstat` |
| "Migration failed" | Schema mismatch | Check migration files, run `sqlx migrate info` |
| "Blank screen in Tauri" | Static export missing | Run `npm run build` in src-web first |
| "AI features unavailable" | Python sidecar not running | Check `http://localhost:8200/health` |
| "NRS API errors" | Invalid API key or sandbox mode | Verify NRS_API_KEY and NRS_BASE_URL |
