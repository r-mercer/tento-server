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

/// Check if user is the creator of the quiz or has attempted it
pub fn can_view_quiz_results(
    user_id: &str,
    quiz_creator_id: &str,
    has_user_attempted: bool,
) -> AppResult<()> {
    if user_id != quiz_creator_id && !has_user_attempted {
        return Err(AppError::Forbidden(
            "You can only view results for quizzes you created or have attempted".to_string(),
        ));
    }
    Ok(())
}

/// Check if user owns the quiz attempt
pub fn can_view_quiz_attempt(user_id: &str, attempt_user_id: &str) -> AppResult<()> {
    if user_id != attempt_user_id {
        return Err(AppError::Forbidden(
            "You can only view your own quiz attempts".to_string(),
        ));
    }
    Ok(())
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

    #[test]
    fn test_can_view_quiz_results_as_creator() {
        let user_id = "550e8400-e29b-41d4-a716-446655440000";
        assert!(can_view_quiz_results(user_id, user_id, false).is_ok());
    }

    #[test]
    fn test_can_view_quiz_results_after_attempt() {
        let user_id = "550e8400-e29b-41d4-a716-446655440000";
        let quiz_creator = "550e8400-e29b-41d4-a716-446655440001";
        assert!(can_view_quiz_results(user_id, quiz_creator, true).is_ok());
    }

    #[test]
    fn test_can_view_quiz_results_forbidden() {
        let user_id = "550e8400-e29b-41d4-a716-446655440000";
        let quiz_creator = "550e8400-e29b-41d4-a716-446655440001";
        assert!(can_view_quiz_results(user_id, quiz_creator, false).is_err());
    }

    #[test]
    fn test_can_view_quiz_attempt_as_owner() {
        let user_id = "550e8400-e29b-41d4-a716-446655440000";
        assert!(can_view_quiz_attempt(user_id, user_id).is_ok());
    }

    #[test]
    fn test_can_view_quiz_attempt_forbidden() {
        let user_id = "550e8400-e29b-41d4-a716-446655440000";
        let attempt_user_id = "550e8400-e29b-41d4-a716-446655440001";
        assert!(can_view_quiz_attempt(user_id, attempt_user_id).is_err());
    }
}
