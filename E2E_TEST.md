# End-to-End Testing Guide

This guide walks through testing the complete DRASM system with Supabase integration.

## Prerequisites

1. **Install Dependencies**
   - Docker and Docker Compose
   - Rust toolchain with `wasm32-unknown-unknown` target
   - Supabase CLI (via npx)

2. **Add WASM Target** (if not already installed)
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Step 1: Build WASM Examples

Build the example WASM modules that will be uploaded:

```bash
# Build the echo example
cd examples/echo
cargo build --release --target wasm32-unknown-unknown

# Build the adder example (if available)
cd ../adder
cargo build --release --target wasm32-unknown-unknown

# Return to project root
cd ../..
```

The compiled WASM files will be at:
- `examples/target/wasm32-unknown-unknown/release/echo.wasm`
- `examples/target/wasm32-unknown-unknown/release/adder.wasm`

## Step 2: Start All Services

From the project root:

```bash
# Start Kafka, Redis, and Supabase
docker-compose up -d

# Wait for services to be ready (especially Kafka)
sleep 10

# Start Supabase locally
npx supabase start

# Verify Supabase is running and note the credentials
npx supabase status
```

The `.env` file already contains the default local Supabase credentials.

## Step 3: Start the Orchestrator

In a new terminal:

```bash
cd orchestrator
cargo run --release
```

You should see:
```
Orchestrator starting...
HTTP server listening on 0.0.0.0:3000
Results consumer task started
```

## Step 4: Start the Worker

In another terminal:

```bash
cd worker
cargo run --release
```

You should see:
```
Worker starting with ID: <hostname>
Wasmtime engine and linker initialized
Connecting to Redis...
Connected to Redis successfully
Worker running. Waiting for jobs...
```

## Step 5: Run E2E Test

In a new terminal, run the following test sequence:

### 5.1 Sign Up a User

```bash
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "testpassword123"
  }'
```

Expected response:
```json
{
  "id": "<user_id>",
  "email": "test@example.com",
  ...
}
```

### 5.2 Login

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "testpassword123"
  }'
```

Expected response:
```json
{
  "access_token": "eyJhbGc...",
  "user": { ... }
}
```

**Save the `access_token` for subsequent requests!**

### 5.3 Upload a WASM Module

Replace `<YOUR_TOKEN>` with the access token from the login response:

```bash
curl -X POST http://localhost:3000/modules \
  -H "Authorization: Bearer <YOUR_TOKEN>" \
  -F "file=@examples/target/wasm32-unknown-unknown/release/echo.wasm" \
  -F "name=echo"
```

Expected response:
```json
{
  "id": "<module_id>",
  "name": "echo",
  "storage_path": "<user_id>/echo-<hash>.wasm",
  "hash": "sha256_hash_here",
  "size_bytes": 12345,
  "created_at": "2026-03-13T..."
}
```

**Save the `id` (module_id) for the next step!**

### 5.4 List Your Modules

```bash
curl -X GET http://localhost:3000/modules \
  -H "Authorization: Bearer <YOUR_TOKEN>"
```

Expected response: Array of your modules

### 5.5 Submit a Job

Replace `<YOUR_TOKEN>` and `<MODULE_ID>`:

**Note:** The `payload` field accepts a JSON string (not a byte array). The orchestrator will automatically convert it to bytes before sending to the worker.

```bash
curl -X POST http://localhost:3000/jobs \
  -H "Authorization: Bearer <YOUR_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{
    "module_id": "<MODULE_ID>",
    "message": {
      "type_name": "EchoRequest",
      "payload": "{\"message\":\"Hello from DRASM!\"}"
    }
  }'
```

**Alternative:** You can also pass the JSON directly as a string:
```bash
curl -X POST http://localhost:3000/jobs \
  -H "Authorization: Bearer <YOUR_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{
    "module_id": "<MODULE_ID>",
    "message": {
      "type_name": "EchoRequest",
      "payload": "{\"message\":\"Hello from DRASM!\"}"
    }
  }'
```

Expected response:
```json
{
  "job_id": "<job_id>",
  "module_id": "<module_id>",
  "status": "pending",
  ...
}
```

**Save the `job_id`!**

### 5.6 Check Job Status

Wait a few seconds for the worker to process the job, then:

```bash
curl -X GET http://localhost:3000/jobs/<JOB_ID> \
  -H "Authorization: Bearer <YOUR_TOKEN>"
```

Expected response (once completed):
```json
{
  "id": "<job_id>",
  "module_id": "<module_id>",
  "status": "completed",
  "result": "<base64_encoded_result>",
  "created_at": "...",
  "updated_at": "..."
}
```

The `result` field contains the base64-encoded response Message from the WASM module.

### 5.7 List Your Jobs

```bash
curl -X GET http://localhost:3000/jobs \
  -H "Authorization: Bearer <YOUR_TOKEN>"
```

Expected response: Array of your jobs

## Step 6: Verify in Supabase Studio

You can also inspect the database directly:

```bash
# Open Supabase Studio
npx supabase studio
```

Navigate to:
- **Table Editor** → `modules` - see uploaded modules
- **Table Editor** → `jobs` - see job records
- **Storage** → `wasm-modules` - see uploaded WASM files

## Step 7: Monitor Logs

### Worker Logs

Watch the worker terminal for:
```
Downloading module <module_id> from Supabase Storage...
Downloaded and cached module: <module_id>
[guest log] <any logs from the WASM module>
Processed job <job_id> with module <module_id> -> EchoResponse
```

### Orchestrator Logs

Watch the orchestrator terminal for:
```
Received result for job <job_id>
Updated job status to completed
```

### Kafka UI (optional)

Open http://localhost:8080 to see:
- Topics: `wasm_jobs`, `wasm_results`, `wasm_jobs_dlq`
- Messages being produced and consumed

## Troubleshooting

### Module Not Found Error

If the worker logs show "Module not found in database":
- Verify the module was uploaded successfully
- Check the module ID is correct
- Ensure the worker has `SUPABASE_SERVICE_ROLE_KEY` set

### Authentication Errors

If you get 401/403 errors:
- Verify the JWT token is valid and not expired
- Check you're passing the token in the `Authorization: Bearer <token>` header
- Tokens expire after 1 hour by default

### Worker Can't Download Module

If download fails:
- Check Supabase is running: `npx supabase status`
- Verify `.env` has correct `SUPABASE_URL` and `SUPABASE_SERVICE_ROLE_KEY`
- Check storage bucket exists in Supabase Studio

### Job Stays in "pending" Status

If jobs don't complete:
- Verify the worker is running and connected to Kafka
- Check worker logs for errors
- Ensure Redis is running: `docker-compose ps`
- Verify Kafka topics exist in Kafka UI

## Testing with Different Modules

To test with the `adder` module:

1. Build it: `cd examples/adder && cargo build --release --target wasm32-unknown-unknown`
2. Upload it with `name=adder`
3. Submit a job with appropriate `message` structure for the adder module

## Cleanup

```bash
# Stop services
docker-compose down

# Stop Supabase
npx supabase stop

# Clear module cache (optional)
rm -rf /tmp/drasm-modules
```

## Next Steps

After successful E2E testing:
- Implement additional WASM modules
- Add more job status tracking
- Implement job cancellation
- Add monitoring and metrics
- Deploy to production environment