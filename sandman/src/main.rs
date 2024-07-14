mod args;
mod backup;
mod sandman;
mod sha;

#[tokio::main]
async fn main() {
    let _ = sandman::run_sandman().await;
}
