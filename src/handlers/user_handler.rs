use actix_web::{get, post, web, HttpResponse};

use crate::{
    app_state::AppState,
    errors::AppError,
    models::dto::request::{CreateUserRequest, UpdateUserRequest},
};

#[post("/api/users")]
async fn create_user(
    state: web::Data<AppState>,
    request: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let response = state.user_service.create_user(request.into_inner()).await?;
    Ok(HttpResponse::Created().json(response))
}

#[get("/api/users/{username}")]
async fn get_user(
    state: web::Data<AppState>,
    username: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let user = state.user_service.get_user(&username).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[get("/api/users")]
async fn get_all_users(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let users = state.user_service.get_all_users().await?;
    Ok(HttpResponse::Ok().json(users))
}

#[post("/api/users/{username}")]
async fn update_user(
    state: web::Data<AppState>,
    username: web::Path<String>,
    request: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let response = state
        .user_service
        .update_user(&username, request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::delete("/api/users/{username}")]
async fn delete_user(
    state: web::Data<AppState>,
    username: web::Path<String>,
) -> Result<HttpResponse, AppError> {
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
