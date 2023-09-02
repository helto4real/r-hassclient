use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct HaEventStream {
    stream: (Sender<String>, Receiver<String>),
}

impl HaEventStream {
    pub fn new() -> HaEventStream {
        HaEventStream {
            stream: mpsc::channel(100),
        }
    }
    pub fn sender_clone(&self) -> Sender<String> {
        self.stream.0.clone()
    }
}
