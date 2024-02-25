use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::{channel, Sender};
use uuid::Uuid;

use super::application_state::ApplicationState;

#[allow(non_snake_case, non_upper_case_globals)]
mod BroadcastCommands {
    pub const End: &str = "end";
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

    async fn verify_live(state: &Arc<ApplicationState>, broadcast_id: &String) -> bool {
        let broadcasts = state.live_broadcasts.lock().await;
        if Uuid::parse_str(broadcast_id.as_str()).is_err() {
            return false;
        }

        if !broadcasts.contains_key(&Uuid::parse_str(broadcast_id.as_str()).unwrap()) {
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
            broadcasts.insert(broadcast.id.clone(), broadcast.clone());
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
                let _ = transmitter.send(format!("Message received: {msg}").into());

                match msg.to_lowercase().as_str() {
                    BroadcastCommands::End => break,
                    _ => {}
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
