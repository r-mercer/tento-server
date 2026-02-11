use actix_web::{middleware::Logger, web, App, HttpMessage, HttpServer};
use actix_cors::Cors;
use actix_web::http;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
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

async fn graphql_playground() -> actix_web::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>GraphQL Playground</title>
                <link rel="stylesheet" href="https://unpkg.com/graphql-playground-react/build/static/css/index.css" />
                <script src="https://unpkg.com/graphql-playground-react/build/static/js/middleware.js"></script>
            </head>
            <body>
                <div id="root"></div>
                <script>
                    window.addEventListener('load', function (event) {
                        GraphQLPlayground.init(document.getElementById('root'), {
                            endpoint: '/graphql',
                            settings: {
                                'request.credentials': 'same-origin'
                            }
                        })
                    })
                </script>
            </body>
            </html>
            "#,
        ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env.local file if it exists
    dotenvy::from_filename(".env.local").ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();

    // Will panic if secrets aren't set
    if env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) != "test" {
        config.validate_for_production();
        log::info!("Configuration validated successfully");
    }

    let app_state = AppState::new(config.clone())
        .await
        .expect("Failed to initialize application state");

    let schema = create_schema(app_state.clone());
    let jwt_service = app_state.jwt_service.clone();

    let host = config.web_server_host.clone();
    let port = config.web_server_port;

    log::info!("Starting server at http://{}:{}", host, port);
    #[cfg(debug_assertions)]
    log::info!(
        "GraphQL Playground available at http://{}:{}/playground",
        host,
        port
    );
    #[cfg(not(debug_assertions))]
    log::info!("GraphQL Playground disabled in release mode");

    HttpServer::new(move || {
        let mut app = App::new()
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
                    .supports_credentials()
            )
            // Public routes
            .service(handlers::health_check)
            .service(handlers::auth_github_callback)
            .service(handlers::refresh_token);

        // Conditionally add playground in debug mode only
        #[cfg(debug_assertions)]
        {
            app = app.route("/playground", web::get().to(graphql_playground));
        }

        app
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
                    .route("/graphql", web::post().to(graphql_handler)),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
