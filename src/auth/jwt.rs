use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    auth::claims::{Claims, RefreshClaims},
    errors::{AppError, AppResult},
    models::domain::user::User,
};

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    expiration_hours: i64,
    refresh_expiration_hours: i64,
}

impl JwtService {
    pub fn new(secret: &SecretString, expiration_hours: i64, refresh_expiration_hours: i64) -> Self {
        let secret_bytes = secret.expose_secret().as_bytes();

        Self {
            encoding_key: EncodingKey::from_secret(secret_bytes),
            decoding_key: DecodingKey::from_secret(secret_bytes),
            validation: Validation::default(),
            expiration_hours,
            refresh_expiration_hours,
        }
    }

    pub fn create_token(&self, user: &User) -> AppResult<String> {
        let claims = Claims::new(user, self.expiration_hours);

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalError(format!("Failed to create JWT: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))
    }

    pub fn create_refresh_token(&self, username: &str) -> AppResult<String> {
        let claims = RefreshClaims::new(username, self.refresh_expiration_hours);

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalError(format!("Failed to create refresh token: {}", e)))
    }

    pub fn refresh_expiration_hours(&self) -> i64 {
        self.refresh_expiration_hours
    }

    pub fn validate_refresh_token(&self, token: &str) -> AppResult<RefreshClaims> {
        let token_data = decode::<RefreshClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::Unauthorized("Refresh token has expired".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    AppError::Unauthorized("Invalid refresh token format".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    AppError::Unauthorized("Refresh token signature is invalid".to_string())
                }
                _ => AppError::Unauthorized(format!("Refresh token validation failed: {}", e)),
            })?;

        // Additional validation: ensure it's a refresh token
        if token_data.claims.token_type != "refresh" {
            return Err(AppError::Unauthorized(
                "Token is not a refresh token".to_string(),
            ));
        }

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_jwt_create_and_validate() {
        let config = Config::test_config();
        let jwt_service = JwtService::new(&config.jwt_secret, 1, 168);

        let user = User::new("John", "Doe", "johndoe", "john@example.com");
        let token = jwt_service.create_token(&user).unwrap();

        assert!(!token.is_empty());

        let claims = jwt_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "johndoe");
        assert_eq!(claims.email, "john@example.com");
    }

    #[test]
    fn test_jwt_invalid_token() {
        let config = Config::test_config();
        let jwt_service = JwtService::new(&config.jwt_secret, 1, 168);

        let result = jwt_service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_create_and_validate() {
        let config = Config::test_config();
        let jwt_service = JwtService::new(&config.jwt_secret, 1, 168);

        let refresh_token = jwt_service.create_refresh_token("johndoe").unwrap();
        assert!(!refresh_token.is_empty());

        let claims = jwt_service.validate_refresh_token(&refresh_token).unwrap();
        assert_eq!(claims.sub, "johndoe");
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_refresh_token_invalid() {
        let config = Config::test_config();
        let jwt_service = JwtService::new(&config.jwt_secret, 1, 168);

        let result = jwt_service.validate_refresh_token("invalid.token.here");
        assert!(result.is_err());

        // Verify it's an Unauthorized error with appropriate message
        match result {
            Err(AppError::Unauthorized(msg)) => {
                assert!(msg.contains("refresh token") || msg.contains("Refresh token"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }
}
