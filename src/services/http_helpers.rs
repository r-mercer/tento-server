use actix_web::HttpResponse;

/// Creates an internal server error response
pub fn internal_error(err: impl std::fmt::Display) -> HttpResponse {
    HttpResponse::InternalServerError().body(err.to_string())
}

/// Creates a not found error response
pub fn not_found(message: impl std::fmt::Display) -> HttpResponse {
    HttpResponse::NotFound().body(message.to_string())
}

/// Creates a success JSON response
pub fn success_json<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(data)
}

/// Creates a bad request error response
pub fn bad_request(message: impl std::fmt::Display) -> HttpResponse {
    HttpResponse::BadRequest().body(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    #[test]
    fn test_internal_error() {
        let response = internal_error("Test error");
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_not_found() {
        let response = not_found("Not found");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_bad_request() {
        let response = bad_request("Bad request");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
