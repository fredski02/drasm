use postgrest::Postgrest;
use std::sync::Arc;

#[derive(Clone)]
pub struct SupabaseClient {
    pub url: String,
    pub anon_key: String,
    pub service_role_key: String,
}

impl SupabaseClient {
    pub fn new(url: String, anon_key: String, service_role_key: String) -> Self {
        Self {
            url,
            anon_key,
            service_role_key,
        }
    }

    /// Get PostgREST client with user's JWT token
    pub fn rest_with_token(&self, token: &str) -> Postgrest {
        Postgrest::new(format!("{}/rest/v1", self.url))
            .insert_header("apikey", &self.anon_key)
            .insert_header("Authorization", format!("Bearer {}", token))
    }

    /// Get PostgREST client with service role key (bypasses RLS)
    pub fn rest_service(&self) -> Postgrest {
        Postgrest::new(format!("{}/rest/v1", self.url))
            .insert_header("apikey", &self.service_role_key)
            .insert_header("Authorization", format!("Bearer {}", self.service_role_key))
    }

    /// Get Supabase Auth URL
    pub fn auth_url(&self) -> String {
        format!("{}/auth/v1", self.url)
    }

    /// Get Supabase Storage URL
    pub fn storage_url(&self) -> String {
        format!("{}/storage/v1", self.url)
    }
}

pub type SharedSupabaseClient = Arc<SupabaseClient>;