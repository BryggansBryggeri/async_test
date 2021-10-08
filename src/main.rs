use async_nats::{Connection, Options, Subscription};
use async_trait::async_trait;
use tokio::join;
use tokio::time::{self, Duration};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct ClientState(AtomicBool);

impl ClientState {
    fn new() -> Self {
        ClientState(AtomicBool::new(true))
    }
    fn is_active(&self) -> bool {
        self.0.fetch_and(true, Ordering::SeqCst)
    }
    fn inactivate(&self) {
        self.0.fetch_and(false, Ordering::SeqCst);
    }
}

struct Log {
    nats: NatsClient,
}

#[async_trait]
impl PubSub for Log {
    async fn client_loop(mut self) {
        // A msg on the 'tick' subject is received every 5 seconds.
        let ticking_subj = self.nats.subscribe("tick").await;
        // The 'spor_1' subject receives messages sporadically, unpredictable.
        let sporadic_subj_1 = self.nats.subscribe("spor_1").await;
        let state = ClientState::new();
        let ticking = async {
            while state.is_active() {
                let _tick = ticking_subj.next().await;
                println!("handling tick");
            }
        };
        let sporadic = async {
            let _spor = sporadic_subj_1.next().await;
            println!("handling spor_1");
            state.inactivate();
        };
        tokio::join!(ticking, sporadic);
        println!("Exiting loop.");
    }
}

struct Ticker {
    nats: NatsClient,
}

#[async_trait]
impl PubSub for Ticker {
    async fn client_loop(self) {
        let mut interval = time::interval(Duration::from_millis(1000));
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
        self.0.subscribe(subject).await.expect("Sub")
    }

    pub async fn publish(&self, subject: &str, msg: &str) {
        self.0.publish(subject, &msg).await.expect("Pub")
    }
}

#[tokio::main]
async fn main() {
    let log = Log {
        nats: NatsClient::new().await,
    };
    let ticker = Ticker {
        nats: NatsClient::new().await,
    };
    let ticker_loop = ticker.client_loop();
    let log_loop = log.client_loop();
    join!(ticker_loop, log_loop);
}
