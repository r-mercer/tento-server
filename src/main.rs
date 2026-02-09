use actix_web::{middleware::Logger, web, App, HttpServer};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

pub mod app_state;
pub mod config;
pub mod db;
pub mod errors;
pub mod graphql;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

use app_state::AppState;
use config::Config;
use graphql::create_schema;

async fn graphql_handler(
    schema: web::Data<graphql::Schema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
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
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();

    let app_state = AppState::new(config.clone())
        .await
        .expect("Failed to initialize application state");

    let schema = create_schema();

    let host = config.web_server_host.clone();
    let port = config.web_server_port;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(schema.clone()))
            .wrap(Logger::default())
            .service(handlers::health_check)
            .service(handlers::create_user)
            .service(handlers::get_user)
            .service(handlers::get_all_users)
            .service(handlers::update_user)
            .service(handlers::delete_user)
            .route("/graphql", web::post().to(graphql_handler))
            .route("/playground", web::get().to(graphql_playground))
    })
    .bind((host, port))?
    .run()
    .await
}
