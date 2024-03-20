use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::{channel, Sender};
use uuid::Uuid;

use super::application_state::ApplicationState;

#[allow(non_snake_case, non_upper_case_globals)]
mod BroadcastCommands {
    pub const ScrollSpeed1: &str = "scroll:speed_1";
    pub const ScrollSpeed2: &str = "scroll:speed_2";
    pub const ScrollSpeed3: &str = "scroll:speed_3";
    pub const ScrollSpeed4: &str = "scroll:speed_4";
    pub const ScrollSpeed5: &str = "scroll:speed_5";
    pub const Scroll: &str = "scroll";
    pub const OneMinute: &str = "timing:one_minute";
    pub const ThirtySeconds: &str = "timing:thirty_seconds";
    pub const Wrap: &str = "timing:wrap";
    pub const HardWrap: &str = "timing:hard_wrap";
    pub const ResetTiming: &str = "timing:reset";
    pub const End: &str = "state:end";
}

#[derive(Debug, Clone)]
pub struct Broadcast {
    pub id: Uuid,
    pub subs: HashSet<SocketAddr>,
    pub transmitter: Sender<String>,
}

impl Broadcast {
    fn new(initial_subscriber: SocketAddr) -> Self {
        let mut broadcast = Self {
            id: Uuid::new_v4(),
            subs: HashSet::new(),
            transmitter: channel(11).0,
        };
        broadcast.subs.insert(initial_subscriber);
        broadcast
    }

    async fn verify_live(state: &Arc<ApplicationState>, broadcast_id: &str) -> bool {
        let broadcasts = state.live_broadcasts.lock().await;
        if Uuid::parse_str(broadcast_id).is_err() {
            return false;
        }

        if !broadcasts.contains_key(&Uuid::parse_str(broadcast_id).unwrap()) {
            return false;
        }
        true
    }

    pub async fn init(socket: WebSocket, who: SocketAddr, state: Arc<ApplicationState>) {
        let (mut client_sender, mut client_receiver) = socket.split();

        // Create new broadcast and subscribe to broadcast transmitter
        let broadcast = Self::new(who);
        let mut receiver = broadcast.transmitter.subscribe();

        {
            // Add new broadcast to table of live broadcasts then alert client
            let mut broadcasts = state.live_broadcasts.lock().await;
            broadcasts.insert(broadcast.id, broadcast.clone());
            let _ = broadcast
                .transmitter
                .send(format!("Broadcast {} started. {who} joined.", broadcast.id));
        }

        // Receive messages from Broadcast and send message to client
        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                // Break loop for any websocket error
                if client_sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        // Receive message from client and send to broadcast subscribers
        let transmitter = broadcast.transmitter.clone();

        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(Message::Text(msg))) = client_receiver.next().await {
                match msg.to_lowercase().as_str() {
                    BroadcastCommands::Scroll
                    | BroadcastCommands::ScrollSpeed1
                    | BroadcastCommands::ScrollSpeed2
                    | BroadcastCommands::ScrollSpeed3
                    | BroadcastCommands::ScrollSpeed4
                    | BroadcastCommands::ScrollSpeed5
                    | BroadcastCommands::OneMinute
                    | BroadcastCommands::ThirtySeconds
                    | BroadcastCommands::Wrap
                    | BroadcastCommands::HardWrap
                    | BroadcastCommands::ResetTiming => {
                        let _ = transmitter.send(msg.to_string());
                    }
                    BroadcastCommands::End => break,
                    _ => {
                        let _ = transmitter.send("Invalid message".to_string());
                    }
                }
            }
        });

        // If one task ends, the other is aborted
        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        };

        println!("Websocket context {who} destroyed");
    }

    pub async fn subscribe(socket: WebSocket, who: SocketAddr, state: Arc<ApplicationState>) {
        let (mut client_sender, mut client_receiver) = socket.split();
        let mut broadcast_id = String::new();

        // Verify broadcast is live with given id
        while let Some(Ok(message)) = client_receiver.next().await {
            if let Message::Text(id) = message {
                if !Self::verify_live(&state, &id).await {
                    let _ = client_sender
                        .send(Message::Text(String::from("Broadcast doesn't exist!")))
                        .await;
                    return;
                }
                broadcast_id.push_str(&id);
                break;
            }
        }

        // Subscribe client to live broadcast
        let mut live_broadcasts = state.live_broadcasts.lock().await;
        let broadcast = live_broadcasts
            .get_mut(&Uuid::parse_str(broadcast_id.as_str()).unwrap())
            .unwrap();
        let mut receiver = broadcast.transmitter.subscribe();
        broadcast.subs.insert(who);

        // Receive messages from Broadcast and send message to client
        tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                // Break loop for any websocket error
                if client_sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });
    }
}
