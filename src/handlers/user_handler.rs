use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse};

use crate::{
    app_state::AppState,
    auth::{require_admin, require_owner_or_admin, AuthenticatedUser},
    errors::AppError,
    models::dto::request::{CreateUserRequestDto, PaginationParams, UpdateUserRequestDto},
};

#[post("/api/users")]
async fn create_user(
    state: web::Data<Arc<AppState>>,
    request: web::Json<CreateUserRequestDto>,
    _auth: AuthenticatedUser, // Require authentication
) -> Result<HttpResponse, AppError> {
    let response = state.user_service.create_user(request.into_inner()).await?;
    Ok(HttpResponse::Created().json(response))
}

#[get("/api/users/{username}")]
async fn get_user(
    state: web::Data<Arc<AppState>>,
    username: web::Path<String>,
    auth: AuthenticatedUser, // Require authentication
) -> Result<HttpResponse, AppError> {
    require_owner_or_admin(&auth.0, &username)?;

    let user = state.user_service.get_user(&username).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/api/users")]
async fn get_all_users(
    state: web::Data<Arc<AppState>>,
    query: web::Query<PaginationParams>,
    auth: AuthenticatedUser, // Require authentication
) -> Result<HttpResponse, AppError> {
    require_admin(&auth.0)?;

    let pagination = query.into_inner();
    let response = state
        .user_service
        .get_all_users_paginated(pagination.offset(), pagination.limit())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::put("/api/users/{username}")]
async fn update_user(
    state: web::Data<Arc<AppState>>,
    username: web::Path<String>,
    request: web::Json<UpdateUserRequestDto>,
    auth: AuthenticatedUser, // Require authentication
) -> Result<HttpResponse, AppError> {
    require_owner_or_admin(&auth.0, &username)?;

    let response = state
        .user_service
        .update_user(&username, request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::delete("/api/users/{username}")]
async fn delete_user(
    state: web::Data<Arc<AppState>>,
    username: web::Path<String>,
    auth: AuthenticatedUser, // Require authentication
) -> Result<HttpResponse, AppError> {
    require_owner_or_admin(&auth.0, &username)?;

    let response = state.user_service.delete_user(&username).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[get("/health")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[get("/health/ready")]
async fn health_check_ready(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let db_health = state.db.health_check().await;

    let status = if db_health.is_ok() {
        "ready"
    } else {
        "not_ready"
    };

    let response = serde_json::json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "dependencies": {
            "mongodb": if db_health.is_ok() { "ok" } else { "error" }
        }
    });

    if db_health.is_ok() {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

#[get("/health/live")]
async fn health_check_live() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(App::new().service(health_check)).await;

        let req = test::TestRequest::get().uri("/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
