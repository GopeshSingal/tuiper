use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthAction {
    Login,
    Signup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPublic {
    pub id: i64,
    pub username: String,
    pub elo: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub action: AuthAction,
    pub token: String,
    pub account: AccountPublic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}
