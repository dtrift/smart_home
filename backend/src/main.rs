//! Backend entry point: listens on `0.0.0.0:3000`, serves API and static assets.

use std::net::SocketAddr;

use backend::{app, demo_state};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let state = demo_state();
    let app = app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("backend listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
