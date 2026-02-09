use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client, Collection,
};
use std::time::Duration;

use crate::{config::Config, errors::AppResult};

#[derive(Clone)]
pub struct Database {
    client: Client,
    db_name: String,
}

impl Database {
    pub async fn connect(config: &Config) -> AppResult<Self> {
        let mut client_options = ClientOptions::parse(&config.mongo_conn_string).await?;

        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);
        client_options.max_pool_size = Some(10);
        client_options.min_pool_size = Some(2);
        client_options.connect_timeout = Some(Duration::from_secs(5));
        client_options.server_selection_timeout = Some(Duration::from_secs(5));

        let client = Client::with_options(client_options)?;

        client
            .database("admin")
            .run_command(doc! { "ping": 1 })
            .await?;

        println!("✓ Successfully connected to MongoDB");

        Ok(Self {
            client,
            db_name: config.mongo_db_name.clone(),
        })
    }

    pub fn get_collection<T>(&self, collection_name: &str) -> Collection<T>
    where
        T: Send + Sync,
    {
        self.client
            .database(&self.db_name)
            .collection(collection_name)
    }
    pub async fn health_check(&self) -> AppResult<()> {
        self.client
            .database("admin")
            .run_command(doc! { "ping": 1 })
            .await?;
        Ok(())
    }
    pub fn client(&self) -> &Client {
        &self.client
    }
    pub fn db_name(&self) -> &str {
        &self.db_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_structure() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Database>();
    }
}
