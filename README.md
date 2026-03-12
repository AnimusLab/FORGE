# 🔨 FORGE
### File-Oriented Rust Grade Engine

FORGE is a high-performance Rust engine that turns free cloud storage (Google Drive, MEGA, pCloud) into a fully queryable database. Bring your own storage. Own your data. Pay nothing.

---

## Why FORGE?

Every cloud database charges you eventually. Supabase, PlanetScale, Firebase — free tiers run out, and you're locked in.

FORGE flips the model. Instead of storing your data on someone else's servers, FORGE uses storage you already own — Google Drive, MEGA, or any cloud file storage — and turns it into a real database with its own binary format, query engine, and REST API.

---

## How It Works
```
Your App
    ↓  HTTPS + API Key
FORGE Engine (deployed on Railway / Fly.io)
    ↓
.forge binary files
    ↓
Your Google Drive (free 15GB)
```

---

## Features

- **Custom binary format** — `.forge` files, our own spec, no third party format
- **REST API** — clean versioned endpoints under `/v1`
- **API key auth** — every request authenticated via `X-Forge-Key`
- **In-memory query engine** — built from scratch in Rust
- **Write Ahead Log** — no data loss on crashes *(Sprint 4)*
- **Google Drive sync** — your data, your storage *(Sprint 6)*
- **One-click deploy** — Docker image, runs on Railway/Fly.io free tier *(Sprint 7)*

---

## API
```
Base URL: https://your-engine.railway.app/v1

POST    /v1/data/:collection          → insert record
GET     /v1/data/:collection          → query all
GET     /v1/data/:collection/:id      → query one
PATCH   /v1/data/:collection/:id      → update record
DELETE  /v1/data/:collection/:id      → delete record
GET     /v1/collections               → list all collections
GET     /v1/health                    → engine status
```

Every protected request requires:
```
X-Forge-Key: YOUR_SECRET_KEY
```

---

## Quick Start

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone & Setup
```bash
git clone https://github.com/AnimusLab/FORGE.git
cd FORGE
cp .env.example .env
```

### 3. Configure `.env`
```
FORGE_API_KEY=your_secret_key_here
PORT=8080
```

### 4. Run
```bash
cargo run
```

### 5. Test
```bash
# Health check
curl http://localhost:8080/v1/health

# Insert
curl -X POST http://localhost:8080/v1/data/users \
  -H "X-Forge-Key: your_secret_key_here" \
  -H "Content-Type: application/json" \
  -d '{"name": "John", "age": 25}'

# Query
curl http://localhost:8080/v1/data/users \
  -H "X-Forge-Key: your_secret_key_here"
```

---

## The `.forge` Binary Format

FORGE stores data in its own binary format — not Parquet, not CSV, not someone else's spec.
```
Header (64 bytes)
├── Magic bytes: "FORGE001"
├── Version
├── Created timestamp
├── Row count
└── Schema hash

Body
├── Schema block  → field names + types
├── Index block   → row offsets for fast lookup
└── Data blocks   → binary encoded rows
```

---

## Stack

| Layer | Technology |
|---|---|
| Core Engine | Rust |
| Web Framework | axum |
| Async Runtime | tokio |
| TLS | rustls |
| File Format | `.forge` (custom) |
| Query Engine | Custom (Rust) |
| Storage Backend | Google Drive API |
| Deployment | Docker + Railway |

---

## Roadmap

- [x] Sprint 1 — REST API + auth
- [x] Sprint 2 — `.forge` binary format
- [x] Sprint 3 — query engine wiring
- [ ] Sprint 4 — WAL + idempotency
- [ ] Sprint 5 — index engine
- [ ] Sprint 6 — Google Drive sync
- [ ] Sprint 7 — Docker + one-click deploy
- [ ] Sprint 8 — FORGE CLI

---

## License

MIT © [AnimusLab](https://github.com/AnimusLab)