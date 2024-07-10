mod backup;
mod sandman;
mod sha;

/*
    TO DO: Add config file parsing
 */
#[tokio::main]
async fn main() {
    let _ = sandman::run_sandman().await;
}
