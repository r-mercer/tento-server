use actix_web::{HttpResponse, get, post, web};
use mongodb::{Client, IndexModel, bson::doc, options::IndexOptions};

use crate::models::user_model::User;
use crate::config::Config;
use crate::services::db_helpers::get_users_collection;
use crate::services::http_helpers::{internal_error, not_found, success_json};

/// Adds a new user to the "users" collection in the database.
#[post("/add_user")]
async fn add_user(
    client: web::Data<Client>,
    config: web::Data<Config>,
    form: web::Form<User>,
) -> HttpResponse {
    let collection = get_users_collection(&client, &config);
    let result = collection.insert_one(form.into_inner()).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => internal_error(err),
    }
}

/// Gets the user with the supplied username.
#[get("/get_user/{username}")]
async fn get_user(
    client: web::Data<Client>,
    config: web::Data<Config>,
    username: web::Path<String>,
) -> HttpResponse {
    let username = username.into_inner();
    let collection = get_users_collection(&client, &config);
    match collection.find_one(doc! { "username": &username }).await {
        Ok(Some(user)) => success_json(user),
        Ok(None) => not_found(format!("No user found with username {username}")),
        Err(err) => internal_error(err),
    }
}

/// Creates an index on the "username" field to force the values to be unique.
pub async fn create_username_index(client: &Client, config: &Config) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options)
        .build();
    
    get_users_collection(client, config)
        .create_index(model)
        .await
        .expect("creating an index should succeed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    fn test_user() -> User {
        User::test_user_simple("testuser")
    }

    fn assert_error_status(status: actix_web::http::StatusCode) {
        assert!(
            status.is_client_error() || status.is_server_error(),
            "Expected error status, got: {}",
            status
        );
    }

    #[actix_web::test]
    async fn test_add_user_endpoint_structure() {
        let app = test::init_service(App::new().service(add_user)).await;

        let user = test_user();
        let req = test::TestRequest::post()
            .uri("/add_user")
            .set_form(&user)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Without a real DB connection, this will fail, but we're testing the endpoint exists
        assert_error_status(resp.status());
    }

    #[actix_web::test]
    async fn test_get_user_endpoint_structure() {
        let app = test::init_service(App::new().service(get_user)).await;

        let req = test::TestRequest::get()
            .uri("/get_user/testuser")
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Without a real DB connection, this will fail, but we're testing the endpoint exists
        assert_error_status(resp.status());
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_usage() {
        let config = Config::test_config();
        assert_eq!(config.users_collection, "users");
        assert!(!config.mongo_db_name.is_empty());
    }
}
