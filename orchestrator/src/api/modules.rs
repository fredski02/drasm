use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{kafka::SharedKafkaProducer, middleware::AuthUser, supabase::SharedSupabaseClient};

#[derive(Serialize)]
pub struct ModuleResponse {
    pub id: String,
    pub name: String,
    pub hash: String,
    pub size_bytes: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub storage_path: String,
    pub hash: String,
    pub size_bytes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Upload WASM module
pub async fn upload_module(
    State((supabase, _kafka)): State<(SharedSupabaseClient, SharedKafkaProducer)>,
    auth: axum::Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<ModuleResponse>, StatusCode> {
    let auth = auth.0;

    let mut wasm_bytes: Option<Vec<u8>> = None;
    let mut module_name: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                wasm_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .to_vec(),
                );
            }
            "name" => {
                module_name = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            _ => {}
        }
    }

    let wasm_bytes = wasm_bytes.ok_or(StatusCode::BAD_REQUEST)?;
    let name = module_name.unwrap_or_else(|| "unnamed".to_string());

    // Calculate hash
    let mut hasher = Sha256::new();
    hasher.update(&wasm_bytes);
    let hash = format!("{:x}", hasher.finalize());

    let module_id = Uuid::new_v4().to_string();
    let storage_path = format!("{}/{}.wasm", auth.user_id, module_id);

    // Upload to Supabase Storage
    let client = reqwest::Client::new();
    let storage_url = format!(
        "{}/object/wasm-modules/{}",
        supabase.storage_url(),
        storage_path
    );

    let upload_response = client
        .post(&storage_url)
        .header("apikey", &supabase.anon_key)
        .header("Authorization", &format!("Bearer {}", auth.token))
        .header("Content-Type", "application/wasm")
        .body(wasm_bytes.clone())
        .send()
        .await
        .map_err(|e| {
            eprintln!("Storage upload error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !upload_response.status().is_success() {
        let error_text = upload_response.text().await.unwrap_or_default();
        eprintln!("Storage upload failed: {}", error_text);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Insert module record in database
    let module = Module {
        id: module_id.clone(),
        user_id: auth.user_id.clone(),
        name: name.clone(),
        storage_path: storage_path.clone(),
        hash: hash.clone(),
        size_bytes: wasm_bytes.len() as i64,
        created_at: None,
        updated_at: None,
    };

    let insert_response = supabase
        .rest_with_token(&auth.token)
        .from("modules")
        .insert(serde_json::to_string(&module).unwrap())
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

    Ok(Json(ModuleResponse {
        id: module_id,
        name,
        hash,
        size_bytes: wasm_bytes.len() as i64,
    }))
}

/// List user's modules
pub async fn list_modules(
    State((supabase, _kafka)): State<(SharedSupabaseClient, SharedKafkaProducer)>,
    auth: axum::Extension<AuthUser>,
) -> Result<Json<Vec<Module>>, StatusCode> {
    let auth = &auth.0;

    let response = supabase
        .rest_with_token(&auth.token)
        .from("modules")
        .select("*")
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if response.status() != 200 {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let body = response
        .text()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let modules: Vec<Module> =
        serde_json::from_str(&body).map_err(|e| {
            eprintln!("Failed to parse modules response: {}", e);
            eprintln!("Response body: {}", body);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(modules))
}