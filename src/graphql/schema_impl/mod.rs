pub mod mutations;
pub mod queries;

use async_graphql::{EmptySubscription, Schema as GraphQLSchema};

use crate::app_state::AppState;

pub use mutations::MutationRoot;
pub use queries::QueryRoot;

pub type Schema = GraphQLSchema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(app_state: AppState) -> Schema {
    GraphQLSchema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(app_state)
        .finish()
}
