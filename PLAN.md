# DRASM MVP Implementation Plan

## 🎯 Goal
Build a production-ready distributed WASM execution platform with Supabase integration for auth, storage, and database.

## 🏗️ Architecture Overview

```
Frontend (Future)
    │ JWT Auth
    ▼
┌─────────────────┐      ┌──────────────┐
│  Orchestrator   │◄────►│   Supabase   │
│  (Axum REST)    │      │ - Auth       │
└────┬────────┬───┘      │ - Storage    │
     │        │          │ - Postgres   │
     │        │          └──────────────┘
     │        │
     │        │          ┌──────────────┐
     │        └─────────►│    Kafka     │
     │                   │ (Job Queue)  │
     │                   └──────┬───────┘
     │                          │
     │                          ▼
     │                   ┌──────────────┐      ┌──────────────┐
     │                   │   Workers    │◄────►│    Redis     │
     │                   │ (N replicas) │      │ (Idempotency)│
     │                   └──────┬───────┘      └──────────────┘
     │                          │
     │                          │ Download WASM
     └──────────────────────────┼──────────────────────┐
                                ▼                      │
                         ┌──────────────┐             │
                         │   Supabase   │◄────────────┘
                         │   Storage    │
                         └──────────────┘
```

## 📦 Services

### Use Supabase For:
- ✅ WASM module storage (Supabase Storage)
- ✅ User authentication (Supabase Auth)
- ✅ Module registry (Postgres)
- ✅ Job history/metadata (Postgres)
- ✅ User quotas/limits (Postgres)

### Keep Existing:
- ✅ Kafka - Job queue (high throughput)
- ✅ Redis - Idempotency tracking (sub-ms performance)

---

## ✅ Phase 1: Core MVP (COMPLETED)

### ✅ Prerequisites (DONE)
- [x] Install Supabase CLI (`npx supabase`)
- [x] Initialize Supabase locally (`npx supabase init`)
- [x] Start Supabase locally (`npx supabase start`)

### ✅ 1. Database Schema Setup (DONE)

**Create migrations:**
```bash
npx supabase migration new initial_schema
```

**Tables to create:**

```sql
-- Module registry
CREATE TABLE modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    storage_path TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL UNIQUE,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Job metadata
CREATE TABLE jobs (
    job_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    module_id UUID REFERENCES modules(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    input_message JSONB NOT NULL,
    result JSONB,
    error TEXT,
    worker_id TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    
    CONSTRAINT valid_status CHECK (status IN ('pending', 'processing', 'completed', 'failed'))
);

-- Indexes
CREATE INDEX idx_jobs_user_id ON jobs(user_id);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_created_at ON jobs(created_at DESC);
CREATE INDEX idx_modules_user_id ON modules(user_id);
CREATE INDEX idx_modules_hash ON modules(hash);

-- Row Level Security (RLS)
ALTER TABLE modules ENABLE ROW LEVEL SECURITY;
ALTER TABLE jobs ENABLE ROW LEVEL SECURITY;

-- Policies (users can only see their own data)
CREATE POLICY "Users can view own modules" ON modules
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own modules" ON modules
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can view own jobs" ON jobs
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own jobs" ON jobs
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own jobs" ON jobs
    FOR UPDATE USING (auth.uid() = user_id);
```

**Apply migration:**
```bash
npx supabase db reset  # Apply all migrations
```

### ✅ 2. Storage Bucket Setup (DONE)

**Via Supabase Studio (http://localhost:54323):**
1. Go to Storage
2. Create bucket: `wasm-modules`
3. Set to **private** (auth required)
4. Set size limit: 10MB per file

**Or via SQL:**
```sql
INSERT INTO storage.buckets (id, name, public)
VALUES ('wasm-modules', 'wasm-modules', false);

-- Storage policies
CREATE POLICY "Users can upload own modules"
ON storage.objects FOR INSERT
WITH CHECK (
    bucket_id = 'wasm-modules' 
    AND auth.uid()::text = (storage.foldername(name))[1]
);

CREATE POLICY "Users can read own modules"
ON storage.objects FOR SELECT
USING (
    bucket_id = 'wasm-modules'
    AND auth.uid()::text = (storage.foldername(name))[1]
);
```

### ✅ 3. Add Axum REST API to Orchestrator (DONE)

**Dependencies:**
```toml
# orchestrator/Cargo.toml
[dependencies]
axum = "0.7"
tokio = { workspace = true, features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
postgrest = "2.0"  # Supabase client
reqwest = { workspace = true, features = ["json", "multipart"] }
```

**Endpoints to implement:**

- `POST /auth/signup` - Create user (proxy to Supabase)
- `POST /auth/login` - Login user (proxy to Supabase)
- `POST /modules` - Upload WASM module
- `GET /modules` - List user's modules
- `POST /jobs` - Submit job for execution
- `GET /jobs/:job_id` - Get job status and result
- `GET /jobs` - List user's jobs
- `GET /health` - Health check

**Structure:**
```
orchestrator/
├── src/
│   ├── main.rs           # Axum server + Kafka consumer
│   ├── api/
│   │   ├── mod.rs
│   │   ├── auth.rs       # Auth endpoints
│   │   ├── modules.rs    # Module upload/list
│   │   └── jobs.rs       # Job submission/status
│   ├── supabase.rs       # Supabase client wrapper
│   ├── kafka.rs          # Kafka producer
│   └── middleware/
│       └── auth.rs       # JWT validation middleware
```

### ✅ 4. Supabase Integration in Orchestrator (DONE)

**Key tasks:**
- Validate JWT from Supabase Auth
- Upload WASM to Supabase Storage
- Insert module record in Postgres
- Insert job record in Postgres
- Publish job to Kafka
- Update job status from Kafka results

**Environment variables:**
```bash
# .env
SUPABASE_URL=http://localhost:54321
SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
KAFKA_BROKERS=localhost:9094
REDIS_URL=redis://localhost:6379
```

### ✅ 5. Update Worker for Supabase Storage (DONE)

**Dependencies:**
```toml
# worker/Cargo.toml
[dependencies]
reqwest = { workspace = true, features = ["stream"] }
tokio = { workspace = true, features = ["fs"] }
```

**Changes:**
- Download WASM from Supabase Storage using signed URLs
- Cache in `/tmp/drasm-modules/{module_id}.wasm`
- Check cache before downloading
- Use service role key for downloads (no user auth needed)

**Flow:**
1. Worker receives job with `module_id`
2. Check local cache: `/tmp/drasm-modules/{module_id}.wasm`
3. If not cached:
   - Get signed URL from Supabase Storage
   - Download WASM file
   - Save to cache
4. Execute module (existing logic)

### ✅ 6. Test End-to-End (READY FOR USER TESTING)

**See E2E_TEST.md for detailed testing instructions.**

**Test flow:**
1. Start Supabase: `npx supabase start`
2. Start Kafka/Redis: `docker-compose up -d`
3. Create topics: `./scripts/create-topics.sh`
4. Build examples: `cargo build --release --target wasm32-unknown-unknown`
5. Start worker: `cargo run --release --bin worker`
6. Start orchestrator: `cargo run --release --bin orchestrator`
7. Test API:
   ```bash
   # Sign up
   curl -X POST http://localhost:3000/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"email":"test@example.com","password":"password123"}'
   
   # Login (get JWT)
   curl -X POST http://localhost:3000/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"test@example.com","password":"password123"}'
   
   # Upload module
   curl -X POST http://localhost:3000/modules \
     -H "Authorization: Bearer <JWT>" \
     -F "file=@examples/target/wasm32-unknown-unknown/release/echo.wasm" \
     -F "name=echo"
   
   # Submit job
   curl -X POST http://localhost:3000/jobs \
     -H "Authorization: Bearer <JWT>" \
     -H "Content-Type: application/json" \
     -d '{"module_id":"<UUID>","message":{"type_name":"Request","payload":[...]}}'
   
   # Check job status
   curl http://localhost:3000/jobs/<JOB_ID> \
     -H "Authorization: Bearer <JWT>"
   ```

---

## 🚀 Phase 2: Deployment Prep (1 week)

### 7. Dockerize Services (2 hours)

**Create Dockerfiles:**

```dockerfile
# orchestrator/Dockerfile
FROM rust:1.80-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin orchestrator

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/orchestrator /usr/local/bin/
CMD ["orchestrator"]
```

```dockerfile
# worker/Dockerfile
FROM rust:1.80-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin worker

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/worker /usr/local/bin/
CMD ["worker"]
```

**Update docker-compose.yml:**
```yaml
services:
  orchestrator:
    build:
      context: .
      dockerfile: orchestrator/Dockerfile
    ports:
      - "3000:3000"
    environment:
      SUPABASE_URL: ${SUPABASE_URL}
      SUPABASE_ANON_KEY: ${SUPABASE_ANON_KEY}
      SUPABASE_SERVICE_ROLE_KEY: ${SUPABASE_SERVICE_ROLE_KEY}
      KAFKA_BROKERS: kafka:9092
      REDIS_URL: redis://redis:6379
    depends_on:
      - kafka
      - redis

  worker:
    build:
      context: .
      dockerfile: worker/Dockerfile
    environment:
      SUPABASE_URL: ${SUPABASE_URL}
      SUPABASE_SERVICE_ROLE_KEY: ${SUPABASE_SERVICE_ROLE_KEY}
      KAFKA_BROKERS: kafka:9092
      REDIS_URL: redis://redis:6379
    depends_on:
      - kafka
      - redis
    deploy:
      replicas: 2  # Run 2 workers
```

### 8. Environment Config (1 hour)

**Create `.env.example`:**
```bash
# Supabase
SUPABASE_URL=http://localhost:54321
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key

# Kafka
KAFKA_BROKERS=localhost:9094

# Redis
REDIS_URL=redis://localhost:6379

# Orchestrator
ORCHESTRATOR_PORT=3000
ORCHESTRATOR_HOST=0.0.0.0
```

**Use `dotenvy` crate:**
```toml
[dependencies]
dotenvy = "0.15"
```

### 9. Create Supabase Cloud Project (15 min)

1. Go to https://supabase.com
2. Create account (free tier)
3. Create new project
4. Copy production keys
5. Link local project:
   ```bash
   npx supabase link --project-ref <PROJECT_ID>
   ```
6. Push schema:
   ```bash
   npx supabase db push
   ```

### 10. Deploy to Cloud (2-4 hours)

**Option A: Fly.io (Recommended for MVP)**
```bash
# Install Fly CLI
curl -L https://fly.io/install.sh | sh

# Deploy orchestrator
fly launch --dockerfile orchestrator/Dockerfile --name drasm-orchestrator
fly secrets set SUPABASE_URL=... SUPABASE_ANON_KEY=... (etc)
fly deploy

# Deploy worker
fly launch --dockerfile worker/Dockerfile --name drasm-worker
fly secrets set SUPABASE_URL=... (etc)
fly scale count 2  # 2 workers
fly deploy
```

**Option B: DigitalOcean Kubernetes**
- Create K8s cluster ($12/mo)
- Use managed Kafka (Upstash: $10/mo)
- Use managed Redis (Upstash: $10/mo)
- Deploy with kubectl/helm

---

## 📊 Success Metrics

### MVP is complete when:
- ✅ User can sign up and login
- ✅ User can upload WASM module
- ✅ User can submit job
- ✅ Worker executes job from Supabase Storage
- ✅ User can check job status and result
- ✅ Idempotency works (duplicate jobs skipped)
- ✅ Multiple workers run in parallel
- ✅ Deployed to production (accessible via URL)

---

## 🔧 Development Commands

### Supabase
```bash
# Start local Supabase
npx supabase start

# Stop local Supabase
npx supabase stop

# Reset database (rerun migrations)
npx supabase db reset

# Create new migration
npx supabase migration new <name>

# Push schema to production
npx supabase db push

# View Studio UI
# http://localhost:54323
```

### Build & Run
```bash
# Build WASM examples
cargo build --release --target wasm32-unknown-unknown

# Build workspace
cargo build --release

# Run locally
cargo run --release --bin orchestrator
cargo run --release --bin worker

# Run with Docker
docker-compose up --build
```

---

## 📝 Notes

- All Supabase commands use `npx supabase` (not global install)
- Keep Kafka and Redis for performance (don't replace with Supabase)
- Use RLS policies for multi-tenancy security
- Service role key for worker downloads (bypass RLS)
- Local dev uses Supabase CLI, production uses Supabase cloud

---

## 🎯 Next Immediate Action

Start with **Phase 1, Task 1**: Create database schema migration.

Command:
```bash
npx supabase migration new initial_schema
```

Then edit the generated file in `supabase/migrations/` with the SQL schema above.