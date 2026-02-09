pub mod quiz_repository;
pub mod user_repository;

pub use quiz_repository::{MongoQuizRepository, QuizRepository};
pub use user_repository::{MongoUserRepository, UserRepository};
