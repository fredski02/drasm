pub mod auth;
pub mod jobs;
pub mod modules;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

use crate::{
    kafka::SharedKafkaProducer,
    middleware::auth_middleware,
    supabase::SharedSupabaseClient,
};

pub fn create_router(
    supabase: SharedSupabaseClient,
    kafka: SharedKafkaProducer,
) -> Router {
    // All routes with shared state
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Auth endpoints (no auth required)
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        // Protected endpoints with middleware
        .route("/modules", post(modules::upload_module).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/modules", get(modules::list_modules).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/jobs", post(jobs::submit_job).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/jobs/:job_id", get(jobs::get_job).layer(axum::middleware::from_fn(auth_middleware)))
        .route("/jobs", get(jobs::list_jobs).layer(axum::middleware::from_fn(auth_middleware)))
        .with_state((supabase, kafka))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}