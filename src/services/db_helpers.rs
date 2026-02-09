use crate::config::Config;
use crate::models::domain::User;
use mongodb::{Client, Collection};

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
        let config = Config::test_config();
        assert_eq!(config.users_collection, "users");
    }
}
