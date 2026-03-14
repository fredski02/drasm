use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{supabase::SharedSupabaseClient, kafka::SharedKafkaProducer};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: serde_json::Value,
}

/// Sign up a new user
pub async fn signup(
    State((supabase, _kafka)): State<(SharedSupabaseClient, SharedKafkaProducer)>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let client = reqwest::Client::new();
    let auth_url = format!("{}/signup", supabase.auth_url());

    let response = client
        .post(&auth_url)
        .header("apikey", &supabase.anon_key)
        .json(&serde_json::json!({
            "email": req.email,
            "password": req.password
        }))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("Supabase signup error: {}", error_text);
        return Err(StatusCode::BAD_REQUEST);
    }

    let auth_data: serde_json::Value = response
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let access_token = auth_data
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let user = auth_data
        .get("user")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    Ok(Json(AuthResponse { access_token, user }))
}

/// Login an existing user
pub async fn login(
    State((supabase, _kafka)): State<(SharedSupabaseClient, SharedKafkaProducer)>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let client = reqwest::Client::new();
    let auth_url = format!("{}/token?grant_type=password", supabase.auth_url());

    let response = client
        .post(&auth_url)
        .header("apikey", &supabase.anon_key)
        .json(&serde_json::json!({
            "email": req.email,
            "password": req.password
        }))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("Supabase login error: {}", error_text);
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth_data: serde_json::Value = response
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let access_token = auth_data
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let user = auth_data
        .get("user")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    Ok(Json(AuthResponse { access_token, user }))
}