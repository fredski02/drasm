# DRASM - Distributed WASM Execution System

A distributed system for executing WebAssembly modules using Kafka as the message broker and Redis for idempotency tracking.

## Components

1. **Producer** - Submits WASM jobs (WAT format) to Kafka
2. **Worker** - Consumes jobs, executes WASM via Wasmtime, publishes results
3. **Results Consumer** - Monitors and displays job results in real-time
4. **Redis** - Idempotency tracking (24-hour TTL)
5. **Kafka** - Message broker for job queue and results
6. **Kafka UI** - Web interface for monitoring topics (http://localhost:8080)

## Topics

- `wasm_jobs` (3 partitions) - Job queue
- `wasm_results` (3 partitions) - Execution results  
- `wasm_jobs_dlq` (3 partitions) - Failed jobs after 3 retry attempts

## Features

### 🔄 Idempotent Processing
- Jobs processed exactly once using Redis
- Simple EXISTS check before processing
- 24-hour TTL for automatic cleanup
- No complex state management

### 🔁 Automatic Retries & DLQ
- Failed jobs retry up to 3 times
- After max retries, jobs move to DLQ
- Prevents poison pills from blocking partitions
- DLQ visible in Kafka UI for manual inspection

### 📊 Horizontal Scalability
- Multiple workers via Kafka consumer groups
- Automatic partition rebalancing
- Each partition processed by one worker
- Add/remove workers dynamically

### 💪 Durability & Reliability
- `acks=all` - Full replication before acknowledgment
- `enable.idempotence=true` - No duplicate sends
- Manual offset commits after successful processing
- Redis for fast, reliable deduplication

## Prerequisites

- Docker & Docker Compose
- Rust 1.70+
- Cargo

## Quick Start

### 1. Start Infrastructure

```bash
# Start Kafka, Redis, and Kafka UI
docker-compose up -d

# Verify services are running
docker-compose ps
```

### 2. Create Kafka Topics

```bash
./scripts/create-topics.sh
```

### 3. Build Project

```bash
cargo build --release
```

### 4. Run the System

**Terminal 1: Start Worker**
```bash
./target/release/worker
```

Expected output:
```
Worker starting with ID: <hostname>
Connected to Redis successfully
Worker running. Waiting for jobs...
```

**Terminal 2: Start Results Consumer** (optional)
```bash
./target/release/results
```

Expected output:
```
Results consumer running (showing new results only)...
```

**Terminal 3: Submit Jobs**
```bash
./target/release/producer
```

Expected output:
```
Submitted job_id=<uuid> partition=X offset=Y
```

### 5. Monitor via Kafka UI

Open browser: http://localhost:8080

- View topics and messages
- Inspect DLQ for failed jobs
- Monitor consumer group lag
- See partition distribution

## Redis

Redis is used for idempotency tracking. Each job ID is stored with a "completed" value and 24-hour TTL.

### Quick Commands

```bash
# Check if job exists
redis-cli EXISTS <job-id>

# Get job status
redis-cli GET <job-id>

# Reset all state (clears idempotency cache)
redis-cli FLUSHDB
```

## Potential Improvements

- [ ] WASI support for filesystem/network access
- [ ] Multiple function exports per module
- [ ] Richer input/output types (strings, bytes, structs)
- [ ] Resource limits (fuel metering, memory caps, timeouts)
- [ ] Metrics and tracing (Prometheus, Jaeger)
- [ ] REST API for job submission
- [ ] WebSocket for real-time result streaming
- [ ] Module caching to avoid recompilation
- [ ] Multi-broker Kafka cluster (replication-factor=3)
