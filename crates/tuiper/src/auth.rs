use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use protocols::{AccountPublic, ApiError, AuthRequest, AuthResponse, LeaderboardResponse, RaceHistoryResponse};
use serde::Deserialize;

fn http_base_from_ws_url(ws_url: &str) -> Result<String, String> {
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

    Ok(format!("{scheme}://{authority}"))
}

pub fn auth_url_for_ws_url(ws_url: &str) -> Result<String, String> {
    let base = http_base_from_ws_url(ws_url)?;
    Ok(format!("{base}/auth"))
}

pub fn account_elo_url_for_ws_url(ws_url: &str, username: &str) -> Result<String, String> {
    let username = username.trim();
    if username.is_empty() {
        return Err("username was expected".to_string());
    }

    let base = http_base_from_ws_url(ws_url)?;
    let encoded_user = utf8_percent_encode(username, NON_ALPHANUMERIC).to_string();
    Ok(format!("{base}/accounts/{encoded_user}/elo"))
}

pub fn leaderboard_url_for_ws_url(ws_url: &str) -> Result<String, String> {
    let base = http_base_from_ws_url(ws_url)?;
    Ok(format!("{base}/leaderboard"))
}

pub fn race_history_url_for_ws_url(ws_url: &str, username: &str) -> Result<String, String> {
    let username = username.trim();
    if username.is_empty() {
        return Err("username was expected".to_string());
    }

    let base = http_base_from_ws_url(ws_url)?;
    let encoded_user = utf8_percent_encode(username, NON_ALPHANUMERIC).to_string();
    Ok(format!("{base}/accounts/{encoded_user}/races"))
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

pub fn fetch_leaderboard(leaderboard_url: &str) -> Result<LeaderboardResponse, String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(leaderboard_url)
        .send()
        .map_err(|e| format!("leaderboard request failed: {e}"))?;

    let status = response.status();
    if status.is_success() {
        return response
            .json::<LeaderboardResponse>()
            .map_err(|e| format!("invalid leaderboard response: {e}"));
    }

    match response.json::<ApiError>() {
        Ok(api_error) => Err(api_error.error),
        Err(_) => Err(format!("leaderboard fetch failed ({status})")),
    }
}

pub fn fetch_race_history(race_history_url: &str) -> Result<RaceHistoryResponse, String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(race_history_url)
        .send()
        .map_err(|e| format!("race history request failed: {e}"))?;

    let status = response.status();
    if status.is_success() {
        return response
            .json::<RaceHistoryResponse>()
            .map_err(|e| format!("invalid race history response: {e}"));
    }

    match response.json::<ApiError>() {
        Ok(api_error) => Err(api_error.error),
        Err(_) => Err(format!("race history fetch failed ({status})")),
    }
}
