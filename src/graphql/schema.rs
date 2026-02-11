use async_graphql::{Context, EmptySubscription, Object, Schema as GraphQLSchema, ID};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::{extract_claims_from_context, require_admin, require_owner_or_admin, can_view_quiz_results, can_view_quiz_attempt},
    errors::{AppError, AppResult},
    graphql::helpers::{parse_id, validate_quiz_available_for_taking},
    models::{
        domain::Quiz,
        dto::{
            request::{CreateUserRequest, UpdateUserRequest, SubmitQuizAttemptInput},
            response::{
                CreateUserResponse, DeleteUserResponse, PaginatedResponseUserDto,
                UpdateUserResponse, UserDto, QuizForTaking, QuizAttemptResponse,
                QuizAttemptReview, PaginatedResponseQuizAttempt, PaginationMetadata,
            },
        },
    },
    services::quiz_attempt_service::QuizAttemptService,
};

pub type Schema = GraphQLSchema<QueryRoot, MutationRoot, EmptySubscription>;

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

        extract_claims_from_context(ctx)?;

        let uuid = Uuid::parse_str(&id).map_err(|_| {
            crate::errors::AppError::ValidationError("Invalid UUID format".to_string())
        })?;
        state.quiz_service.get_quiz(&uuid).await
    }

    // Get quiz for taking (without answers)
    async fn quiz_for_taking(&self, ctx: &Context<'_>, id: ID) -> AppResult<QuizForTaking> {
        let state = ctx.data::<AppState>()?;
        extract_claims_from_context(ctx)?;

        let uuid = parse_id(&id)?;
        let quiz = state.quiz_service.get_quiz(&uuid).await?;

        validate_quiz_available_for_taking(&quiz.status)?;

        Ok(QuizForTaking::from_quiz(quiz))
    }

    // Get quiz for results (with answers)
    async fn quiz_for_results(&self, ctx: &Context<'_>, id: ID) -> AppResult<Quiz> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let quiz_uuid = parse_id(&id)?;
        let quiz = state.quiz_service.get_quiz(&quiz_uuid).await?;
        let user_id = parse_id(&claims.sub)?;

        // Check if user has attempted this quiz
        let has_attempted = state
            .quiz_attempt_repository
            .has_user_attempted_quiz(user_id, quiz_uuid)
            .await?;

        // Authorization: User created quiz OR has attempted it
        can_view_quiz_results(user_id, quiz.created_by_user_id, has_attempted)?;

        Ok(quiz)
    }

    // Get user's quiz attempts
    async fn quiz_attempts(
        &self,
        ctx: &Context<'_>,
        quiz_id: Option<ID>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> AppResult<PaginatedResponseQuizAttempt> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let user_id = parse_id(&claims.sub)?;

        let offset = offset.unwrap_or(0).max(0);
        let limit = limit.unwrap_or(10).clamp(1, 50);

        let quiz_id_opt = quiz_id.and_then(|id| parse_id(&id).ok());

        let (attempts, total) = state
            .quiz_attempt_repository
            .get_user_attempts(user_id, quiz_id_opt, offset, limit)
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

    // Get single quiz attempt with details
    async fn quiz_attempt(
        &self,
        ctx: &Context<'_>,
        attempt_id: ID,
    ) -> AppResult<QuizAttemptReview> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let attempt_uuid = parse_id(&attempt_id)?;
        let user_id = parse_id(&claims.sub)?;

        let attempt = state
            .quiz_attempt_repository
            .find_by_id(&attempt_uuid)
            .await?
            .ok_or(AppError::NotFound("Quiz attempt not found".to_string()))?;

        // Authorization: User must own the attempt
        can_view_quiz_attempt(user_id, attempt.user_id)?;

        let quiz = state
            .quiz_service
            .get_quiz(&attempt.quiz_id)
            .await?;

        // Build question results
        let question_results = attempt
            .question_answers
            .iter()
            .map(|qa| {
                let question = quiz
                    .questions
                    .as_ref()
                    .and_then(|qs| qs.iter().find(|q| q.id == qa.quiz_question_id))
                    .ok_or(AppError::NotFound("Question not found".to_string()))?;

                let correct_option_ids: Vec<Uuid> = question
                    .options
                    .iter()
                    .filter(|opt| opt.correct)
                    .map(|opt| opt.id)
                    .collect();

                // Get explanation from first explanation available
                let explanation = question
                    .options
                    .iter()
                    .find(|opt| opt.correct)
                    .map(|opt| opt.explanation.clone())
                    .unwrap_or_default();

                Ok(crate::models::dto::response::QuestionAttemptDetail {
                    question_id: qa.quiz_question_id,
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

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: CreateUserRequest,
    ) -> AppResult<CreateUserResponse> {
        let state = ctx.data::<AppState>()?;

        extract_claims_from_context(ctx)?;

        state.user_service.create_user(input).await
    }

    async fn update_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        input: UpdateUserRequest,
    ) -> AppResult<UpdateUserResponse> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        require_owner_or_admin(&claims, &username)?;

        state.user_service.update_user(&username, input).await
    }

    async fn delete_user(
        &self,
        ctx: &Context<'_>,
        username: String,
    ) -> AppResult<DeleteUserResponse> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        require_owner_or_admin(&claims, &username)?;

        state.user_service.delete_user(&username).await
    }

    // Submit quiz attempt
    async fn submit_quiz_attempt(
        &self,
        ctx: &Context<'_>,
        input: SubmitQuizAttemptInput,
    ) -> AppResult<QuizAttemptResponse> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let user_id = parse_id(&claims.sub)?;
        let quiz_id = parse_id(&input.quiz_id)?;

        // Fetch quiz
        let quiz = state.quiz_service.get_quiz(&quiz_id).await?;

        // Check attempt limit
        let attempt_count = state
            .quiz_attempt_repository
            .count_user_attempts(user_id, quiz_id)
            .await?;

        if attempt_count >= quiz.attempt_limit as usize {
            return Err(AppError::BadRequest(format!(
                "Quiz attempt limit ({}) reached",
                quiz.attempt_limit
            )));
        }

        // Grade the attempt
        let (points_earned, question_answers) =
            QuizAttemptService::grade_attempt(&quiz, &input.answers)?;

        // Determine if passed
        let _passed = points_earned >= quiz.required_score;

        // Create and persist attempt
        let attempt = QuizAttemptService::create_attempt(
            user_id,
            quiz_id,
            points_earned,
            quiz.question_count,
            (attempt_count + 1) as i16,
            quiz.required_score,
            question_answers,
        );

        let attempt = state.quiz_attempt_repository.create(attempt).await?;

        Ok(QuizAttemptResponse::from(attempt))
    }
}

pub fn create_schema(app_state: AppState) -> Schema {
    GraphQLSchema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(app_state)
        .finish()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_schema_creation() {}
}
