use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse};

use crate::{
    app_state::AppState, auth::AuthenticatedUser, errors::AppError,
    models::dto::request::QuizDraftDto,
};

#[get("/api/quizzes/{id}")]
async fn get_quiz(
    state: web::Data<Arc<AppState>>,
    id: web::Path<String>,
    _auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let quiz = state.quiz_service.get_quiz(&id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(quiz))
}

#[post("/api/quizzes/drafts")]
async fn create_quiz_draft(
    state: web::Data<Arc<AppState>>,
    request: web::Json<QuizDraftDto>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let response = state
        .quiz_service
        .create_quiz_draft(request.into_inner(), &auth.0.sub)
        .await?;
    Ok(HttpResponse::Created().json(response))
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App};

    use super::*;

    #[actix_web::test]
    async fn get_quiz_route_registered_for_get() {
        let app = test::init_service(App::new().service(get_quiz)).await;

        let req = test::TestRequest::post().uri("/api/quizzes/test-id").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn create_quiz_draft_route_registered_for_post() {
        let app = test::init_service(App::new().service(create_quiz_draft)).await;

        let req = test::TestRequest::get().uri("/api/quizzes/drafts").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn get_quiz_without_required_app_data_returns_server_error() {
        let app = test::init_service(App::new().service(get_quiz)).await;

        let req = test::TestRequest::get().uri("/api/quizzes/test-id").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn create_quiz_draft_without_required_app_data_returns_server_error() {
        let app = test::init_service(App::new().service(create_quiz_draft)).await;

        let req = test::TestRequest::post()
            .uri("/api/quizzes/drafts")
            .set_json(serde_json::json!({
                "name": "Draft Quiz",
                "question_count": 5,
                "required_score": 70,
                "attempt_limit": 3,
                "url": "https://example.com"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_server_error());
    }
}
