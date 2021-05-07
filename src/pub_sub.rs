use async_nats::{Connection, Options};
use async_trait::async_trait;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::future::Future;

mod event_handler;
use event_handler::EventHandler;
use event_handler::Event;

#[async_trait]
pub trait PubSub{
    async fn client_loop(self);
    async fn new(config: &Config) -> Self;
}

#[derive(Clone)]
pub struct NatsClient {
    connection: Connection,
    handler: EventHandler,
}

impl NatsClient {
    pub async fn new(handler: EventHandler, config: &Config) -> NatsClient {
        let opts = Options::with_user_pass(&config.user, &config.pass);
        NatsClient {
            connection: Connection(opts.connect(&config.server)).await.expect("Connect err"),
            handler: handler,
        }
    }

    pub async fn publish(&self, subject: &str, msg: &str) {
        self.connection.publish(&subject, &msg).await.expect("Pub")
    }

    pub async fn subscribe(&self, subject: &str, cb: fn(msg: &str)) {
        self.handler.register(
            Event{ 
                future: self.connection.subscribe(&subject).await.expect("Sub").await,
                cb: cb,
            }
        );
    }
}

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


