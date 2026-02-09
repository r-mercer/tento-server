pub mod user_handler;

pub use user_handler::{
    create_user, delete_user, get_all_users, get_user, health_check, update_user,
};
