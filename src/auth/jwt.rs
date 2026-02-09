use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    auth::claims::Claims,
    errors::{AppError, AppResult},
    models::domain::user::User,
};

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    expiration_hours: i64,
}

impl JwtService {
    pub fn new(secret: &SecretString, expiration_hours: i64) -> Self {
        let secret_bytes = secret.expose_secret().as_bytes();
        
        Self {
            encoding_key: EncodingKey::from_secret(secret_bytes),
            decoding_key: DecodingKey::from_secret(secret_bytes),
            validation: Validation::default(),
            expiration_hours,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_jwt_create_and_validate() {
        let config = Config::test_config();
        let jwt_service = JwtService::new(&config.jwt_secret, 1);
        
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
        let jwt_service = JwtService::new(&config.jwt_secret, 1);
        
        let result = jwt_service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }
}
