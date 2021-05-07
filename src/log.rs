use tokio::select;
use async_trait::async_trait;

use crate::{NatsClient, Config, pub_sub::PubSub};

pub struct Log {
    nats: NatsClient,
}

#[async_trait]
impl PubSub for Log {
    async fn client_loop(self) {
        // A msg on the 'tick' subject is received every 5 seconds.
        let ticking_subj = self.nats.subscribe("tick").await;
        // The 'spor_1' subject receives messages sporadically, unpredictable.
        let sporadic_subj_1 = self.nats.subscribe("spor_1").await;
        loop {
            // It seems a bit strange for me to call next() here, but I can't find a better place.
            // I don't know what impact calling next() multiple times before actually using the
            // output.
            let tick = ticking_subj.next();
            let spor_1 = sporadic_subj_1.next();
            // I need to pin for it to compile.
            tokio::pin!(tick, spor_1);
            select! {
                _ = &mut spor_1 => {
                    println!("handling spor_1");
                }
                _ = &mut tick => {
                    println!("handling tick");
                }
            }
        }
    }

    async fn new(config: &Config) -> Self {
        Log {
            nats: NatsClient::new(config).await
        }
    }
}
