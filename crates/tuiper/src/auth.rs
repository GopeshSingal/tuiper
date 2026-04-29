use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use protocols::{AccountPublic, ApiError, AuthRequest, AuthResponse};
use serde::Deserialize;

pub fn auth_url_for_ws_url(ws_url: &str) -> Result<String, String> {
    let ws_url = ws_url.trim();
    let (scheme, rest) = if let Some(r) = ws_url.strip_prefix("wss://") {
        ("https", r)
    } else if let Some(r) = ws_url.strip_prefix("ws://") {
        ("http", r)
    } else {
        return Err("WS_URL must start with ws:// or wss://".to_string());
    };

    let authority_end = rest.find('/').unwrap_or(rest.len());
    let authority = rest[..authority_end].trim();
    if authority.is_empty() {
        return Err("WS_URL is missing host".to_string());
    }

    Ok(format!("{scheme}://{authority}/auth"))
}

pub fn account_elo_url_for_ws_url(ws_url: &str, username: &str) -> Result<String, String> {
    let ws_url = ws_url.trim();
    let username = username.trim();
    if username.is_empty() {
        return Err("username was expected".to_string());
    }

    let (scheme, rest) = if let Some(r) = ws_url.strip_prefix("wss://") {
        ("https", r)
    } else if let Some(r) = ws_url.strip_prefix("ws://") {
        ("http", r)
    } else {
        return Err("WS_URL must start with ws:// or wss://".to_string());
    };

    let authority_end = rest.find('/').unwrap_or(rest.len());
    let authority = rest[..authority_end].trim();
    if authority.is_empty() {
        return Err("WS_URL is missing host".to_string());
    }

    let encoded_user = utf8_percent_encode(username, NON_ALPHANUMERIC).to_string();
    Ok(format!("{scheme}://{authority}/accounts/{encoded_user}/elo"))
}

pub fn ws_url_with_token(ws_base: &str, token: &str) -> String {
    let encoded = utf8_percent_encode(token, NON_ALPHANUMERIC).to_string();
    let sep = if ws_base.contains('?') { '&' } else { '?' };
    format!("{ws_base}{sep}token={encoded}")
}

pub fn login(
    auth_url: &str,
    username: &str,
    password: &str,
) -> Result<(String, AccountPublic), String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(auth_url)
        .json(&AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
        })
        .send()
        .map_err(|e| format!("auth request failed: {e}"))?;

    let status = response.status();
    if status.is_success() {
        let parsed = response
            .json::<AuthResponse>()
            .map_err(|e| format!("invalid auth response: {e}"))?;
        return Ok((parsed.token, parsed.account));
    }

    match response.json::<ApiError>() {
        Ok(api_error) => Err(api_error.error),
        Err(_) => Err(format!("auth failed ({status})")),
    }
}

#[derive(Debug, Deserialize)]
struct AccountEloResponse {
    pub username: String,
    pub elo: i64,
}

pub fn fetch_account_elo(elo_url: &str) -> Result<(String, i64), String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(elo_url)
        .send()
        .map_err(|e| format!("account elo request failed: {e}"))?;

    let status = response.status();
    if status.is_success() {
        let parsed = response
            .json::<AccountEloResponse>()
            .map_err(|e| format!("invalid account elo response: {e}"))?;
        return Ok((parsed.username, parsed.elo));
    }

    match response.json::<ApiError>() {
        Ok(api_error) => Err(api_error.error),
        Err(_) => Err(format!("account elo fetch failed ({status})")),
    }
}