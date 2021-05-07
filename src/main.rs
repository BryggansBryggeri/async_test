use async_nats::{Connection, Options, Subscription};
use async_trait::async_trait;
use tokio::{join, select};
use tokio::time::{self, Duration};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("'{0}' is not an active client")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl std::convert::From<ConfigError> for String {
    fn from(err: ConfigError) -> Self {
        err.to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    server: String,
    user: String,
    pass: String,
}

impl Config {
    pub fn new(config_file: &Path) -> Result<Config, ConfigError> {
        let mut f = match fs::File::open(config_file) {
            Ok(f) => f,
            Err(err) => return Err(ConfigError::Io(err)),
        };
        let mut config_string = String::new();
        match f.read_to_string(&mut config_string) {
            Ok(_) => {}
            Err(err) => return Err(ConfigError::Io(err)),
        };
        let conf_presumptive = serde_json::from_str(&config_string)
            .map_err(|err| ConfigError::Parse(err.to_string()))?;
        Config::validate(conf_presumptive)
    }

    fn validate(pres: Config) -> Result<Config, ConfigError> {
        Ok(pres)
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
    pub async fn new(config: &Config) -> NatsClient {
        let opts = Options::with_user_pass(&config.user, &config.pass);
        NatsClient(opts.connect(&config.server).await.expect("Connect err"))
    }
    pub async fn subscribe(&self, subject: &str) -> Subscription {
        self.0.subscribe(&subject).await.expect("Sub")
    }

    pub async fn publish(&self, subject: &str, msg: &str) {
        self.0.publish(&subject, &msg).await.expect("Pub")
    }
}

async fn run() -> Result<(), String>{
    let opt = Opt::from_args();
    match opt {
        Opt::Run { config_file } => {
            let config = Config::new(&config_file)?;
            let log = Log {nats: NatsClient::new(&config).await};
            let ticker = Ticker {nats: NatsClient::new(&config).await};
            let ticker_loop = ticker.client_loop();
            let log_loop = log.client_loop();
            join!(ticker_loop, log_loop);
        }
    };

    Err("Foo".to_string())
}

#[tokio::main]
async fn main(){
    match run().await {
        Ok(_) => {},
        Err(err) => println!("{}", err)
    }
}



#[derive(Debug, StructOpt)]
#[structopt(name = "async_test")]
pub enum Opt {
    ///Run supervisor
    #[structopt(name = "run")]
    Run { config_file: PathBuf },
}
