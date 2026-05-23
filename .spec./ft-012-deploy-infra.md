# FT-012 — Deploy e Infraestrutura (Docker + Config + Healthcheck)

**Módulo:** Deploy | **Versão:** v5.9 | **Prioridade:** P1 — Alto  
**Artefato ID:** FT-012-DEPLOY | **Atualização:** 2026-05-23

---

## 1. Contexto e Objetivo

Deploy real do KCE em containers Docker com configuração 12-factor (env vars), healthcheck inteligente e readiness probes. Sem deploy funcional, o sistema é apenas código — não produto. Inclui Docker, docker-compose e preparação para K8s.

---

## 2. Critérios de Aceite (AC)

- [ ] AC-010: Dockerfile multi-stage (builder + runtime slim)
- [ ] AC-011: docker-compose funcional com volumes persistentes
- [ ] AC-012: Configuração via env vars (12-factor): `KINESQL_PATH`, `WAL_PATH`, `PORT`, `API_KEY`
- [ ] AC-013: `/health` endpoint inteligente (verifica WAL, DB, latência)
- [ ] AC-014: Startup time < 2s
- [ ] AC-015: Image size < 100MB
- [ ] AC-016: Graceful shutdown (flush WAL + close connections)
- [ ] AC-017: dotenv suportado para dev

---

## 3. Definition of Done (DoD)

- [ ] Dockerfile funcional e otimizado
- [ ] docker-compose com volume mapping
- [ ] Config via `config` + `dotenv` crates
- [ ] Healthcheck inteligente (3 checks)
- [ ] Graceful shutdown implementado
- [ ] Deploy testado end-to-end

---

## 4. Exemplos de Uso

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

### GET /health (inteligente)
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

### GET /health (degradado)
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

## 5. Planos de Teste

### 5.1 Testes Unitários

| ID | Caso | Saída Esperada |
|----|------|----------------|
| UT-001 | Config carrega de env var | valor correto |
| UT-002 | Config fallback para default | default funciona |
| UT-003 | Health — tudo ok | `status: healthy` |
| UT-004 | Health — DB down | `status: degraded` |

### 5.2 Testes Funcionais

| ID | Cenário | Resultado |
|----|---------|-----------|
| FT-001 | `docker build` | Imagem criada < 100MB |
| FT-002 | `docker-compose up` | Sistema inicia, /health retorna 200 |
| FT-003 | `docker stop` (graceful) | WAL flushed, sem corrupção |

### 5.3 Testes de Integração

| ID | Componentes | Resultado |
|----|-------------|-----------|
| IT-001 | Docker → API → KineSQL | Pipeline funcional em container |
| IT-002 | Docker restart → recovery | Dados recuperados do volume |

---

## 6. Formato CARE

**Context:** Deploy containerizado é obrigatório para produção; configuração 12-factor para portabilidade; healthcheck inteligente para orquestração.

**Assumptions:** Docker disponível em prod; volumes persistentes para data/WAL; K8s é futuro (readiness probe preparado).

**Requirements:** R-001: Dockerfile multi-stage | R-002: docker-compose | R-003: Config 12-factor | R-004: Healthcheck inteligente | R-005: Graceful shutdown | R-006: Image < 100MB.

**Evidence:** `Dockerfile` | `docker-compose.yml` | `.env.example`

---

## 7–11. (Resumo)

**Não funcionais:** Startup < 2s | Image < 100MB | Shutdown < 5s (flush WAL)  
**Qualidade:** 0 corrupção em stop/restart | Config 100% via env | Cobertura healthcheck ≥ 3 componentes  
**Deps:** `config` ^0.14, `dotenv` ^0.15  
**Rastreabilidade:** FT-012-DEPLOY | `Dockerfile`, `docker-compose.yml`, `.env.example`

**Roadmap:** MVP: Dockerfile + docker-compose + .env | Iter1: healthcheck inteligente + graceful shutdown | Iter2: K8s manifests + readiness/liveness probes + autoscaling
