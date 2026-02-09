use tento_server::models::domain::User;

#[actix_web::test]
async fn test_user_serialization_in_form() {
    let user = User::new("Test", "User", "inttest", "inttest@example.com");

    let json_str = serde_json::to_string(&user).unwrap();
    let deserialized: User = serde_json::from_str(&json_str).unwrap();

    assert_eq!(user, deserialized);
}

#[cfg(test)]
mod tests {
    use tento_server::models::domain::User;

    #[test]
    fn test_user_struct_size() {
        use std::mem;
        let size = mem::size_of::<User>();
        assert!(
            size <= 200,
            "User struct size is {} bytes, which seems too large",
            size
        );
    }
}
