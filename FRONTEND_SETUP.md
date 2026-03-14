# Frontend Setup Guide

Quick guide to get the DRASM frontend running.

## Prerequisites

- Node.js 18+ (recommended: 20+)
- npm (comes with Node.js)

## Quick Start

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Create environment file
cp .env.example .env

# Start development server
npm run dev
```

The frontend will start on **http://localhost:3001** (since the orchestrator is on 3000).

## Configuration

The `.env` file should contain:

```env
NUXT_PUBLIC_API_BASE=http://localhost:3000
```

This points to your orchestrator API. Update if running on a different host/port.

## Testing the Frontend

### 1. Start Backend Services First

Make sure these are running:
- Kafka & Redis: `docker-compose up -d`
- Supabase: `npx supabase start`
- Orchestrator: `cd orchestrator && cargo run --release`
- Worker: `cd worker && cargo run --release`

### 2. Access the Frontend

Open http://localhost:3001 in your browser.

### 3. Test User Flow

1. **Register**: Click "Register" → Create account with email/password
2. **Login**: After registration, you'll be auto-logged in
3. **Upload Module**: 
   - Go to "Modules" → "Upload Module"
   - Upload a `.wasm` file (e.g., `examples/target/wasm32-unknown-unknown/release/echo.wasm`)
   - Give it a name like "echo"
4. **Create Job**:
   - Go to "Jobs" → "Create Job"
   - Select your module
   - Type name: `Request`
   - Payload: `{"data": "Hello World!"}`
   - Submit
5. **View Results**:
   - Click "View Details" on your job
   - Click "Refresh Status" to see updates
   - When completed, you'll see the result

## Troubleshooting

### Port 3001 Already in Use

If port 3001 is taken, Nuxt will automatically try the next available port (3002, 3003, etc.).

### CORS Errors

If you see CORS errors, make sure the orchestrator has CORS enabled (it should by default with `CorsLayer`).

### 401 Unauthorized

- Make sure you're logged in
- JWT tokens expire after some time, try logging out and back in
- Check that `NUXT_PUBLIC_API_BASE` points to the correct orchestrator URL

### Module Upload Fails

- Check file is `.wasm` format
- Check file size is under 10MB
- Make sure Supabase is running: `npx supabase status`

### Job Never Completes

- Check worker is running and processing jobs
- Check worker logs for errors
- Verify Kafka is running: `docker-compose ps`
