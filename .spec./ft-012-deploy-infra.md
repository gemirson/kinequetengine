# FT-012 — Deploy and Infrastructure (Docker + Config + Healthcheck)

**Module:** Deploy | **Version:** v5.9 | **Priority:** P1 — High  
**Artifact ID:** FT-012-DEPLOY | **Update:** 2026-05-23

---

## 1. Context and Objective

Real deployment of KCE in Docker containers with 12-factor configuration (env vars), smart healthcheck and readiness probes. Without functional deployment, the system is just code — not a product. Includes Docker, docker-compose and K8s preparation.

---

## 2. Acceptance Criteria (AC)

- [ ] AC-010: Multi-stage Dockerfile (builder + slim runtime)
- [ ] AC-011: functional docker-compose with persistent volumes
- [ ] AC-012: Configuration via env vars (12-factor): `KINESQL_PATH`, `WAL_PATH`, `PORT`, `API_KEY`
- [ ] AC-013: `/health` smart endpoint (checks WAL, DB, latency)
- [ ] AC-014: Startup time < 2s
- [ ] AC-015: Image size < 100MB
- [ ] AC-016: Graceful shutdown (flush WAL + close connections)
- [ ] AC-017: dotenv supported for dev

---

## 3. Definition of Done (DoD)

- [ ] Functional and optimized Dockerfile
- [ ] docker-compose with volume mapping
- [ ] Config via `config` + `dotenv` crates
- [ ] Smart healthcheck (3 checks)
- [ ] Graceful shutdown implemented
- [ ] End-to-end tested deployment

---

## 4. Usage Examples

### docker-compose.yml
```yaml
version: "3"
services:
  kce:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./data:/data
    environment:
      - KINESQL_PATH=/data/db.kinesql
      - WAL_PATH=/data/wal.log
      - PORT=3000
      - API_KEY=tk_prod_abc123
```

### .env (dev)
```env
KINESQL_PATH=./data/db.kinesql
WAL_PATH=./data/wal.log
PORT=3000
API_KEY=tk_dev_local
```

### GET /health (smart)
```json
{
  "status": "healthy",
  "components": {
    "wal_ok": true,
    "db_ok": true,
    "latency_ok": true
  },
  "version": "5.9.0",
  "uptime_seconds": 86400
}
```

### GET /health (degraded)
```json
{
  "status": "degraded",
  "components": {
    "wal_ok": true,
    "db_ok": false,
    "latency_ok": true
  },
  "message": "Database read error detected"
}
```

---

## 5. Test Plans

### 5.1 Unit Tests

| ID | Case | Expected Output |
|----|------|----------------|
| UT-001 | Config loads from env var | correct value |
| UT-002 | Config fallback to default | default works |
| UT-003 | Health — everything ok | `status: healthy` |
| UT-004 | Health — DB down | `status: degraded` |

### 5.2 Functional Tests

| ID | Scenario | Result |
|----|---------|-----------|
| FT-001 | `docker build` | Image created < 100MB |
| FT-002 | `docker-compose up` | System starts, /health returns 200 |
| FT-003 | `docker stop` (graceful) | WAL flushed, no corruption |

### 5.3 Integration Tests

| ID | Components | Result |
|----|-------------|-----------|
| IT-001 | Docker → API → KineSQL | Functional pipeline in container |
| IT-002 | Docker restart → recovery | Data retrieved from the volume |

---

## 6. CARE Format

**Context:** Containerized deployment is mandatory for production; 12-factor configuration for portability; Smart healthcheck for orchestration.

**Assumptions:** Docker available in prod; persistent volumes for data/WAL; K8s is future (readiness probe prepared).

**Requirements:** R-001: Multi-stage Dockerfile | R-002: docker-compose | R-003: 12-factor Config | R-004: Smart Healthcheck | R-005: Graceful shutdown | R-006: Image < 100MB.

**Evidence:** `Dockerfile` | `docker-compose.yml` | `.env.example`

---

## 7–11. (Summary)

**Non-functional:** Startup < 2s | Image < 100MB | Shutdown < 5s (WAL flush)  
**Quality:** 0 corruption in stop/restart | Config 100% via env | healthcheck coverage ≥ 3 components  
**Deps:** `config` ^0.14, `dotenv` ^0.15  
**Traceability:** FT-012-DEPLOY | `Dockerfile`, `docker-compose.yml`, `.env.example`

**Roadmap:** MVP: Dockerfile + docker-compose + .env | Iter1: smart healthcheck + graceful shutdown | Iter2: K8s manifests + readiness/liveness probes + autoscaling
