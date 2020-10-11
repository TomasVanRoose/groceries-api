mod database;
mod filters;
mod handlers;
mod models;

use crate::database::Db;

use dotenv::dotenv;
use pretty_env_logger;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "debug");
    }
    let database_uri = env::var("DATABASE_URL")?;

    pretty_env_logger::init();

    let db = Db::initialize(&database_uri).await?;
    let api = filters::grocery_items(db);

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
