use std::net::{SocketAddr, ToSocketAddrs};
use std::process::ExitCode;

use tiny_http::{ConfigListenAddr, Header, Response, Server, ServerConfig, SslConfig, StatusCode};

/// Describes how to respond to a specific request.
pub enum Route {
    /// Respond with a file.
    File {
        /// The path to the file in the file system.
        path: &'static str,
        /// The content type of the file.
        content_type: &'static str,
    },
    /// Request a redirection.
    Redirect(&'static str),
    /// A simple status code.
    NotFound,
}

/// Routes the provided URI to a static file path.
fn route(uri: &str) -> Route {
    match uri {
        "/favicon.ico" => Route::File {
            path: "www/favicon.ico",
            content_type: "image/ico",
        },
        "/cv/discord.png" => Route::File {
            path: "www/cv/discord.png",
            content_type: "image/png",
        },
        "/cv/docker.png" => Route::File {
            path: "www/cv/docker.png",
            content_type: "image/png",
        },
        "/cv/git.png" => Route::File {
            path: "www/cv/git.png",
            content_type: "image/png",
        },
        "/cv/github.png" => Route::File {
            path: "www/cv/github.png",
            content_type: "image/png",
        },
        "/cv/linux.png" => Route::File {
            path: "www/cv/linux.png",
            content_type: "image/png",
        },
        "/cv/spotify.png" => Route::File {
            path: "www/cv/spotify.png",
            content_type: "image/png",
        },
        "/cv/windows.png" => Route::File {
            path: "www/cv/windows.png",
            content_type: "image/png",
        },
        "/cv/photo.jpg" => Route::File {
            path: "www/cv/photo.jpg",
            content_type: "image/jpeg",
        },
        "/cv/notion.png" => Route::File {
            path: "www/cv/notion.png",
            content_type: "image/png",
        },
        "/cv/man-thinking.png" => Route::File {
            path: "www/cv/man-thinking.png",
            content_type: "image/png",
        },
        "/cv/presentation.png" => Route::File {
            path: "www/cv/presentation.png",
            content_type: "image/png",
        },
        "/cv/tel.png" => Route::File {
            path: "www/cv/tel.png",
            content_type: "image/png",
        },
        "/cv/mail.png" => Route::File {
            path: "www/cv/mail.png",
            content_type: "image/png",
        },
        "/cv/dl.png" => Route::File {
            path: "www/cv/dl.png",
            content_type: "image/png",
        },
        "/cv/" => Route::File {
            path: "www/cv/index.html",
            content_type: "text/html",
        },
        "/cv" => Route::Redirect("/cv/"),
        _ => Route::NotFound,
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
            Route::File { path, content_type } => {
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
                        .with_header(Header::from_bytes("Content-Type", content_type).unwrap()),
                ) {
                    eprintln!("error: failed to respond: {err}");
                }
            }
            Route::Redirect(to) => {
                let response =
                    Response::empty(301).with_header(Header::from_bytes("Location", to).unwrap());
                if let Err(err) = req.respond(response) {
                    eprintln!("error: failed to respond: {err}");
                }
            }
            Route::NotFound => {
                if let Err(err) = req.respond(Response::empty(StatusCode(404))) {
                    eprintln!("error: failed to respond: {err}");
                }
            }
        }
    }

    ExitCode::SUCCESS
}
