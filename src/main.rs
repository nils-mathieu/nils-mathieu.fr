use std::panic::PanicInfo;
use std::process::ExitCode;

mod config;
use self::config::Config;

mod app;

/// A future that returns when the server receives a shutdown signal.
async fn graceful_shutdown() {
    match tokio::signal::ctrl_c().await {
        Ok(_) => (),
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

    // Start the HTTP server.
    tracing::info!("binding to address: {}", config.address);
    if let Err(err) = axum::Server::bind(&config.address)
        .serve(self::app::app().into_make_service())
        .with_graceful_shutdown(graceful_shutdown())
        .await
    {
        tracing::error!("server error: {}", err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
