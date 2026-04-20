use std::time::Duration;

mod db;
mod http;
mod sse;
mod todo;

pub const COMPILE_CURRENT_TIME: &str = env!("COMPILE_CURRENT_TIME");
pub const ENDPOINT_DELAY: Duration = Duration::ZERO;

#[tokio::main]
async fn main() -> Result<(), String> {
    let db = db::connect()?;
    let broadcaster = sse::Broadcaster::new();

    Ok(http::start(db, broadcaster).await)
}
