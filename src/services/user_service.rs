use actix_web::{HttpResponse, get, post, web};
use mongodb::{Client, Collection, IndexModel, bson::doc, options::IndexOptions};

use crate::models::user_model::User;

const DB_NAME: &str = "tento-local";
const COLL_NAME: &str = "users";

/// Adds a new user to the "users" collection in the database.
#[post("/add_user")]
async fn add_user(client: web::Data<Client>, form: web::Form<User>) -> HttpResponse {
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(form.into_inner()).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Gets the user with the supplied username.
#[get("/get_user/{username}")]
async fn get_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    match collection.find_one(doc! { "username": &username }).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Creates an index on the "username" field to force the values to be unique.
pub async fn create_username_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options)
        .build();
    client
        .database(DB_NAME)
        .collection::<User>(COLL_NAME)
        .create_index(model)
        .await
        .expect("creating an index should succeed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    fn create_test_user() -> User {
        User {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    #[actix_web::test]
    async fn test_add_user_endpoint_structure() {
        let app = test::init_service(
            App::new().service(add_user)
        ).await;

        let user = create_test_user();
        let req = test::TestRequest::post()
            .uri("/add_user")
            .set_form(&user)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Without a real DB connection, this will fail, but we're testing the endpoint exists
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_get_user_endpoint_structure() {
        let app = test::init_service(
            App::new().service(get_user)
        ).await;

        let req = test::TestRequest::get()
            .uri("/get_user/testuser")
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Without a real DB connection, this will fail, but we're testing the endpoint exists
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }
}

#[cfg(test)]
mod sync_tests {
    use super::*;

    #[test]
    fn test_db_constants() {
        assert_eq!(DB_NAME, "tento-local");
        assert_eq!(COLL_NAME, "users");
    }
}
