use async_graphql::{Context, EmptySubscription, Object, Schema as GraphQLSchema};

use crate::{
    app_state::AppState,
    errors::AppResult,
    models::dto::{
        request::{CreateUserRequest, UpdateUserRequest},
        response::{CreateUserResponse, DeleteUserResponse, UpdateUserResponse, UserDto},
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

pub fn create_schema() -> Schema {
    GraphQLSchema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let schema = create_schema();
        assert!(schema.sdl().contains("type QueryRoot"));
    }
}
