use actix_web::{get, web, HttpResponse};
use octocrab::Octocrab;
use secrecy::ExposeSecret as _;
use serde::Deserialize;

use crate::{app_state::AppState, errors::AppError};
// use shuttle_runtime::configtore;

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    code: String,
}

#[get("/auth/github/callback")]
pub async fn auth_github_callback(
    state: web::Data<AppState>,
    web::Query(params): web::Query<CallbackParams>,
) -> Result<HttpResponse, AppError> {
    let client_id = &state.config.gh_client_id;
    let client_secret = &state.config.gh_client_secret;

    let oauth_client = octocrab::Octocrab::builder()
        .base_uri("https://github.com")
        .unwrap()
        .add_header("accept".parse().unwrap(), "application/json".to_string())
        .build()
        .unwrap();

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
        .unwrap();

    let oauth = serde_json::from_value::<octocrab::auth::OAuth>(oauth.clone())
        .unwrap_or_else(|_| panic!("couldn't parse OAuth credentials from {oauth:?}"));

    let client = Octocrab::builder()
        .user_access_token(oauth.access_token.expose_secret())
        .build()
        .unwrap();

    let user = client.current().user().await.unwrap();

    Ok(HttpResponse::Ok().json(user.login))
}
