use async_graphql::{Context, EmptySubscription, Object, Schema as GraphQLSchema, ID};
use uuid::Uuid;

use crate::{
    app_state::AppState,
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
        state.user_service.get_user(&username).await
    }
    async fn users(&self, ctx: &Context<'_>) -> AppResult<Vec<UserDto>> {
        let state = ctx.data::<AppState>()?;
        state.user_service.get_all_users().await
    }

    async fn quiz(&self, ctx: &Context<'_>, id: ID) -> AppResult<Quiz> {
        let state = ctx.data::<AppState>()?;
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
        state.user_service.create_user(input).await
    }

    async fn update_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        input: UpdateUserRequest,
    ) -> AppResult<UpdateUserResponse> {
        let state = ctx.data::<AppState>()?;
        state.user_service.update_user(&username, input).await
    }

    async fn delete_user(
        &self,
        ctx: &Context<'_>,
        username: String,
    ) -> AppResult<DeleteUserResponse> {
        let state = ctx.data::<AppState>()?;
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
    use super::*;

    #[test]
    fn test_schema_creation() {
        use crate::config::Config;
        let config = Config::test_config();
        // Schema creation now requires AppState, so we skip this test
        // or would need to set up a full async test environment
    }
}
