mod schema;

pub mod auth;
pub mod chat;
pub mod config;
pub mod error;

use std::future;

use actix_web::{
    guard::Header, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use async_graphql::Data;
use async_graphql_actix_web::{Request, Response, WSSubscription};
use auth::auth_info::AuthInfo;
use chat::ChatData;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::mpsc::{self, Sender};

use crate::config::CONFIG;
use crate::schema::{
    mutation::MutationRoot, query::QueryRoot, subscription::SubscriptionRoot, AppSchema,
};

#[cfg(feature = "playground")]
async fn playground_handler() -> Result<HttpResponse> {
    use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
        )))
}

async fn graphql_handler(
    schema: web::Data<AppSchema>,
    req: Request,
    auth_info: AuthInfo,
    chat_tx: web::Data<Sender<ChatData>>,
) -> Response {
    let cloned = chat_tx.get_ref().clone();
    schema
        .execute(req.into_inner().data(auth_info).data(cloned))
        .await
        .into()
}

async fn subscription_handler(
    schema: web::Data<AppSchema>,
    req: HttpRequest,
    stream: web::Payload,
    chat_tx: web::Data<Sender<ChatData>>,
) -> Result<HttpResponse> {
    let cloned = chat_tx.get_ref().clone();

    WSSubscription::start_with_initializer(AppSchema::clone(&*schema), &req, stream, |value| {
        let mut data = Data::default();

        data.insert(AuthInfo::from_header(
            value
                .as_object()
                .and_then(|m| m.get("Authorization"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        ));
        data.insert(cloned);
        future::ready(Ok(data))
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    #[cfg(not(feature = "sqs"))]
    let (chat_tx, chat_handle) = {
        let (tx, rx) = mpsc::channel(64);
        (tx, tokio::spawn(chat::broadcast(rx)))
    };

    let schema_data = web::Data::new(build_schema().await);
    let chat_tx_data = web::Data::new(chat_tx.clone());

    let actix_result = HttpServer::new(move || {
        let app = App::new()
            .app_data(schema_data.clone())
            .wrap(Logger::default())
            .route("/graphql", web::post().to(graphql_handler))
            .route(
                "/graphql",
                web::get()
                    .guard(Header("Upgrade", "websocket"))
                    .to(subscription_handler),
            );

        #[cfg(not(feature = "sqs"))]
        let app = app.app_data(chat_tx_data.clone());

        #[cfg(feature = "playground")]
        let app = app.route("/graphql", web::get().to(playground_handler));

        app
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await;

    chat_handle.abort();

    actix_result
}

async fn build_schema() -> AppSchema {
    AppSchema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot::default(),
    )
    .data(build_pool().await)
    .finish()
}

async fn build_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&CONFIG.database_url)
        .await
        .unwrap()
}
