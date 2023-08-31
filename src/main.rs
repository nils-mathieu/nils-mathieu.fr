use std::panic::PanicInfo;
use std::process::ExitCode;

mod config;
use axum_server::tls_rustls::RustlsConfig;

use self::config::Config;

mod app;

/// A future that updates the [`axum_server::Handle`] when a graceful shutdown is requested.
async fn graceful_shutdown_task(handle: axum_server::Handle) {
    match tokio::signal::ctrl_c().await {
        Ok(_) => {
            tracing::info!("received CTRL+C signal, shutting down the server gracefully");
            handle.graceful_shutdown(None);
        }
        Err(err) => {
            tracing::error!("failed to receive graceful shutdown: {}", err);
            tracing::error!("forced shutdown will crash the server");

            let unreachable = std::future::pending::<std::convert::Infallible>().await;
            match unreachable {}
        }
    }
}

/// This function will be called when something in our code panics.
fn panic_hook(info: &PanicInfo) {
    tracing::error!("panic: {info}");
}

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize the logger and panic hook.
    std::panic::set_hook(Box::new(panic_hook));
    if let Err(err) = tracing_subscriber::fmt()
        .with_ansi(true)
        .without_time()
        .with_target(false)
        .try_init()
    {
        use std::io::Write;

        let stderr = std::io::stderr();
        let mut stderr = stderr.lock();
        let _ = writeln!(stderr, "warning: failed to initialize the logger: {}", err);
    }

    // Load the configuration file.
    let config = match Config::try_load() {
        Ok(config) => config,
        Err(err) => {
            tracing::error!("failed to load 'config.ron': {}", err);
            return ExitCode::FAILURE;
        }
    };

    // Load the SSL certificate and key.
    let ssl_config = match RustlsConfig::from_pem_file(&config.ssl_cert, &config.ssl_key).await {
        Ok(config) => config,
        Err(err) => {
            tracing::error!("failed to load SSL certificate and key: {}", err);
            return ExitCode::FAILURE;
        }
    };

    let http_config = axum_server::HttpConfig::new().http2_only(true).build();

    // Create and setup the handle that will be used to control the server.
    let handle = axum_server::Handle::new();
    tokio::spawn(graceful_shutdown_task(handle.clone()));

    // Start the HTTP server.
    tracing::info!("binding to address: {}", config.address);
    if let Err(err) = axum_server::bind_rustls(config.address, ssl_config)
        .http_config(http_config)
        .handle(handle)
        .serve(self::app::app().into_make_service())
        .await
    {
        tracing::error!("server error: {}", err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
