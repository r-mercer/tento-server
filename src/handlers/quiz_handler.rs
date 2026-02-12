use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::AuthenticatedUser,
    errors::AppError,
    models::dto::request::CreateQuizDraftRequest,
};

#[get("/api/quizzes/{id}")]
async fn get_quiz(
    state: web::Data<Arc<AppState>>,
    id: web::Path<Uuid>,
    _auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let quiz = state.quiz_service.get_quiz(&id).await?;
    Ok(HttpResponse::Ok().json(quiz))
}

#[post("/api/quizzes/drafts")]
async fn create_quiz_draft(
    state: web::Data<Arc<AppState>>,
    request: web::Json<CreateQuizDraftRequest>,
    _auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let response = state.quiz_service.create_quiz_draft(request.into_inner()).await?;
    Ok(HttpResponse::Created().json(response))
}
