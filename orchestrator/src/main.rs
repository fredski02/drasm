mod api;
mod kafka;
mod middleware;
mod supabase;

use axum::Router;
use common::{ResultMsg, RESULTS_TOPIC};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    message::Message as KafkaMessage,
    ClientConfig,
};
use std::sync::Arc;
use tokio::task;
use tower_http::cors::CorsLayer;
use worker_api::Message;

use crate::{
    kafka::create_kafka_producer,
    supabase::SupabaseClient,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    let supabase_url = std::env::var("SUPABASE_URL")?;
    let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY")?;
    let supabase_service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")?;
    let kafka_brokers = std::env::var("KAFKA_BROKERS")?;
    let _redis_url = std::env::var("REDIS_URL").ok();
    let orchestrator_host = std::env::var("ORCHESTRATOR_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let orchestrator_port = std::env::var("ORCHESTRATOR_PORT").unwrap_or_else(|_| "3000".to_string());

    // Create shared clients
    let supabase = Arc::new(SupabaseClient::new(
        supabase_url,
        supabase_anon_key,
        supabase_service_role_key,
    ));
    let kafka_producer = Arc::new(create_kafka_producer(&kafka_brokers)?);

    // Create API router
    let app = Router::new()
        .nest("/api", api::create_router(supabase.clone(), kafka_producer.clone()))
        .layer(CorsLayer::permissive());

    // Spawn Kafka results consumer task
    let supabase_clone = supabase.clone();
    task::spawn(async move {
        if let Err(e) = run_results_consumer(kafka_brokers, supabase_clone).await {
            eprintln!("Results consumer error: {}", e);
        }
    });

    // Start HTTP server
    let addr = format!("{}:{}", orchestrator_host, orchestrator_port);
    println!("🚀 Orchestrator listening on http://{}", addr);
    println!("   Health check: http://{}/health", addr);
    println!("   API: http://{}/api", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Background task: consume Kafka results and update job status in Supabase
async fn run_results_consumer(
    kafka_brokers: String,
    supabase: Arc<SupabaseClient>,
) -> anyhow::Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_brokers)
        .set("group.id", "orchestrator-results")
        .set("auto.offset.reset", "latest")
        .create()?;

    consumer.subscribe(&[RESULTS_TOPIC])?;

    println!("📊 Results consumer started");

    loop {
        let msg = match consumer.recv().await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Kafka error: {}", e);
                continue;
            }
        };

        let payload = match msg.payload_view::<str>() {
            Some(Ok(s)) => s,
            _ => continue,
        };

        let result: ResultMsg = match serde_json::from_str(payload) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to parse result: {}", e);
                continue;
            }
        };

        // Update job status in Supabase
        if let Err(e) = update_job_status(&supabase, &result).await {
            eprintln!("Failed to update job status: {}", e);
        }
    }
}

async fn update_job_status(
    supabase: &SupabaseClient,
    result: &ResultMsg,
) -> anyhow::Result<()> {
    let (status, result_json, error) = if result.ok {
        let result_json = if let Some(output_bytes) = &result.output {
            serde_json::from_slice::<Message>(output_bytes)
                .ok()
                .and_then(|msg| serde_json::from_slice::<serde_json::Value>(&msg.payload).ok())
        } else {
            None
        };
        ("completed", result_json, None)
    } else {
        ("failed", None, result.error.clone())
    };

    let update = serde_json::json!({
        "status": status,
        "result": result_json,
        "error": error,
        "worker_id": result.worker_id,
        "completed_at": chrono::Utc::now().to_rfc3339(),
    });

    let response = supabase
        .rest_service()
        .from("jobs")
        .eq("job_id", &result.job_id)
        .update(update.to_string())
        .execute()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Update failed: {}", error_text);
    }

    println!("✓ Updated job {} status: {}", result.job_id, status);

    Ok(())
}