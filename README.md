# DRASM - Distributed WASM Execution System

A distributed system for executing WebAssembly modules using Kafka as the message broker and Redis for idempotency tracking. Workers execute WASM modules with a rich Message protocol and host functions for logging and HTTP requests.

## Components

1. **Orchestrator** - Submits WASM jobs to Kafka and monitors results
2. **Worker** - Consumes jobs, executes WASM modules with host functions (log, HTTP), publishes results
3. **Redis** - Idempotency tracking (24-hour TTL)
4. **Kafka** - Message broker for job queue and results
5. **Kafka UI** - Web interface for monitoring topics (http://localhost:8080)

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

### 🔌 Host Functions
- **log** - Guest modules can log messages to worker console
- **http_request** - Async HTTP calls from WASM (supports JSON APIs)
- Message protocol for structured request/response

## Prerequisites

- Docker & Docker Compose
- Rust 1.70+
- Cargo
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`

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

### 3. Build WASM Examples

```bash
# Build the example modules (echo, adder)
cargo build --release --target wasm32-unknown-unknown
```

### 4. Build and Run

**Terminal 1: Start Worker**
```bash
cargo run --release --bin worker
```

Expected output:
```
Worker starting with ID: <hostname>
Wasmtime engine and linker initialized
Connected to Redis successfully
Worker running. Waiting for jobs...
```

**Terminal 2: Start Orchestrator** (submits job and monitors results)
```bash
cargo run --release --bin orchestrator
```

Expected output:
```
Submitted job_id=<uuid> module=echo partition=X offset=Y
Orchestrator listening for results...
[guest log] echo: hello from orchestrator!
[guest log] typed url from httpbin: ...
✓ RESULT [worker=<hostname>] job_id=<uuid> type=Response payload={"data":"Echo: hello from orchestrator!"}
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

- [ ] S3 module storage (upload WASM to S3, workers fetch and cache)
- [ ] WASI support for filesystem/network access
- [ ] Resource limits (fuel metering, memory caps, timeouts)
- [ ] Metrics and tracing (Prometheus, Jaeger)
- [ ] REST API for job submission (orchestrator as web server)
- [ ] WebSocket for real-time result streaming
- [ ] Module compilation caching
- [ ] Multi-broker Kafka cluster (replication-factor=3)
