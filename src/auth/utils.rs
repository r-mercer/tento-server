use async_graphql::Context;

use crate::{
    auth::Claims,
    errors::{AppError, AppResult},
    models::domain::user::UserRole,
};

pub fn require_admin(claims: &Claims) -> AppResult<()> {
    if claims.role != UserRole::Admin {
        return Err(AppError::Unauthorized(
            "Only admins can perform this action".to_string(),
        ));
    }
    Ok(())
}

pub fn require_owner_or_admin(claims: &Claims, resource_owner: &str) -> AppResult<()> {
    if claims.role != UserRole::Admin && claims.sub != resource_owner {
        return Err(AppError::Unauthorized(
            "You can only access your own resources".to_string(),
        ));
    }
    Ok(())
}

pub fn extract_claims_from_context(ctx: &Context<'_>) -> AppResult<Claims> {
    ctx.data::<Claims>()
        .cloned()
        .map_err(|_| AppError::Unauthorized("Authentication required".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_claims(username: &str, role: UserRole) -> Claims {
        Claims {
            sub: username.to_string(),
            email: format!("{}@example.com", username),
            role,
            iat: 0,
            exp: 9999999999,
        }
    }

    #[test]
    fn test_require_admin_success() {
        let claims = create_test_claims("admin", UserRole::Admin);
        assert!(require_admin(&claims).is_ok());
    }

    #[test]
    fn test_require_admin_failure() {
        let claims = create_test_claims("user", UserRole::User);
        assert!(require_admin(&claims).is_err());
    }

    #[test]
    fn test_require_owner_or_admin_as_owner() {
        let claims = create_test_claims("john", UserRole::User);
        assert!(require_owner_or_admin(&claims, "john").is_ok());
    }

    #[test]
    fn test_require_owner_or_admin_as_admin() {
        let claims = create_test_claims("admin", UserRole::Admin);
        assert!(require_owner_or_admin(&claims, "other_user").is_ok());
    }

    #[test]
    fn test_require_owner_or_admin_failure() {
        let claims = create_test_claims("john", UserRole::User);
        assert!(require_owner_or_admin(&claims, "jane").is_err());
    }
}
