
mod backup;
mod config;
mod sandman;
mod sha;

#[tokio::main]
async fn main() {
    let _ = sandman::run_sandman().await;
}
