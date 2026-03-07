#[cfg(test)]
mod tests {
    use crate::handlers::auth_handler::validate_and_select_redirect_uri;

    #[test]
    fn allowed_provided_redirect_is_accepted() {
        let allowed = vec![
            "http://example.com".to_string(),
            "https://app.test:8443".to_string(),
        ];
        let provided = Some("https://app.test:8443/some/path?q=1");

        let result = validate_and_select_redirect_uri(&allowed, provided).unwrap();
        assert_eq!(result, "https://app.test:8443/some/path?q=1");
    }

    #[test]
    fn disallowed_provided_redirect_is_rejected() {
        let allowed = vec!["http://example.com".to_string()];
        let provided = Some("https://evil.com/callback");

        let result = validate_and_select_redirect_uri(&allowed, provided);
        assert!(result.is_err());
    }

    #[test]
    fn no_provided_redirect_uses_first_allowed() {
        let allowed = vec![
            "http://localhost:5173".to_string(),
            "http://localhost:3000".to_string(),
        ];

        let result = validate_and_select_redirect_uri(&allowed, None).unwrap();
        assert_eq!(result, "http://localhost:5173/auth/callback");
    }
}
