use anyhow::Context;
use switchboard_backend_api::{build_router, AppState};
use switchboard_backend_runtime::{telemetry, BackendServices};
use switchboard_config::load as load_config;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init_tracing().context("failed to initialise tracing")?;

    info!("starting Switchboard backend");

    let config = load_config().context("failed to load configuration")?;

    let services = BackendServices::initialise(&config)
        .await
        .context("failed to initialise backend services")?;

    let state = AppState::new(services.orchestrator.clone(), services.authenticator.clone());
    let app = build_router(state);

    let address = format!("{}:{}", config.http.address, config.http.port);
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("failed to bind http listener on {address}"))?;

    info!(%address, "http server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(switchboard_backend_runtime::shutdown_signal())
        .await
        .context("http server error")?;

    info!("backend shut down");
    Ok(())
}
