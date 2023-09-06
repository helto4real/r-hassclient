use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::home_assistant::model::HaEvent;
pub struct HaEventStream {
    ch: (Sender<HaEvent>, Receiver<HaEvent>)
}

impl HaEventStream {
    pub fn new() -> HaEventStream {
        HaEventStream {
            ch: mpsc::channel(100),
        }
    }
    pub fn sender_clone(&self) -> Sender<HaEvent> {
        self.ch.0.clone()
    }

    pub async fn next_async(&mut self) -> Option<HaEvent> {

        if let Some(msg) = self.ch.1.recv().await {
            return Some(msg);
        }
        None
    }
}

