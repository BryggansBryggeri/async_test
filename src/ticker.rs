use async_trait::async_trait;
use tokio::time::{self, Duration};

use crate::{NatsClient, Config, pub_sub::PubSub};

pub struct Ticker {
    nats: NatsClient,
}

#[async_trait]
impl PubSub for Ticker {
    async fn client_loop(self) {
        let mut interval = time::interval(Duration::from_millis(10000));
        loop {
            interval.tick().await;
            self.nats.publish("tick", "aba").await;
        }
    }

    async fn new(config: &Config) -> Self {
        Ticker {
            nats: NatsClient::new(config).await
        }
    }
}

