pub mod claims;
pub mod jwt;
pub mod middleware;
pub mod utils;

pub use claims::Claims;
pub use jwt::JwtService;
pub use middleware::{AuthMiddleware, AuthenticatedUser};
pub use utils::{
    can_view_quiz_attempt, can_view_quiz_results, extract_claims_from_context, require_admin,
    require_owner_or_admin,
};
