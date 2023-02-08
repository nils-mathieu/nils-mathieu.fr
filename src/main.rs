use rouille::Response;

const WHITELIST: &[(&str, &str, &str)] = &[
    ("/favicon.ico", "www/favicon.ico", "image/ico"),
    ("/cv/discord.png", "www/cv/discord.png", "image/png"),
    ("/cv/docker.png", "www/cv/docker.png", "image/png"),
    ("/cv/git.png", "www/cv/git.png", "image/png"),
    ("/cv/github.png", "www/cv/github.png", "image/png"),
    ("/cv/linux.png", "www/cv/linux.png", "image/png"),
    ("/cv/spotify.png", "www/cv/spotify.png", "image/png"),
    ("/cv/windows.png", "www/cv/windows.png", "image/png"),
    ("/cv/photo.jpg", "www/cv/photo.jpg", "image/jpeg"),
    ("/cv/", "www/cv/index.html", "text/html"),
];

/// Routes the provided URI to a static file path.
fn route(uri: &str) -> Option<(&'static str, &'static str)> {
    WHITELIST
        .iter()
        .find(|&&(route, _, _)| uri == route)
        .map(|&(_, path, mime)| (path, mime))
}

fn main() {
    let socket_addr =
        std::env::var("SERVER_ADDR").expect("`SERVER_ADDR` not found in the environment");

    rouille::start_server(socket_addr, |req| {
        if let Some((path, mime)) = route(req.raw_url()) {
            let file = rouille::try_or_404!(std::fs::File::open(path));
            Response::from_file(mime, file)
        } else {
            Response::text("not found").with_status_code(404)
        }
    });
}
