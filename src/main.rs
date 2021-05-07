use async_nats::{Connection, Options, Subscription};
use async_trait::async_trait;
use tokio::{join};
use tokio::time::{self, Duration};
use futures::{future, pin_mut};

struct Log {
    nats: NatsClient,
}

#[async_trait]
impl PubSub for Log {
    async fn client_loop(self) {
        // A msg on the 'tick' subject is received every 5 seconds.
        let mut ticking_subj = self.nats.subscribe("tick").await;
        // The 'spor_1' subject receives messages sporadically, unpredictable.
        let mut sporadic_subj_1 = self.nats.subscribe("spor_1").await;
        let mut tick = ticking_subj.next();
        let mut spor_1 = sporadic_subj_1.next();
        loop {
            let current = future::select(tick, spor_1).await;
            match current {
                future::Either::Left((spor_val,_)) => {
                    println!("handling spor_1");
                    spor_1 = sporadic_subj_1.next();
                },
                future::Either::Left((tick_val,_)) => {
                    println!("handling tick");
                    tick = ticking_subj.next();
                },
            }
        }
    }
}

struct Ticker {
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
}

#[async_trait]
trait PubSub {
    async fn client_loop(self);
}

#[derive(Clone)]
pub struct NatsClient(Connection);
impl NatsClient {
    pub async fn new() -> NatsClient {
        let opts = Options::new();
        NatsClient(opts.connect("localhost").await.expect("Connect err"))
    }
    pub async fn subscribe(&self, subject: &str) -> Subscription {
        self.0.subscribe(&subject).await.expect("Sub")
    }

    pub async fn publish(&self, subject: &str, msg: &str) {
        self.0.publish(&subject, &msg).await.expect("Pub")
    }
}

#[tokio::main]
async fn main() {
    let log = Log {nats: NatsClient::new().await};
    let ticker = Ticker {nats: NatsClient::new().await};
    let ticker_loop = ticker.client_loop();
    let log_loop = log.client_loop();
    join!(ticker_loop, log_loop);
}
