mod retry;

use common::*;
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message as KafkaMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use redis::AsyncCommands;
use retry::RetryTracker;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use wasmtime::{Caller, Engine, FuncType, Linker, Module, Store, Val, ValType};
use worker_api::{Message, WasmSlice};

const GROUP_ID: &str = "wasm-workers";
const MODULE_CACHE_DIR: &str = "/tmp/drasm-modules";

// Get worker ID (hostname or UUID)
fn get_worker_id() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

/// Download WASM module from Supabase Storage if not cached
async fn download_module(
    module_id: &str,
    supabase_url: &str,
    service_role_key: &str,
) -> anyhow::Result<String> {
    // Check cache first
    tokio::fs::create_dir_all(MODULE_CACHE_DIR).await?;
    let cache_path = format!("{}/{}.wasm", MODULE_CACHE_DIR, module_id);
    
    if Path::new(&cache_path).exists() {
        println!("Using cached module: {}", module_id);
        return Ok(cache_path);
    }

    println!("Downloading module {} from Supabase Storage...", module_id);

    // Query database to get storage_path for this module_id
    let client = reqwest::Client::new();
    let db_url = format!(
        "{}/rest/v1/modules?id=eq.{}&select=storage_path",
        supabase_url, module_id
    );
    
    let response = client
        .get(&db_url)
        .header("apikey", service_role_key)
        .header("Authorization", &format!("Bearer {}", service_role_key))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Module {} not found in database", module_id);
    }

    let modules: Vec<serde_json::Value> = response.json().await?;
    let storage_path = modules
        .first()
        .and_then(|m| m.get("storage_path"))
        .and_then(|p| p.as_str())
        .ok_or_else(|| anyhow::anyhow!("No storage_path for module {}", module_id))?;

    // Download from Supabase Storage
    let storage_url = format!("{}/storage/v1/object/wasm-modules/{}", supabase_url, storage_path);
    
    let wasm_response = client
        .get(&storage_url)
        .header("apikey", service_role_key)
        .header("Authorization", &format!("Bearer {}", service_role_key))
        .send()
        .await?;

    if !wasm_response.status().is_success() {
        anyhow::bail!("Failed to download module from storage: {}", wasm_response.status());
    }

    let wasm_bytes = wasm_response.bytes().await?;
    
    // Save to cache
    tokio::fs::write(&cache_path, &wasm_bytes).await?;
    
    println!("Downloaded and cached module: {}", module_id);
    Ok(cache_path)
}

async fn execute_module(
    engine: &Engine,
    linker: &Linker<()>,
    module_id: &str,
    message: Message,
    supabase_url: &str,
    service_role_key: &str,
) -> anyhow::Result<Message> {
    // Download module from Supabase Storage (or use cached version)
    let path = download_module(module_id, supabase_url, service_role_key).await?;

    let module = Module::from_file(engine, &path)?;

    // Create store and instantiate via linker
    let mut store = Store::new(engine, ());
    let instance = linker.instantiate_async(&mut store, &module).await?;

    // Get exports
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow::anyhow!("no memory export"))?;
    let alloc = instance.get_typed_func::<i32, i32>(&mut store, "alloc")?;
    let dealloc = instance.get_typed_func::<(i32, i32), ()>(&mut store, "dealloc")?;
    let handle = instance.get_typed_func::<(i32, i32), i64>(&mut store, "handle")?;

    // Serialize input message to JSON
    let input = serde_json::to_vec(&message)?;
    let input_len = input.len() as i32;

    // Allocate memory in guest and write input
    let ptr = alloc.call_async(&mut store, input_len).await?;
    memory.write(&mut store, ptr as usize, &input)?;

    // Call handle function
    let packed = handle.call_async(&mut store, (ptr, input_len)).await?;
    let slice = WasmSlice::unpack(packed);

    // Read response from guest memory
    let mut buf = vec![0u8; slice.len as usize];
    memory.read(&mut store, slice.ptr as usize, &mut buf)?;

    // Cleanup guest memory
    dealloc
        .call_async(&mut store, (slice.ptr, slice.len))
        .await?;
    dealloc.call_async(&mut store, (ptr, input_len)).await?;

    // Parse response message
    let response: Message = serde_json::from_slice(&buf)?;
    Ok(response)
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    let supabase_url = std::env::var("SUPABASE_URL")
        .expect("SUPABASE_URL must be set");
    let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
        .expect("SUPABASE_SERVICE_ROLE_KEY must be set");
    
    let worker_id = get_worker_id();
    println!("Worker starting with ID: {}", worker_id);

    // Initialize Wasmtime engine
    let engine = Engine::default();

    // Create linker with host functions
    let mut linker = Linker::new(&engine);

    // Host function: log
    linker
        .func_wrap(
            "env",
            "log",
            |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
                let mem = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .expect("guest has no memory export");

                let mut buf = vec![0u8; len as usize];
                mem.read(&mut caller, ptr as usize, &mut buf)
                    .expect("memory.read failed");

                println!("[guest log] {}", String::from_utf8_lossy(&buf));
            },
        )
        .expect("Failed to define log host function");

    // Host function: http_request (async)
    linker
        .func_new_async(
            "env",
            "http_request",
            FuncType::new(&engine, [ValType::I32, ValType::I32], [ValType::I64]),
            |mut caller, params, results| {
                Box::new(async move {
                    let ptr = params[0].i32().unwrap();
                    let len = params[1].i32().unwrap();

                    let mem = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .expect("no memory");

                    // Read job JSON from guest
                    let mut buf = vec![0u8; len as usize];
                    mem.read(&mut caller, ptr as usize, &mut buf).expect("read");

                    // Parse { url, method?, headers?, body? }
                    let job: serde_json::Value = serde_json::from_slice(&buf).unwrap_or_default();
                    let url = job
                        .get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("https://example.com");
                    let method = job.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
                    let headers = job.get("headers").cloned().unwrap_or(serde_json::json!({}));
                    let body = job.get("body");

                    // Do HTTP via reqwest
                    let client = reqwest::Client::new();
                    let mut req =
                        client.request(method.parse().unwrap_or(reqwest::Method::GET), url);

                    if let Some(hs) = headers.as_object() {
                        for (k, v) in hs {
                            if let Some(val) = v.as_str() {
                                req = req.header(k, val);
                            }
                        }
                    }
                    if let Some(b) = body {
                        if b.is_string() {
                            req = req.body(b.as_str().unwrap().to_owned());
                        } else {
                            req = req.json(b);
                        }
                    }

                    let resp = req.send().await;

                    let resp_bytes = match resp {
                        Ok(r) => {
                            let status = r.status().as_u16();

                            // Try to decide if it's JSON from headers
                            let is_json_hdr = r
                                .headers()
                                .get(reqwest::header::CONTENT_TYPE)
                                .and_then(|h| h.to_str().ok())
                                .map(|ct| ct.to_ascii_lowercase().contains("application/json"))
                                .unwrap_or(false);

                            if is_json_hdr {
                                // Return the JSON BODY *directly* so the guest can parse into its T
                                match r.json::<serde_json::Value>().await {
                                    Ok(v) => serde_json::to_vec(&v).unwrap(),
                                    Err(e) => serde_json::to_vec(&serde_json::json!({
                                        "_error": format!("bad json body: {e}"),
                                        "status": status
                                    }))
                                    .unwrap(),
                                }
                            } else {
                                // Not advertised as JSON -> read text, but still try to parse as JSON first
                                let text = r.text().await.unwrap_or_default();
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                                    // Looks like JSON anyway: return body JSON directly
                                    serde_json::to_vec(&v).unwrap()
                                } else {
                                    // Plain text fallback: keep wrapper
                                    serde_json::to_vec(
                                        &serde_json::json!({ "status": status, "body": text }),
                                    )
                                    .unwrap()
                                }
                            }
                        }
                        Err(e) => {
                            // Network or request error
                            serde_json::to_vec(&serde_json::json!({
                                "_error": format!("host http error: {e}")
                            }))
                            .unwrap()
                        }
                    };

                    // Ask guest to alloc a buffer for the response
                    let alloc = caller
                        .get_export("alloc")
                        .and_then(|e| e.into_func())
                        .expect("no alloc");
                    let alloc = alloc.typed::<i32, i32>(&caller).expect("typed alloc");
                    let out_ptr = alloc
                        .call_async(&mut caller, resp_bytes.len() as i32)
                        .await
                        .expect("alloc call failed");

                    // Write response into guest memory
                    mem.write(&mut caller, out_ptr as usize, &resp_bytes)
                        .expect("write");

                    // Return packed (ptr,len)
                    let packed = WasmSlice {
                        ptr: out_ptr,
                        len: resp_bytes.len() as i32,
                    }
                    .pack();
                    results[0] = Val::I64(packed);
                    Ok(())
                })
            },
        )
        .expect("Failed to define http_request host function");

    println!("Wasmtime engine and linker initialized");

    // Connect to Redis for idempotency tracking
    println!("Connecting to Redis at {}...", REDIS_URL);
    let redis_client = redis::Client::open(REDIS_URL).expect("Failed to create Redis client");
    let mut redis_conn = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .expect("Failed to connect to Redis");

    println!("Connected to Redis successfully");

    // Retry tracker
    let mut retry_tracker = RetryTracker::new();

    // Consumer for jobs
    let jobs_consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("group.id", GROUP_ID)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create jobs consumer");

    jobs_consumer
        .subscribe(&[JOBS_TOPIC])
        .expect("Failed to subscribe to jobs topic");

    // Producer for results and DLQ
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", BROKERS)
        .set("message.timeout.ms", "10000")
        .set("acks", "all")
        .set("enable.idempotence", "true")
        .create()
        .expect("Failed to create producer");

    println!("Worker running. Waiting for jobs...");

    loop {
        let msg = match jobs_consumer.recv().await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Kafka error: {e}");
                continue;
            }
        };

        let payload = match msg.payload_view::<str>() {
            Some(Ok(s)) => s,
            _ => {
                eprintln!("Bad payload, skipping");
                let _ = jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);
                continue;
            }
        };

        let job: Job = match serde_json::from_str(payload) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Bad JSON: {e}");
                let _ = jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);
                continue;
            }
        };

        // Check Redis for idempotency (simple EXISTS check)
        let exists: bool = redis_conn.exists(&job.job_id).await.unwrap_or(false);
        if exists {
            println!("Job {} already processed, skipping", job.job_id);
            let _ = jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);
            continue;
        }

        // Execute module
        let result = execute_module(
            &engine,
            &linker,
            &job.module_id,
            job.message.clone(),
            &supabase_url,
            &service_role_key,
        )
        .await;

        match result {
            Ok(response_msg) => {
                // Success: serialize response Message and publish
                let response_bytes = serde_json::to_vec(&response_msg).unwrap_or_default();

                let result_msg = ResultMsg {
                    job_id: job.job_id.clone(),
                    worker_id: worker_id.clone(),
                    ok: true,
                    output: Some(response_bytes),
                    error: None,
                };

                // Publish result
                if let Err(e) = publish_result(&producer, &result_msg).await {
                    eprintln!("Failed to publish result: {e}");
                    continue; // Don't commit, will retry
                }

                // Mark as processed in Redis with TTL
                let _: () = redis_conn
                    .set_ex(&job.job_id, "completed", REDIS_TTL_SECONDS)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to set Redis key: {}", e);
                    });

                // Clear retry tracker
                retry_tracker.remove(&job.job_id);

                // Commit offset
                let _ = jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);

                println!(
                    "Processed job {} with module {} -> {}",
                    job.job_id, job.module_id, response_msg.type_name
                );
            }
            Err(e) => {
                // Failure: check retry count
                let attempts = retry_tracker.increment(&job.job_id);

                eprintln!(
                    "Job {} (module {}) failed (attempt {}/{}): {}",
                    job.job_id, job.module_id, attempts, MAX_RETRIES, e
                );

                if attempts >= MAX_RETRIES {
                    // Max retries reached: send to DLQ
                    let dlq_msg = DLQMessage {
                        job_id: job.job_id.clone(),
                        job: job.clone(),
                        attempts,
                        last_error: e.to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };

                    if let Err(e) = publish_to_dlq(&producer, &dlq_msg).await {
                        eprintln!("Failed to publish to DLQ: {e}");
                        continue; // Don't commit, will retry
                    }

                    // Mark as failed in Redis with TTL
                    let _: () = redis_conn
                        .set_ex(&job.job_id, "failed", REDIS_TTL_SECONDS)
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to set Redis key: {}", e);
                        });

                    // Clear retry tracker
                    retry_tracker.remove(&job.job_id);

                    // Commit offset (unblock partition)
                    let _ =
                        jobs_consumer.commit_message(&msg, rdkafka::consumer::CommitMode::Async);

                    println!("Job {} sent to DLQ after {} attempts", job.job_id, attempts);
                } else {
                    // Don't commit: will retry on next poll
                    println!("Job {} will retry (attempt {})", job.job_id, attempts);
                }
            }
        }
    }
}

async fn publish_result(
    producer: &FutureProducer,
    result: &ResultMsg,
) -> Result<(), rdkafka::error::KafkaError> {
    let payload = serde_json::to_string(result).unwrap();
    producer
        .send(
            FutureRecord::to(RESULTS_TOPIC)
                .key(&result.job_id)
                .payload(&payload),
            Duration::from_secs(0),
        )
        .await
        .map(|_| ())
        .map_err(|(e, _)| e)
}

async fn publish_to_dlq(
    producer: &FutureProducer,
    dlq_msg: &DLQMessage,
) -> Result<(), rdkafka::error::KafkaError> {
    let payload = serde_json::to_string(dlq_msg).unwrap();
    producer
        .send(
            FutureRecord::to(DLQ_TOPIC)
                .key(&dlq_msg.job_id)
                .payload(&payload),
            Duration::from_secs(0),
        )
        .await
        .map(|_| ())
        .map_err(|(e, _)| e)
}