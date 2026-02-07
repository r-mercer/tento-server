use actix_web::{App, HttpServer, middleware::Logger, web};
use mongodb::Client;

pub mod models;
pub mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri =
        std::env::var("MONGO_CONN_STRING").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");

    println!("starting HTTP server on port 8080");
    println!("GraphiQL playground: http://localhost:8080/graphiql");

    services::user_service::create_username_index(&client).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap(Logger::default())
            .service(services::user_service::add_user)
            .service(services::user_service::get_user)
        // .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
