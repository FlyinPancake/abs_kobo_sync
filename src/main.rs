mod abs_client;
mod config;
mod kobo_api;

use std::{path::Path, sync::Arc};

use abs_client::AbsClient;
use anyhow::Context;
use config::Config;
use migration::MigratorTrait;
use poem::{
    EndpointExt, Route, Server,
    listener::TcpListener,
    middleware::{Cors, Tracing as PoemTracing},
};
use poem_openapi::OpenApiService;
use sea_orm::Database;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt::SubscriberBuilder, prelude::*};

type AbsKoboResult<T> = anyhow::Result<T>;

#[tokio::main]
async fn main() -> AbsKoboResult<()> {
    // Initialize tracing (logs). Respect RUST_LOG if set, default to info for our crate and warn for deps.
    let default_filter = format!(
        "{}=info,poem=info,reqwest=warn,h2=warn",
        env!("CARGO_PKG_NAME")
    );
    let env_filter = std::env::var("RUST_LOG").unwrap_or(default_filter);
    SubscriberBuilder::default()
        .with_env_filter(EnvFilter::new(env_filter))
        .with_target(false)
        .with_level(true)
        .pretty()
        .finish()
        .with(ErrorLayer::default())
        .init();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "starting ABS Kobo Sync"
    );
    // Load environment variables from .env files
    if Path::new(".env.local").exists() {
        dotenvy::from_filename(".env.local")?;
    } else if Path::new(".env").exists() {
        dotenvy::from_filename(".env")?;
    };
    let config = Config::load();
    match config.validate() {
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow::anyhow!(e));
        }
    }

    let db_conn = Database::connect(&config.db_connection_string)
        .await
        .with_context(|| "Failed to connect to database")?;

    migration::Migrator::up(&db_conn, None)
        .await
        .with_context(|| "Failed to run database migrations")?;

    let client = AbsClient::new(&config.abs_base_url)?.with_api_key(&config.abs_api_key);
    let has_api_key = !config.abs_api_key.is_empty();
    tracing::info!(abs_base = %config.abs_base_url, has_api_key, "configured ABS client");

    // let status = client.get_status().await?;

    // eprintln!(
    //     "ABS Version is: {}",
    //     status
    //         .server_version
    //         .context("Failed to get server version")?
    // );

    // let libraries = client.get_libraries().await?;

    // let books_library = libraries
    //     .libraries
    //     .into_iter()
    //     .find(|l| l.name == "Books")
    //     .context("Books library not found")?;

    // let series = client
    //     .get_library_series(&books_library.id, 100, None, None)
    //     .await?;

    // for s in series.results {
    //     eprintln!("  {}", s.name);
    // }
    run_poem(Arc::new(client), Arc::new(config), Arc::new(db_conn)).await?;
    Ok(())
}

pub async fn run_poem(
    client: Arc<AbsClient>,
    config: Arc<Config>,
    db: Arc<sea_orm::DatabaseConnection>,
) -> AbsKoboResult<()> {
    let version = env!("CARGO_PKG_VERSION");
    let api = kobo_api::AbsKoboApi { client, config, db };
    let api_service =
        OpenApiService::new(api, "ABS Kobo API", version).server("http://localhost:3000");
    //.extra_request_header(poem_openapi::ExtraHeader::new("X-Abs-Kobo-Version"))
    let ui = api_service.rapidoc();
    let spec = api_service.spec();
    let route = Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
        .nest("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
        .with(Cors::new())
        .with(PoemTracing);

    let bind_addr = "0.0.0.0:3000";
    tracing::info!(%bind_addr, "starting HTTP server");
    Server::new(TcpListener::bind(bind_addr)).run(route).await?;
    Ok(())
}
