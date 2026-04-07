use std::time::Duration;

mod db;
mod http;
mod todo;

pub const COMPILE_CURRENT_TIME: &str = env!("COMPILE_CURRENT_TIME");
pub const ENDPOINT_DELAY: Duration = Duration::ZERO;

#[tokio::main]
async fn main() -> Result<(), String> {
    let db = db::connect()?;

    Ok(http::start(db).await)
}
