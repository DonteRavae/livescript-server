use std::{error::Error, net::SocketAddr};

use axum::{
    routing::{get, post},
    Router,
};
use livescript::{self, ApplicationState};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    livescript::welcome();

    let app = Router::new()
        .route("/auth/register", post(livescript::register_user))
        .route("/auth/login", post(livescript::login_user))
        .route("/auth/logout", get(livescript::logout_user))
        .route("/auth/refresh", get(livescript::refresh_user))
        .route("/broadcast/init", get(livescript::init_broadcast))
        .route(
            "/broadcast/subscribe",
            get(livescript::subscribe_to_broadcast),
        )
        .with_state(ApplicationState::init().await)
        .layer(CookieManagerLayer::new());

    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
