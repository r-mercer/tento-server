use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub mongo_conn_string: String,
    pub mongo_db_name: String,
    pub users_collection: String,
    pub web_server_host: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            mongo_conn_string: env::var("MONGO_CONN_STRING")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            mongo_db_name: env::var("MONGO_DB_NAME").unwrap_or_else(|_| "tento-local".to_string()),
            users_collection: env::var("USERS_COLLECTION").unwrap_or_else(|_| "users".to_string()),
            web_server_host: env::var("WEB_SERVER_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
        }
    }

    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            mongo_conn_string: "mongodb://localhost:27017".to_string(),
            mongo_db_name: "tento-test".to_string(),
            users_collection: "users".to_string(),
            web_server_host: "127.0.0.1".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env_with_defaults() {
        let config = Config::from_env();

        // Should use env vars if set, or fall back to defaults
        assert!(!config.mongo_conn_string.is_empty());
        assert!(!config.mongo_db_name.is_empty());
        assert_eq!(config.users_collection, "users");
    }

    #[test]
    fn test_test_config() {
        let config = Config::test_config();

        assert_eq!(config.mongo_conn_string, "mongodb://localhost:27017");
        assert_eq!(config.mongo_db_name, "tento-test");
        assert_eq!(config.users_collection, "users");
    }
}
