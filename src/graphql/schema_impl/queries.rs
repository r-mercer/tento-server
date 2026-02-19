use async_graphql::{Context, Object, ID};

use crate::{
    app_state::AppState,
    auth::{
        can_view_quiz_attempt, can_view_quiz_results, extract_claims_from_context, require_admin,
        require_owner_or_admin,
    },
    errors::{AppError, AppResult},
    graphql::helpers::{parse_id, validate_quiz_available_for_taking},
    models::{
        domain::Quiz,
        dto::response::{
            PaginatedResponseQuizAttempt, PaginatedResponseUserDto, PaginationMetadata,
            QuizAttemptResponse, QuizAttemptReview, QuizForTaking, UserDto,
        },
    },
};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn user(&self, ctx: &Context<'_>, username: String) -> AppResult<UserDto> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        require_owner_or_admin(&claims, &username)?;

        state.user_service.get_user(&username).await
    }

    async fn users(
        &self,
        ctx: &Context<'_>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> AppResult<PaginatedResponseUserDto> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        require_admin(&claims)?;

        let offset = offset.unwrap_or(0).max(0);
        let limit = limit.unwrap_or(20).clamp(1, 100);

        state
            .user_service
            .get_all_users_paginated(offset, limit)
            .await
    }

    async fn quiz(&self, ctx: &Context<'_>, id: ID) -> AppResult<Quiz> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let id_str = parse_id(&id)?;
        let quiz_dto = state.quiz_service.get_quiz(&id_str).await?;

        if quiz_dto.created_by_user_id != claims.sub {
            return Err(AppError::Forbidden(
                "Only quiz creator can view full quiz with answers".into(),
            ));
        }

        let quiz: Quiz = quiz_dto.try_into()?;
        Ok(quiz)
    }

    async fn quiz_for_taking(&self, ctx: &Context<'_>, id: ID) -> AppResult<QuizForTaking> {
        let state = ctx.data::<AppState>()?;
        extract_claims_from_context(ctx)?;

        let id_str = parse_id(&id)?;
        let quiz_dto = state.quiz_service.get_quiz(&id_str).await?;

        validate_quiz_available_for_taking(&quiz_dto.status)?;

        let quiz: Quiz = quiz_dto.try_into()?;
        Ok(QuizForTaking::from_quiz(quiz))
    }

    async fn quiz_for_results(&self, ctx: &Context<'_>, id: ID) -> AppResult<Quiz> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let quiz_id = parse_id(&id)?;
        let quiz_dto = state.quiz_service.get_quiz(&quiz_id).await?;
        let user_id = claims.sub.clone();

        let has_attempted = state
            .quiz_attempt_repository
            .has_user_attempted_quiz(&user_id, &quiz_id)
            .await?;

        can_view_quiz_results(&user_id, &quiz_dto.created_by_user_id, has_attempted)?;

        let quiz: Quiz = quiz_dto.try_into()?;
        Ok(quiz)
    }

    async fn quizzes(
        &self,
        ctx: &Context<'_>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> AppResult<Vec<Quiz>> {
        let state = ctx.data::<AppState>()?;

        extract_claims_from_context(ctx)?;

        let offset = offset.unwrap_or(0).max(0);
        let limit = limit.unwrap_or(20).clamp(1, 100);

        let (quiz_dtos, _total) = state.quiz_service.list_quizzes(offset, limit).await?;

        let quizzes = quiz_dtos
            .into_iter()
            .map(|qdto| qdto.try_into())
            .collect::<AppResult<Vec<Quiz>>>()?;

        Ok(quizzes)
    }

    async fn user_quizzes(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> AppResult<Vec<Quiz>> {
        let state = ctx.data::<AppState>()?;

        extract_claims_from_context(ctx)?;

        let user_id_str = user_id.to_string();

        let offset = offset.unwrap_or(0).max(0);
        let limit = limit.unwrap_or(20).clamp(1, 100);

        let (quiz_dtos, _total) = state
            .quiz_service
            .list_quizzes_by_user(&user_id_str, offset, limit)
            .await?;

        let quizzes = quiz_dtos
            .into_iter()
            .map(|qdto| qdto.try_into())
            .collect::<AppResult<Vec<Quiz>>>()?;

        Ok(quizzes)
    }

    async fn quiz_attempts(
        &self,
        ctx: &Context<'_>,
        quiz_id: Option<ID>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> AppResult<PaginatedResponseQuizAttempt> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let user_id = claims.sub.clone();

        let offset = offset.unwrap_or(0).max(0);
        let limit = limit.unwrap_or(10).clamp(1, 50);

        let quiz_id_opt = quiz_id.and_then(|id| parse_id(&id).ok());

        let (attempts, total) = state
            .quiz_attempt_repository
            .get_user_attempts(&user_id, quiz_id_opt.as_deref(), offset, limit)
            .await?;

        let data = attempts
            .into_iter()
            .map(QuizAttemptResponse::from)
            .collect();

        Ok(PaginatedResponseQuizAttempt {
            data,
            pagination: PaginationMetadata {
                offset,
                limit,
                total,
            },
        })
    }

    async fn quiz_attempt(
        &self,
        ctx: &Context<'_>,
        attempt_id: ID,
    ) -> AppResult<QuizAttemptReview> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let attempt_id_str = parse_id(&attempt_id)?;
        let user_id = claims.sub.clone();

        let attempt = state
            .quiz_attempt_repository
            .find_by_id(&attempt_id_str)
            .await?
            .ok_or(AppError::NotFound("Quiz attempt not found".to_string()))?;

        can_view_quiz_attempt(&user_id, &attempt.user_id)?;

        let quiz_dto = state.quiz_service.get_quiz(&attempt.quiz_id).await?;

        let quiz: Quiz = quiz_dto.try_into()?;

        let question_results = attempt
            .question_answers
            .iter()
            .map(|qa| {
                let question = quiz
                    .questions
                    .as_ref()
                    .and_then(|qs| qs.iter().find(|q| q.id == qa.quiz_question_id))
                    .ok_or(AppError::NotFound("Question not found".to_string()))?;

                let correct_option_ids: Vec<String> = question
                    .options
                    .iter()
                    .filter(|opt| opt.correct)
                    .map(|opt| opt.id.clone())
                    .collect();

                let explanation = question
                    .options
                    .iter()
                    .find(|opt| opt.correct)
                    .map(|opt| opt.explanation.clone())
                    .unwrap_or_default();

                Ok(crate::models::dto::response::QuestionAttemptDetail {
                    question_id: qa.quiz_question_id.clone(),
                    user_selected_option_ids: qa.selected_option_ids.clone(),
                    correct_option_ids,
                    is_correct: qa.is_correct,
                    points_earned: qa.points_earned,
                    explanation,
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(QuizAttemptReview {
            attempt: QuizAttemptResponse::from(attempt),
            quiz: quiz.into(),
            question_results,
        })
    }
}
