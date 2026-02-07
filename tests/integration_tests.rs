use tento_server::models::user_model::User;

#[actix_web::test]
async fn test_user_serialization_in_form() {
    let user = User {
        first_name: "Integration".to_string(),
        last_name: "Test".to_string(),
        username: "inttest".to_string(),
        email: "integration@test.com".to_string(),
    };

    let json_str = serde_json::to_string(&user).unwrap();
    let deserialized: User = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(user, deserialized);
}

#[cfg(test)]
mod sync_tests {
    use tento_server::models::user_model::User;

    #[test]
    fn test_user_struct_size() {
        use std::mem;
        // Ensures User struct remains reasonably sized
        let size = mem::size_of::<User>();
        // User contains 4 Strings, each String is 24 bytes on 64-bit systems
        assert!(size <= 200, "User struct size is {} bytes, which seems too large", size);
    }
}
