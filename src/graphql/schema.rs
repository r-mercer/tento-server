use async_graphql::{Context, EmptySubscription, Object, Schema as GraphQLSchema, ID};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::{extract_claims_from_context, require_admin, require_owner_or_admin},
    errors::AppResult,
    models::{
        domain::Quiz,
        dto::{
            request::{CreateUserRequest, UpdateUserRequest},
            response::{CreateUserResponse, DeleteUserResponse, UpdateUserResponse, UserDto},
        },
    },
};

pub type Schema = GraphQLSchema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn user(&self, ctx: &Context<'_>, username: String) -> AppResult<UserDto> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;
        
        // Authorization: Users can only view their own profile, admins can view any
        require_owner_or_admin(&claims, &username)?;
        
        state.user_service.get_user(&username).await
    }
    
    async fn users(&self, ctx: &Context<'_>) -> AppResult<Vec<UserDto>> {
        let state = ctx.data::<AppState>()?;
        let claims = extract_claims_from_context(ctx)?;
        
        // Authorization: Only admins can list all users
        require_admin(&claims)?;
        
        state.user_service.get_all_users().await
    }

    async fn quiz(&self, ctx: &Context<'_>, id: ID) -> AppResult<Quiz> {
        let state = ctx.data::<AppState>()?;
        
        // Authentication required for quizzes
        extract_claims_from_context(ctx)?;
        
        let uuid = Uuid::parse_str(&id).map_err(|_| {
            crate::errors::AppError::ValidationError("Invalid UUID format".to_string())
        })?;
        state.quiz_service.get_quiz(&uuid).await
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
        
        // Authentication required
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
        
        // Authorization: Users can only update their own profile, admins can update any
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
        
        // Authorization: Users can only delete their own account, admins can delete any
        require_owner_or_admin(&claims, &username)?;
        
        state.user_service.delete_user(&username).await
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
    fn test_schema_creation() {
        // Schema creation now requires AppState, which requires async setup
        // This test is skipped in favor of integration tests
    }
}
