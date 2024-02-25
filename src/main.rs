use std::{error::Error, net::SocketAddr};

use axum::{routing::get, Router};
use livescript::{self, ApplicationState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    livescript::welcome();

    let app = Router::new()
        .route("/ws/init", get(livescript::init_broadcast))
        .with_state(ApplicationState::init().await);

    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
