use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker_api::Message;

use crate::{
    kafka::{publish_job, SharedKafkaProducer},
    middleware::AuthUser,
    supabase::SharedSupabaseClient,
};
use common::Job;

/// API DTO for submitting jobs - accepts JSON string for payload
#[derive(Deserialize)]
pub struct SubmitJobRequest {
    pub module_id: String,
    pub message: MessageDto,
}

/// API DTO for Message - accepts payload as JSON string instead of bytes
#[derive(Deserialize)]
pub struct MessageDto {
    pub type_name: String,
    pub payload: String, // JSON string that will be converted to bytes
}

#[derive(Serialize)]
pub struct SubmitJobResponse {
    pub job_id: String,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct JobRecord {
    pub job_id: String,
    pub user_id: String,
    pub module_id: String,
    pub status: String,
    pub input_message: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub worker_id: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Submit a job for execution
pub async fn submit_job(
    State((supabase, kafka)): State<(SharedSupabaseClient, SharedKafkaProducer)>,
    auth: axum::Extension<AuthUser>,
    Json(req): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, StatusCode> {
    let auth = auth.0;

    let job_id = Uuid::new_v4().to_string();

    // Convert the MessageDto (string payload) to Message (bytes payload)
    let message = Message {
        type_name: req.message.type_name,
        payload: req.message.payload.as_bytes().to_vec(),
    };

    // Create job record in database
    let job_record = serde_json::json!({
        "job_id": job_id,
        "user_id": auth.user_id,
        "module_id": req.module_id,
        "status": "pending",
        "input_message": serde_json::to_value(&message).unwrap(),
    });

    let insert_response = supabase
        .rest_with_token(&auth.token)
        .from("jobs")
        .insert(job_record.to_string())
        .execute()
        .await
        .map_err(|e| {
            eprintln!("DB insert error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !insert_response.status().is_success() {
        let error_text = insert_response.text().await.unwrap_or_default();
        eprintln!("DB insert failed: {}", error_text);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Publish job to Kafka
    let job = Job {
        job_id: job_id.clone(),
        module_id: req.module_id,
        message,
    };

    publish_job(&kafka, &job).await.map_err(|e| {
        eprintln!("Kafka publish error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(SubmitJobResponse {
        job_id,
        status: "pending".to_string(),
    }))
}

/// Get job status and result
pub async fn get_job(
    State((supabase, _kafka)): State<(SharedSupabaseClient, SharedKafkaProducer)>,
    auth: axum::Extension<AuthUser>,
    Path(job_id): Path<String>,
) -> Result<Json<JobRecord>, StatusCode> {
    let auth = &auth.0;

    let response = supabase
        .rest_with_token(&auth.token)
        .from("jobs")
        .select("*")
        .eq("job_id", &job_id)
        .single()
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if response.status() != 200 {
        return Err(StatusCode::NOT_FOUND);
    }

    let body = response.text().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let job: JobRecord = serde_json::from_str(&body)
        .map_err(|e| {
            eprintln!("Failed to parse job response: {}", e);
            eprintln!("Response body: {}", body);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(job))
}

/// List user's jobs
pub async fn list_jobs(
    State((supabase, _kafka)): State<(SharedSupabaseClient, SharedKafkaProducer)>,
    auth: axum::Extension<AuthUser>,
) -> Result<Json<Vec<JobRecord>>, StatusCode> {
    let auth = &auth.0;

    let response = supabase
        .rest_with_token(&auth.token)
        .from("jobs")
        .select("*")
        .order("created_at.desc")
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if response.status() != 200 {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let body = response.text().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let jobs: Vec<JobRecord> = serde_json::from_str(&body)
        .map_err(|e| {
            eprintln!("Failed to parse jobs response: {}", e);
            eprintln!("Response body: {}", body);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(jobs))
}