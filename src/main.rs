use async_nats::{Connection, Options, Subscription};
use async_trait::async_trait;
use futures::{future::join_all};
use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    let supervisor = Supervisor {
        nats: NatsClient::new().await,
    };
    let supervisor_loop = supervisor.client_loop();
    supervisor_loop.await;
}

struct Supervisor {
    nats: NatsClient,
}

#[async_trait]
impl PubSub for Supervisor {
    async fn client_loop(self) {
        let mut clients = Mutex::new(Vec::new());
        let ticker = Ticker {
            nats: NatsClient::new().await,
        };
        let ticker_loop = ticker.client_loop();
        let mut aba = clients.lock().unwrap();
        aba.push(ticker_loop);

        let start_log_subj = self.nats.subscribe("command.start_log").await;
        let log_handle = async {
            loop {
                let _ = start_log_subj.next().await.unwrap();
                let log = Log {
                    nats: NatsClient::new().await,
                };
                println!("Recv starting log");
                let log_loop = log.client_loop();
                let mut aba = clients.lock().unwrap();
                aba.push(log_loop);
            }
        };
        let kill_client_subj = self.nats.subscribe("command.kill_client.*").await;
        let kill_client = async {
            loop {
                let msg = kill_client_subj.next().await.unwrap();
                let id = msg.subject.split('.').last().expect("No id in subj");
                let pub_subj = format!("supervisor.kill_client.{}", id);
                self.nats.publish(&pub_subj, "").await;
            }
        };
        tokio::join!(join_all(clients), kill_client, log_handle);
    }
}

struct Log {
    nats: NatsClient,
}

#[async_trait]
impl PubSub for Log {
    async fn client_loop(self) {
        // A msg on the 'tick' subject is received every 5 seconds.
        let ticking_subj = self.nats.subscribe("tick").await;
        let command_subj = self.nats.subscribe("supervisor.kill_client.log").await;
        let state = ClientState::new();
        let ticking = async {
            while state.is_active() {
                let _tick = ticking_subj.next().await;
                println!("handling tick");
            }
        };

        let command = async {
            let _comm = command_subj.next().await;
            println!("Stopping log client.");
            state.inactivate();
        };
        tokio::join!(ticking, command);
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
