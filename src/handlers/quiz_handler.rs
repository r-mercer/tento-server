use actix_web::{get, web, HttpResponse};
use uuid::Uuid;

use crate::{app_state::AppState, auth::AuthenticatedUser, errors::AppError};

#[get("/api/quizzes/{id}")]
async fn get_quiz(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    _auth: AuthenticatedUser,  // Require authentication
) -> Result<HttpResponse, AppError> {
    let quiz = state.quiz_service.get_quiz(&id).await?;
    Ok(HttpResponse::Ok().json(quiz))
}
