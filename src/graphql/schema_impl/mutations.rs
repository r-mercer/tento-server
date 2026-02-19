use async_graphql::{Context, Object};

use crate::{
    app_state::AppState,
    auth::{extract_claims_from_context, require_owner_or_admin},
    errors::AppResult,
    graphql::helpers::parse_id,
    models::{
        domain::Quiz,
        dto::{
            request::{
                CreateUserRequestDto, SubmitQuizAttemptInput, UpdateQuizInput, UpdateUserRequestDto,
            },
            response::{
                CreateUserResponse, DeleteUserResponse, QuizAttemptResponse, UpdateUserResponse,
            },
        },
    },
    services::quiz_attempt_service::QuizAttemptService,
};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: CreateUserRequestDto,
    ) -> AppResult<CreateUserResponse> {
        let state = ctx.data::<AppState>()?;

        extract_claims_from_context(ctx)?;

        state.user_service.create_user(input).await
    }

    async fn update_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        input: UpdateUserRequestDto,
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

    async fn submit_quiz_attempt(
        &self,
        ctx: &Context<'_>,
        input: SubmitQuizAttemptInput,
    ) -> AppResult<QuizAttemptResponse> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let user_id = claims.sub.clone();
        let quiz_id = parse_id(&input.quiz_id)?;

        let quiz_dto = state.quiz_service.get_quiz(&quiz_id).await?;
        let quiz: Quiz = quiz_dto.try_into()?;

        let attempt_count = state
            .quiz_attempt_repository
            .count_user_attempts(&user_id, &quiz_id)
            .await?;

        if attempt_count >= quiz.attempt_limit as usize {
            return Err(crate::errors::AppError::BadRequest(format!(
                "Quiz attempt limit ({}) reached",
                quiz.attempt_limit
            )));
        }

        let (points_earned, question_answers) =
            QuizAttemptService::grade_attempt(&quiz, &input.answers)?;

        let _passed = points_earned >= quiz.required_score;

        let attempt = QuizAttemptService::create_attempt(
            &user_id,
            &quiz_id,
            points_earned,
            quiz.question_count,
            (attempt_count + 1) as i16,
            quiz.required_score,
            question_answers,
        );

        let attempt = state.quiz_attempt_repository.create(attempt).await?;

        Ok(QuizAttemptResponse::from(attempt))
    }

    async fn update_quiz(&self, ctx: &Context<'_>, input: UpdateQuizInput) -> AppResult<Quiz> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;

        let existing_quiz = state.quiz_service.get_quiz(&input.id).await?;

        require_owner_or_admin(&claims, &existing_quiz.created_by_user_id)?;

        let updated_quiz = state.quiz_service.update_quiz_partial(input).await?;

        updated_quiz.try_into()
    }
}
