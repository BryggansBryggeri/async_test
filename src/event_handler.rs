use futures::stream::{FuturesUnordered, StreamExt};
use async_trait::async_trait;
use thiserror::Error;
use std::future::Future;

#[derive(Error, Debug)]
pub enum EventHandlerError {
    #[error("'{0}' is not an active client")]
    Io(#[from] std::io::Error),
    #[error("No Events: {0}")]
    NoEvents(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl std::convert::From<EventHandlerError> for String {
    fn from(err: EventHandlerError) -> Self {
        err.to_string()
    }
}

pub struct Event<T> {
    future: dyn Future<T>,
    cb: fn(future: dyn Future<T>),
}

pub struct EventHandler {
    events: FuturesUnordered<Event>
}

impl EventHandler {
    pub async fn new() -> Result<EventClient, EventHandlerError> {
        EventHandler {
            events: FuturesUnordered::new()
        }
    }

    pub async fn run(self) -> Result<(), EventHandlerError> {
        loop {
            match self.events.poll_next().await {
                Some(result) => {
                    result.cb(result.future);
                }
                None => {
                    return Err(EventHandlerError::NoEvents)
                }
            }
        }
    }

    pub async fn register(self, event: Event) -> Result<(), EventHandlerError> {
        self.events.push(event);
    }
}
