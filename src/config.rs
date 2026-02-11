use secrecy::SecretString;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub mongo_conn_string: String,
    pub mongo_db_name: String,
    pub gh_client_id: String,
    pub gh_client_secret: SecretString,
    pub users_collection: String,
    pub web_server_host: String,
    pub web_server_port: u16,
    pub jwt_secret: SecretString,
    pub jwt_expiration_hours: i64,
    pub func_enums_embed_model: String,
    pub func_enums_max_response_tokens: u32,
    pub func_enums_max_request_tokens: u32,
    pub func_enums_max_func_tokens: u32,
    pub func_enums_embed_path: String,
    pub openai_api_key: SecretString,
    pub openai_base_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            mongo_conn_string: env::var("MONGO_CONN_STRING")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            mongo_db_name: env::var("MONGO_DB_NAME").unwrap_or_else(|_| "tento-local".to_string()),
            gh_client_id: env::var("GH_CLIENT_ID").unwrap_or_else(|_| "gh_client_id".to_string()),
            gh_client_secret: SecretString::from(
                env::var("GH_CLIENT_SECRET").unwrap_or_else(|_| "gh_client_secret".to_string()),
            ),
            users_collection: env::var("USERS_COLLECTION").unwrap_or_else(|_| "users".to_string()),
            web_server_host: env::var("WEB_SERVER_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            web_server_port: env::var("WEB_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            jwt_secret: SecretString::from(
                env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "dev_secret_key_change_in_production".to_string()),
            ),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|h| h.parse().ok())
                .unwrap_or(24),
            func_enums_embed_model: env::var("FUNC_ENUMS_EMBED_MODEL")
                .unwrap_or_else(|_| "text-embedding-embeddinggemma-300m".to_string()),
            func_enums_max_response_tokens: env::var("FUNC_ENUMS_MAX_RESPONSE_TOKENS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(1000),
            func_enums_max_request_tokens: env::var("FUNC_ENUMS_MAX_REQUEST_TOKENS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(4191),
            func_enums_max_func_tokens: env::var("FUNC_ENUMS_MAX_FUNC_TOKENS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(500),
            func_enums_embed_path: env::var("FUNC_ENUMS_EMBED_PATH")
                .unwrap_or_else(|_| "http://localhost:1234".to_string()),
            openai_api_key: SecretString::from(
                env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-default-key".to_string()),
            ),
            openai_base_url: env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:1234".to_string()),
        }
    }

    // Copilot validation rules
    pub fn validate_for_production(&self) {
        use secrecy::ExposeSecret;

        let jwt_secret = self.jwt_secret.expose_secret();
        let gh_secret = self.gh_client_secret.expose_secret();
        let openai_key = self.openai_api_key.expose_secret();

        if jwt_secret == "dev_secret_key_change_in_production" {
            panic!(
                "FATAL: JWT_SECRET is using default value! Set JWT_SECRET environment variable to a secure random string."
            );
        }

        // if jwt_secret.len() < 32 {
        //     panic!(
        //         "FATAL: JWT_SECRET is too short ({}). Must be at least 32 characters for security.",
        //         jwt_secret.len()
        //     );
        // }

        if gh_secret == "gh_client_secret" {
            panic!(
                "FATAL: GH_CLIENT_SECRET is using default value! Set GH_CLIENT_SECRET environment variable."
            );
        }

        if self.gh_client_id == "gh_client_id" {
            panic!(
                "FATAL: GH_CLIENT_ID is using default value! Set GH_CLIENT_ID environment variable."
            );
        }

        if openai_key == "sk-default-key" {
            panic!(
                "FATAL: OPENAI_API_KEY is using default value! Set OPENAI_API_KEY environment variable."
            );
        }

        if self.func_enums_max_response_tokens == 0 {
            panic!("FATAL: FUNC_ENUMS_MAX_RESPONSE_TOKENS must be greater than 0.");
        }

        if self.func_enums_max_request_tokens == 0 {
            panic!("FATAL: FUNC_ENUMS_MAX_REQUEST_TOKENS must be greater than 0.");
        }

        if self.func_enums_max_func_tokens == 0 {
            panic!("FATAL: FUNC_ENUMS_MAX_FUNC_TOKENS must be greater than 0.");
        }

        if self.func_enums_embed_model.is_empty() {
            panic!("FATAL: FUNC_ENUMS_EMBED_MODEL cannot be empty.");
        }

        if !self.func_enums_embed_path.starts_with("http://")
            && !self.func_enums_embed_path.starts_with("https://")
        {
            panic!("FATAL: FUNC_ENUMS_EMBED_PATH must be a valid HTTP/HTTPS URL.");
        }

        if !self.openai_base_url.starts_with("http://")
            && !self.openai_base_url.starts_with("https://")
        {
            panic!("FATAL: OPENAI_BASE_URL must be a valid HTTP/HTTPS URL.");
        }
    }

    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            mongo_conn_string: "mongodb://localhost:27017".to_string(),
            mongo_db_name: "tento-test".to_string(),
            gh_client_id: "id string".to_string(),
            gh_client_secret: SecretString::from("secret string".to_string()),
            users_collection: "users".to_string(),
            web_server_host: "127.0.0.1".to_string(),
            web_server_port: 8080,
            jwt_secret: SecretString::from("test_jwt_secret_key".to_string()),
            jwt_expiration_hours: 1,
            func_enums_embed_model: "test-embed-model".to_string(),
            func_enums_max_response_tokens: 1000,
            func_enums_max_request_tokens: 4191,
            func_enums_max_func_tokens: 500,
            func_enums_embed_path: "http://localhost:1234".to_string(),
            openai_api_key: SecretString::from("sk-test-key".to_string()),
            openai_base_url: "http://localhost:1234".to_string(),
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
