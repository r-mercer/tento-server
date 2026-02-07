use crate::config::Config;
use crate::models::user_model::User;
use mongodb::{Client, Collection};

/// Gets the users collection from the MongoDB client
pub fn get_users_collection(client: &Client, config: &Config) -> Collection<User> {
    client
        .database(&config.mongo_db_name)
        .collection(&config.users_collection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_users_collection() {
        // This test verifies the function signature compiles
        // Actual MongoDB connection testing would require integration tests
        let config = Config::test_config();
        assert_eq!(config.users_collection, "users");
    }
}
