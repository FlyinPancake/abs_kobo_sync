mod abs_client;
mod config;
mod kobo_api;

use std::{path::Path, sync::Arc};

use abs_client::AbsClient;
use config::Config;
use poem::{EndpointExt, Route, Server, listener::TcpListener, middleware::Cors};
use poem_openapi::OpenApiService;

type AbsKoboResult<T> = anyhow::Result<T>;

#[tokio::main]
async fn main() -> AbsKoboResult<()> {
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
    let client = AbsClient::new(&config.abs_base_url)?.with_api_key(&config.abs_api_key);

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
    run_poem(Arc::new(client)).await?;
    Ok(())
}

pub async fn run_poem(client: Arc<AbsClient>) -> AbsKoboResult<()> {
    let version = env!("CARGO_PKG_VERSION");
    let api = kobo_api::AbsKoboApi { client };
    let api_service =
        OpenApiService::new(api, "ABS Kobo API", version).server("http://localhost:3000");
    let ui = api_service.rapidoc();
    let spec = api_service.spec();
    let route = Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
        .nest("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
        .with(Cors::new());

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(route)
        .await?;
    Ok(())
}
