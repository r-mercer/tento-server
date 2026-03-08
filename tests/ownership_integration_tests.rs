use tento_server::auth::{require_owner_or_admin, require_quiz_owner, Claims};
use tento_server::models::domain::user::UserRole;

#[test]
fn require_quiz_owner_allows_owner() {
    let claims = Claims {
        sub: "user-1".to_string(),
        username: "user1".to_string(),
        email: "u1@example.com".to_string(),
        role: UserRole::User,
        iat: 0,
        exp: 9999999999,
    };

    assert!(require_quiz_owner(&claims, "user-1").is_ok());
}

#[test]
fn require_quiz_owner_denies_other() {
    let claims = Claims {
        sub: "user-2".to_string(),
        username: "user2".to_string(),
        email: "u2@example.com".to_string(),
        role: UserRole::User,
        iat: 0,
        exp: 9999999999,
    };

    assert!(require_quiz_owner(&claims, "user-1").is_err());
}

#[test]
fn require_owner_or_admin_allows_admin() {
    let claims = Claims {
        sub: "admin-1".to_string(),
        username: "admin".to_string(),
        email: "admin@example.com".to_string(),
        role: UserRole::Admin,
        iat: 0,
        exp: 9999999999,
    };

    assert!(require_owner_or_admin(&claims, "some-other").is_ok());
}
