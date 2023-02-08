use std::net::{SocketAddr, ToSocketAddrs};
use std::process::ExitCode;

use tiny_http::{ConfigListenAddr, Header, Response, Server, ServerConfig, SslConfig, StatusCode};

/// Routes the provided URI to a static file path.
fn route(mut uri: &str) -> Option<(&'static str, &'static str)> {
    uri = match uri {
        "/cv" => "/cv/",
        _ => uri,
    };

    match uri {
        "/favicon.ico" => Some(("www/favicon.ico", "image/ico")),
        "/cv/discord.png" => Some(("www/cv/discord.png", "image/png")),
        "/cv/docker.png" => Some(("www/cv/docker.png", "image/png")),
        "/cv/git.png" => Some(("www/cv/git.png", "image/png")),
        "/cv/github.png" => Some(("www/cv/github.png", "image/png")),
        "/cv/linux.png" => Some(("www/cv/linux.png", "image/png")),
        "/cv/spotify.png" => Some(("www/cv/spotify.png", "image/png")),
        "/cv/windows.png" => Some(("www/cv/windows.png", "image/png")),
        "/cv/photo.jpg" => Some(("www/cv/photo.jpg", "image/jpeg")),
        "/cv/" => Some(("www/cv/index.html", "text/html")),
        _ => None,
    }
}

fn main() -> ExitCode {
    let Ok(socket_addr) = std::env::var("SERVER_ADDRESS") else {
        eprintln!("error: no `SERVER_ADDRESS` in the environment");
        return ExitCode::FAILURE;
    };

    let (addr, port) = if let Some((addr, port)) = socket_addr.split_once(':') {
        let Ok(port) = port.parse::<u16>() else {
            eprintln!("error: {port}: invalid port");
            return ExitCode::FAILURE;
        };

        (addr, port)
    } else {
        (socket_addr.as_str(), 443)
    };

    let addresses: Vec<SocketAddr> = match (addr, port).to_socket_addrs() {
        Ok(iter) => iter.collect(),
        Err(err) => {
            eprintln!("error: {socket_addr}: {err}");
            return ExitCode::FAILURE;
        }
    };

    let Ok(certificate) = std::env::var("SERVER_CERTIFICATE") else {
        eprintln!("error: no `SERVER_CERTIFICATE` in the environment");
        return ExitCode::FAILURE;
    };

    let certificate = match std::fs::read(&certificate) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("error: {certificate}: {err}");
            return ExitCode::FAILURE;
        }
    };

    let Ok(private_key) = std::env::var("SERVER_PRIVATE_KEY") else {
        eprintln!("error: no `SERVER_PRIVATE_KEY` in the environment");
        return ExitCode::FAILURE;
    };

    let private_key = match std::fs::read(&private_key) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("error: {private_key}: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!("listening for {:?}", addresses);

    let config = ServerConfig {
        addr: ConfigListenAddr::IP(addresses),
        ssl: Some(SslConfig {
            certificate,
            private_key,
        }),
    };

    let server = match Server::new(config) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("error: failed to initiate the server: {err}");
            return ExitCode::FAILURE;
        }
    };

    for req in server.incoming_requests() {
        match route(req.url()) {
            Some((path, mime)) => {
                let file = match std::fs::File::open(path) {
                    Ok(ok) => ok,
                    Err(err) => {
                        eprintln!("error: failed to open `{path}`: {err}");
                        if let Err(err) = req.respond(Response::empty(StatusCode(500))) {
                            eprintln!("error: failed to respond: {err}");
                        };
                        continue;
                    }
                };

                if let Err(err) = req.respond(
                    Response::from_file(file)
                        .with_header(Header::from_bytes("Content-Type", mime).unwrap()),
                ) {
                    eprintln!("error: failed to respond: {err}");
                }
            }
            None => {
                if let Err(err) = req.respond(Response::empty(StatusCode(404))) {
                    eprintln!("error: failed to respond: {err}");
                }
            }
        }
    }

    ExitCode::SUCCESS
}
