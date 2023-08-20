use std::process::ExitCode;

mod config;
use self::config::Config;

mod app;

/// A future that returns when the server receives a shutdown signal.
async fn graceful_shutdown() {
    match tokio::signal::ctrl_c().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("failed to receive graceful shutdown: {}", err);
            eprintln!("forced shutdown will crash the server");

            let unreachable = std::future::pending::<std::convert::Infallible>().await;
            match unreachable {}
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("failed to load config: {}", err);
            return ExitCode::FAILURE;
        }
    };

    println!("binding to address: {}", config.address);

    let app = self::app::app();

    if let Err(err) = axum::Server::bind(&config.address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(graceful_shutdown())
        .await
    {
        eprintln!("server error: {}", err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
