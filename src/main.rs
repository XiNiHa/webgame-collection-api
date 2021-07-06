mod schema;

pub mod auth;
pub mod config;
pub mod error;

#[cfg(feature = "playground")]
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptySubscription, Schema};
#[cfg(not(feature = "playground"))]
use async_graphql_rocket::Query;
use async_graphql_rocket::{Request, Response};
use auth::auth_info::AuthInfo;
#[cfg(feature = "playground")]
use rocket::response::content::Html;
use rocket::State;
use schema::{AppSchema, QueryRoot};
use sqlx::postgres::PgPoolOptions;

use crate::config::CONFIG;
use crate::schema::MutationRoot;

extern crate rocket;

#[cfg(feature = "playground")]
#[rocket::get("/graphql")]
fn get_graphql_handler() -> Html<String> {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[cfg(not(feature = "playground"))]
#[rocket::get("/graphql?<query..>")]
async fn get_graphql_handler(schema: &State<AppSchema>, auth_info: AuthInfo, query: Query) -> Response {
    let req: Request = query.into();
    req.data(auth_info).execute(schema).await
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn post_graphql_handler(schema: &State<AppSchema>, auth_info: AuthInfo, request: Request) -> Response {
    request.data(auth_info).execute(schema).await
}

#[rocket::launch]
async fn rocket() -> _ {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&CONFIG.database_url)
        .await
        .unwrap();

    rocket::build()
        .manage(
            Schema::build(
                QueryRoot::default(),
                MutationRoot::default(),
                EmptySubscription,
            )
            .data(pool)
            .finish(),
        )
        .mount(
            "/",
            rocket::routes![get_graphql_handler, post_graphql_handler],
        )
}
