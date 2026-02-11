use actix_web::{get, post, web, HttpResponse};
use octocrab::Octocrab;
use secrecy::ExposeSecret as _;
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, errors::AppError, models::domain::user::User};

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    code: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub username: String,
    pub email: String,
}

#[get("/auth/github/callback")]
pub async fn auth_github_callback(
    state: web::Data<AppState>,
    web::Query(params): web::Query<CallbackParams>,
) -> Result<HttpResponse, AppError> {
    let client_id = &state.config.gh_client_id;
    let client_secret = state.config.gh_client_secret.expose_secret();

    let oauth_client = octocrab::Octocrab::builder()
        .base_uri("https://github.com")
        .map_err(|e| AppError::InternalError(format!("OAuth client error: {}", e)))?
        .add_header(
            "accept"
                .parse()
                .map_err(|e| AppError::InternalError(format!("Invalid header name: {}", e)))?,
            "application/json".to_string(),
        )
        .build()
        .map_err(|e| AppError::InternalError(format!("OAuth client build error: {}", e)))?;

    let oauth = oauth_client
        .post::<_, serde_json::Value>(
            "/login/oauth/access_token",
            Some(&serde_json::json!({
                "code": params.code,
                "client_id": client_id,
                "client_secret": client_secret,
            })),
        )
        .await
        .map_err(|e| AppError::InternalError(format!("OAuth token exchange failed: {}", e)))?;

    let oauth_creds = serde_json::from_value::<octocrab::auth::OAuth>(oauth.clone())
        .map_err(|e| AppError::InternalError(format!("Failed to parse OAuth response: {}", e)))?;

    let client = Octocrab::builder()
        .user_access_token(oauth_creds.access_token.expose_secret())
        .build()
        .map_err(|e| AppError::InternalError(format!("Failed to build GitHub client: {}", e)))?;

    let gh_user = client
        .current()
        .user()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to fetch GitHub user: {}", e)))?;

    // Create User from GitHub data
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

    // Upsert user
    let saved_user = state.user_service.upsert_oauth_user(user).await?;

    // Generate tokens
    let token = state.jwt_service.create_token(&saved_user)?;
    let refresh_token_str = state.jwt_service.create_refresh_token(&saved_user.username)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        refresh_token: refresh_token_str,
        username: saved_user.username,
        email: saved_user.email,
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
    state: web::Data<AppState>,
    request: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate refresh token with detailed error messages
    let refresh_claims = state
        .jwt_service
        .validate_refresh_token(&request.refresh_token)?;
    
    // Get full user object from database
    let user = state
        .user_service
        .get_user_for_token(&refresh_claims.sub)
        .await
        .map_err(|_| AppError::Unauthorized("User associated with refresh token not found".to_string()))?;
    
    // Generate new tokens
    let new_token = state.jwt_service.create_token(&user)?;
    let new_refresh_token = state.jwt_service.create_refresh_token(&refresh_claims.sub)?;
    
    log::info!("Token refreshed successfully for user: {}", refresh_claims.sub);
    
    Ok(HttpResponse::Ok().json(RefreshTokenResponse {
        token: new_token,
        refresh_token: new_refresh_token,
    }))
}
