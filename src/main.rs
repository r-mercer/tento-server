use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, post, web};
use mongodb::{Client, Collection, IndexModel, bson::doc, options::IndexOptions};
mod controllers;
mod db;
mod models;
use crate::controllers::user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri =
        std::env::var("MONGO_CONN_STRING").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");

    println!("starting HTTP server on port 8080");
    println!("GraphiQL playground: http://localhost:8080/graphiql");
    db::mongo::create_username_index(&client).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap(Logger::default())
            .service(user::add_user)
            .service(user::get_user)
        // .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
