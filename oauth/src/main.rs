use async_recursion::async_recursion;
use axum::AddExtensionLayer;
// use axum_server::tls_rustls::RustlsConfig;
use std::env;
use std::net::SocketAddr;
use tokio::sync::mpsc;

use oauth::config::{
    config_get, config_init, set_tnc_filesystem_endpoint, tnc_app_endpoint,
    tnc_authorized_drive_endpoint, tnc_authorized_endpoint, tnc_drive_token_endpoint,
    tnc_register_endpoint,
};

#[tokio::main]
async fn main() {
    set_env();
    if let Err(err) = run().await {
        tracing::error!("Server error: {}", err);
    }
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // initialize once; zero-sized channel
    let (signal, mut shutdown) = mpsc::channel(1);

    // requires RUST_LOG env setting
    tracing_subscriber::fmt::init();

    loop {
        config_init()?;
        tracing::info!("🟢 Starting...");
        // clonable parameters
        start_server(signal.clone(), &mut shutdown).await.ok();
    }
}

#[async_recursion]
async fn start_server(
    signal: mpsc::Sender<()>,
    shutdown: &mut mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    // report out endpoints that interact with the app
    tracing::info!(
        "RUST_ENV: {}",
        env::var("RUST_ENV").unwrap_or_else(|_| "⚠️ not set".to_string())
    );
    tracing::info!(
        "RUST_LOG: {}",
        env::var("RUST_LOG").unwrap_or_else(|_| "⚠️ not set".to_string())
    );
    // returns Err: AuthError::InvalidUrl when cannot form Uri
    tracing::info!("authenticated endpoint {:?}", tnc_authorized_endpoint());
    tracing::info!(
        "authorized drive endpoint {:?}",
        tnc_authorized_drive_endpoint()
    );
    tracing::info!("🙂 register endpoint {:?}", tnc_register_endpoint());
    tracing::info!("tnc app endpoint {:?}", tnc_app_endpoint());
    tracing::info!("🗄️  drive-token endpoint {:?}", tnc_drive_token_endpoint());
    tracing::info!(
        "filesystem endpoint {:?}",
        set_tnc_filesystem_endpoint(None)
    );

    let addr = SocketAddr::from((config_get()?.options.host, config_get()?.options.port));
    tracing::info!("listening on {}", &addr);

    /*
    let certs = RustlsConfig::from_pem_file("oauth/localhost.pem", "oauth/localhost-key.pem")
        .await
        .unwrap();
    */

    // axum_server::bind_rustls(addr, certs)
    axum::Server::bind(&addr)
        .serve(
            oauth::app()
                .map(|router| router.layer(AddExtensionLayer::new(signal.clone())))?
                .into_make_service(),
        )
        .with_graceful_shutdown(async {
            let _stopped: Option<()> = shutdown.recv().await;
            tracing::info!("🔴...stopping");
        })
        .await
        .ok();
    Ok(())
}

// utility for main
fn set_env() {
    // look for rust-log value in env; use config when unset
    if std::env::var_os("RUST_LOG").is_none() {
        let setting: String = match config_get() {
            Ok(cfg) => cfg.options.rust_log.clone(),
            Err(_) => "oauth=info,hyper=info".to_string(),
        };
        env::set_var("RUST_LOG", setting);
    }
}

/*
let reload = async {
    reload
        .recv()
        .await
        .expect("failed to install reload handler");
};
let ctrl_c = async {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
};

#[cfg(unix)]
let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("failed to install signal handler")
        .recv()
        .await;
};

#[cfg(not(unix))]
let terminate = std::future::pending::<()>();

tokio::select! {
    _ = reload => {},
    _ = ctrl_c => {},
    _ = terminate => {},
}
    */
