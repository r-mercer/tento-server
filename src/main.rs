use actix_cors::Cors;
use actix_web::http;
use actix_web::{middleware::Logger, web, App, HttpMessage, HttpServer};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use secrecy::ExposeSecret as _;
use std::env;

pub mod app_state;
pub mod auth;
pub mod config;
pub mod constants;
pub mod db;
pub mod errors;
pub mod graphql;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

use app_state::AppState;
use auth::AuthMiddleware;
use config::Config;
use graphql::create_schema;

async fn graphql_handler(
    schema: web::Data<graphql::Schema>,
    req: GraphQLRequest,
    http_req: actix_web::HttpRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();

    if let Some(claims) = http_req.extensions().get::<auth::Claims>() {
        request = request.data(claims.clone());
    }

    schema.execute(request).await.into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::from_filename(".env.local").ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();

    // Panic if secrets aren't set
    if env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) != "test" {
        config.validate_for_production();
        log::info!("Configuration validated successfully");
    }

    // Log OAuth configuration (without exposing secrets)
    log::info!(
        "GitHub OAuth configured with Client ID: {}",
        config.gh_client_id
    );
    log::info!(
        "GitHub OAuth Client Secret is {} characters",
        config.gh_client_secret.expose_secret().len()
    );

    let app_state = AppState::new(config.clone())
        .await
        .expect("Failed to initialize application state");

    let app_state = std::sync::Arc::new(app_state);

    // Set app state on orchestrator and start worker
    app_state
        .agent_orchestrator
        .set_app_state(app_state.clone())
        .await;
    app_state
        .agent_orchestrator
        .start_worker()
        .await
        .expect("Failed to start background worker");

    let schema = create_schema((*app_state).clone());
    let jwt_service = app_state.jwt_service.clone();

    let host = config.web_server_host.clone();
    let port = config.web_server_port;

    log::info!("Starting server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::from(jwt_service.clone()))
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::CONTENT_TYPE,
                    ])
                    .expose_headers(vec![http::header::AUTHORIZATION])
                    .max_age(3600)
                    .supports_credentials(),
            )
            // Public routes
            .service(handlers::health_check)
            .service(handlers::auth_github_callback)
            .service(handlers::refresh_token)
            .service(handlers::logout)
            // Protected routes
            .service(
                web::scope("")
                    .wrap(AuthMiddleware)
                    .service(handlers::create_user)
                    .service(handlers::get_user)
                    .service(handlers::get_all_users)
                    .service(handlers::update_user)
                    .service(handlers::delete_user)
                    .service(handlers::get_quiz)
                    .service(handlers::create_quiz_draft)
                    .route("/graphql", web::post().to(graphql_handler)),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
