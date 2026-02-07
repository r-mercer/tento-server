use actix_web::{App, HttpServer, middleware::Logger, web};
use mongodb::Client;

pub mod config;
pub mod models;
pub mod services;

use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    
    let client = Client::with_uri_str(&config.mongo_conn_string)
        .await
        .expect("failed to connect");

    println!("starting HTTP server on port 8080");
    println!("GraphiQL playground: http://localhost:8080/graphiql");

    services::user_service::create_username_index(&client, &config).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(services::user_service::add_user)
            .service(services::user_service::get_user)
        // .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
