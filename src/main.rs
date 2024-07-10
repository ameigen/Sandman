use serde::Deserialize;

mod backup;
mod sandman;
mod sha;
mod config;

#[tokio::main]
async fn main() {
    let _ = sandman::run_sandman().await;
}
