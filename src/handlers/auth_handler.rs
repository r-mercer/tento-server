use actix_web::{get, post, web, HttpResponse};
use chrono::{Duration, Utc};
use octocrab::Octocrab;
use secrecy::ExposeSecret as _;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    app_state::AppState,
    errors::AppError,
    models::domain::{hash_token, user::User, RefreshToken},
};

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    code: String,
    #[serde(default)]
    redirect_uri: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub full_name: Option<String>,
}

#[get("/auth/github/callback")]
pub async fn auth_github_callback(
    state: web::Data<Arc<AppState>>,
    web::Query(params): web::Query<CallbackParams>,
) -> Result<HttpResponse, AppError> {
    log::info!("=== GitHub OAuth Callback Started ===");
    log::info!("Code: {}", params.code);
    log::info!("Redirect URI: {:?}", params.redirect_uri);

    let client_id = &state.config.gh_client_id;
    let client_secret = state.config.gh_client_secret.expose_secret();

    log::info!("Client ID: {}", client_id);
    log::info!("Client Secret length: {}", client_secret.len());

    let client = reqwest::Client::new();

    let redirect_uri = params
        .redirect_uri
        .as_deref()
        .unwrap_or("http://localhost:5173/auth/callback");

    log::info!("Using redirect_uri: {}", redirect_uri);

    let request_body = serde_json::json!({
        "code": params.code,
        "client_id": client_id,
        "client_secret": client_secret,
        "redirect_uri": redirect_uri,
    });

    log::info!(
        "GitHub request body (JSON): {}",
        serde_json::to_string_pretty(&request_body).unwrap()
    );

    let token_response = client
        .post("https://github.com/login/oauth/access_token")
        .header("accept", "application/json")
        .form(&[
            ("code", params.code.as_str()),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to send request to GitHub: {}", e);
            AppError::InternalError(format!("Failed to exchange OAuth code: {}", e))
        })?;

    let status = token_response.status();
    let response_text = token_response
        .text()
        .await
        .unwrap_or_else(|_| "Could not read response body".to_string());

    log::info!("GitHub token exchange response status: {}", status);
    log::info!("GitHub token exchange response body: {}", response_text);

    let oauth = serde_json::from_str::<serde_json::Value>(&response_text).map_err(|e| {
        log::error!("Failed to parse GitHub response: {}", e);
        AppError::InternalError(format!(
            "Failed to parse token response: {} | Response: {}",
            e, response_text
        ))
    })?;

    let oauth = oauth
        .as_object()
        .ok_or_else(|| AppError::InternalError("Invalid OAuth response format".to_string()))?;

    if let Some(error) = oauth.get("error").and_then(|v| v.as_str()) {
        let error_description = oauth
            .get("error_description")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        log::error!("GitHub OAuth error: {} - {}", error, error_description);
        return Err(AppError::InternalError(format!(
            "GitHub OAuth error: {} - {}",
            error, error_description
        )));
    }

    let access_token = oauth
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            log::error!("No access_token in GitHub response. Full response: {:?}", oauth);
            AppError::InternalError(
                "No access_token in GitHub response. Possible issues: invalid code, or GitHub credentials misconfigured".to_string()
            )
        })?;

    let gh_client = Octocrab::builder()
        .user_access_token(access_token.to_string())
        .build()
        .map_err(|e| AppError::InternalError(format!("Failed to build GitHub client: {}", e)))?;

    let gh_user = gh_client
        .current()
        .user()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to fetch GitHub user: {}", e)))?;

    let github_id = gh_user.id.to_string();
    let username = gh_user.login.clone();
    let email = gh_user
        .email
        .clone()
        .unwrap_or_else(|| format!("{}@users.noreply.github.com", gh_user.login));

    let user = User::from_github(
        github_id,
        username.clone(),
        email.clone(),
        gh_user.name.clone(),
    );

    let saved_user = state.user_service.upsert_oauth_user(user).await?;

    let token = state.jwt_service.create_token(&saved_user)?;
    let subject_id = saved_user
        .id
        .as_ref()
        .map(|oid| oid.to_hex())
        .unwrap_or_else(|| saved_user.username.clone());

    let refresh_token_str = state.jwt_service.create_refresh_token(&subject_id)?;

    let token_hash = hash_token(&refresh_token_str);
    let expires_at = Utc::now() + Duration::hours(168);
    let refresh_token_record = RefreshToken::new(subject_id.clone(), token_hash, expires_at);
    state
        .refresh_token_repository
        .create(refresh_token_record)
        .await?;

    log::info!("Created refresh token for user: {}", subject_id);

    let full_name = if !saved_user.first_name.is_empty() || !saved_user.last_name.is_empty() {
        Some(format!("{} {}", saved_user.first_name, saved_user.last_name).trim().to_string())
    } else {
        None
    };

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        refresh_token: refresh_token_str,
        id: subject_id,
        username: saved_user.username,
        email: saved_user.email,
        role: format!("{:?}", saved_user.role).to_lowercase(),
        full_name,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub refresh_token: String,
}

#[post("/auth/refresh")]
pub async fn refresh_token(
    state: web::Data<Arc<AppState>>,
    request: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse, AppError> {
    state
        .jwt_service
        .validate_refresh_token(&request.refresh_token)?;

    let token_hash = hash_token(&request.refresh_token);

    let stored_token = state
        .refresh_token_repository
        .find_by_token_hash(&token_hash)
        .await?
        .ok_or_else(|| {
            AppError::Unauthorized("Refresh token not found or has been revoked".to_string())
        })?;

    if !stored_token.is_valid() {
        return Err(AppError::Unauthorized(
            "Refresh token has expired or been revoked".to_string(),
        ));
    }

    let user = state
        .user_service
        .get_user_for_token(&stored_token.user_id)
        .await
        .map_err(|_| {
            AppError::Unauthorized("User associated with refresh token not found".to_string())
        })?;

    state
        .refresh_token_repository
        .revoke_by_token_hash(&token_hash)
        .await?;

    log::info!(
        "Revoked old refresh token for user: {}",
        stored_token.user_id
    );

    let subject_id = user
        .id
        .as_ref()
        .map(|oid| oid.to_hex())
        .unwrap_or_else(|| user.username.clone());

    let new_token = state.jwt_service.create_token(&user)?;
    let new_refresh_token_str = state.jwt_service.create_refresh_token(&subject_id)?;

    let new_token_hash = hash_token(&new_refresh_token_str);
    let expires_at = Utc::now() + Duration::hours(168);
    let new_refresh_token_record = RefreshToken::new(subject_id.clone(), new_token_hash, expires_at);
    state
        .refresh_token_repository
        .create(new_refresh_token_record)
        .await?;

    log::info!("Token refreshed successfully for user: {}", subject_id);

    Ok(HttpResponse::Ok().json(RefreshTokenResponse {
        token: new_token,
        refresh_token: new_refresh_token_str,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[post("/auth/logout")]
pub async fn logout(
    state: web::Data<Arc<AppState>>,
    request: web::Json<LogoutRequest>,
) -> Result<HttpResponse, AppError> {
    let token_hash = hash_token(&request.refresh_token);

    match state
        .refresh_token_repository
        .revoke_by_token_hash(&token_hash)
        .await
    {
        Ok(()) => {
            log::info!("Successfully revoked refresh token on logout");
        }
        Err(AppError::NotFound(_)) => {
            log::info!("Refresh token not found on logout (may have already been revoked)");
        }
        Err(e) => return Err(e),
    }

    Ok(HttpResponse::NoContent().finish())
}
