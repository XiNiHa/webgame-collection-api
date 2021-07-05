mod schema;

pub mod auth;

use std::{env, num::NonZeroU32};

#[cfg(feature = "playground")]
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptySubscription, Schema};
#[cfg(not(feature = "playground"))]
use async_graphql_rocket::Query;
use async_graphql_rocket::{Request, Response};
use dotenv::dotenv;
#[cfg(feature = "playground")]
use rocket::response::content::Html;
use rocket::State;
use schema::{AppSchema, QueryRoot};
use sqlx::postgres::PgPoolOptions;

use crate::schema::{AppConfig, AppContext, MutationRoot};

extern crate rocket;

#[cfg(feature = "playground")]
#[rocket::get("/graphql")]
fn get_graphql_handler() -> Html<String> {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[cfg(not(feature = "playground"))]
#[rocket::get("/graphql?<query..>")]
async fn get_graphql_handler(schema: &State<AppSchema>, query: Query) -> Response {
    query.execute(schema).await
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn post_graphql_handler(schema: &State<AppSchema>, request: Request) -> Response {
    request.execute(schema).await
}

#[rocket::launch]
async fn rocket() -> _ {
    dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(var) => var,
        Err(_) => panic!("Environment variable \"DATABASE_URL\" not set"),
    };
    let pbkdf2_salt_size = match env::var("PBKDF2_SALT_SIZE") {
        Ok(var) => match var.parse::<usize>() {
            Ok(size) => size,
            Err(_) => panic!("Failed to parse environment variable \"PBKDF2_SALT_SIZE\""),
        },
        Err(_) => panic!("Environment variable \"PBKDF2_SALT_SIZE\" not set"),
    };
    let pbkdf2_iterations = match env::var("PBKDF2_ITERATIONS") {
        Ok(var) => match var.parse::<NonZeroU32>() {
            Ok(iterations) => iterations,
            Err(_) => panic!("Failed to parse environment variable \"PBKDF2_ITERATIONS\""),
        },
        Err(_) => panic!("Environment variable \"PBKDF2_ITERATIONS\" not set"),
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    rocket::build()
        .manage(
            Schema::build(
                QueryRoot::default(),
                MutationRoot::default(),
                EmptySubscription,
            )
            .data(AppContext {
                pool,
                config: AppConfig {
                    pbkdf2_salt_size,
                    pbkdf2_iterations,
                },
            })
            .finish(),
        )
        .mount(
            "/",
            rocket::routes![get_graphql_handler, post_graphql_handler],
        )
}
