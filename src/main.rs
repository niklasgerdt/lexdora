use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use std::net::SocketAddr;
use tracing::info;
use DORAEU::{Cli, Commands, api, db};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cli = Cli::parse();
    let database_url = if let Some(url) = cli.database_url {
        url
    } else {
        std::env::var("DATABASE_URL").context("DATABASE_URL not provided (flag --database-url or env var)")?
    };
    let pool = db::db_pool(&database_url).await?;

    match cli.command {
        Commands::Org { cmd } => DORAEU::cli::handlers::org_handlers(&pool, cmd).await?,
        Commands::User { cmd } => DORAEU::cli::handlers::user_handlers(&pool, cmd).await?,
        Commands::Incident { cmd } => DORAEU::cli::handlers::incident_handlers(&pool, cmd).await?,
        Commands::Tpp { cmd } => DORAEU::cli::handlers::tpp_handlers(&pool, cmd).await?,
        Commands::Serve { bind } => {
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .init();
            let addr: SocketAddr = bind
                .unwrap_or_else(|| "0.0.0.0:8080".to_string())
                .parse()
                .with_context(|| "Invalid --bind address, use host:port")?;
            let app = api::build_router(pool.clone());
            info!(%addr, "Starting server");
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app)
                .with_graceful_shutdown(api::shutdown_signal())
                .await
                .context("Server error")?;
        }
    }

    Ok(())
}
