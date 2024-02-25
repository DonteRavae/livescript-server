use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use super::broadcast::Broadcast;

#[derive(Debug)]
pub struct ApplicationState {
    pub live_broadcasts: Mutex<HashMap<Uuid, Broadcast>>,
}

impl ApplicationState {
    pub async fn init() -> Arc<ApplicationState> {
        Arc::new(ApplicationState {
            live_broadcasts: Mutex::new(HashMap::new()),
        })
    }
}

