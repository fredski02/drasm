use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub token: String,
}

/// Simple JWT validation middleware
/// For MVP, we trust the JWT signature from Supabase
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = auth_header.trim_start_matches("Bearer ").to_string();

    // Decode JWT to get user_id (without full validation for MVP)
    // In production, validate the signature with Supabase JWT secret
    let user_id = extract_user_id_from_jwt(&token).ok_or(StatusCode::UNAUTHORIZED)?;

    // Store auth info in request extensions for axum::Extension
    request.extensions_mut().insert(AuthUser { 
        user_id, 
        token 
    });

    Ok(next.run(request).await)
}

/// Extract user_id from JWT payload (MVP: no signature validation)
/// In production, use jsonwebtoken crate to validate signature
fn extract_user_id_from_jwt(token: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload = parts[1];
    let decoded = base64_decode(payload)?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;

    json.get("sub")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Base64 URL-safe decode (JWT uses base64url encoding)
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.decode(input).ok()
}