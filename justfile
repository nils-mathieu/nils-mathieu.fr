set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

run:
    git pull
    cargo run --release
