use std::{net::SocketAddr, sync::Arc};

use axum::{extract::{ConnectInfo, State, WebSocketUpgrade}, response::IntoResponse};
use axum_extra::TypedHeader;

use crate::{types::Broadcast, ApplicationState};

fn log_user_agent(user_agent: Option<TypedHeader<headers::UserAgent>>, addr: SocketAddr) {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    println!("`{user_agent}` at {addr} connected.");
}

pub async fn init_broadcast(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<ApplicationState>>,
) -> impl IntoResponse {
    log_user_agent(user_agent, addr);
    ws.on_upgrade(move |socket| Broadcast::init(socket, addr, state))
}

pub async fn subscribe_to_broadcast(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<ApplicationState>>,
) -> impl IntoResponse {
    log_user_agent(user_agent, addr);
    ws.on_upgrade(move |socket| Broadcast::subscribe(socket, addr, state))
}