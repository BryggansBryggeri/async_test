use tokio::join;
use structopt::StructOpt;
use std::path::PathBuf;

mod event_handler;
use event_handler::EventHandler;

mod pub_sub;
use pub_sub::{NatsClient, Config};
use pub_sub::PubSub;

//mod log;
//use log::Log;

//mod ticker;
//use ticker::Ticker;

async fn run() -> Result<(), String>{
    let opt = Opt::from_args();
    match opt {
        Opt::Run { config_file } => {
            let config = Config::new(&config_file)?;
            let event_handler = EventHandler::new()?;
            //let log = Log::new(&config).await;
            //let ticker = Ticker::new(&config).await;
            //let ticker_loop = ticker.client_loop();
            //let log_loop = log.client_loop();
            //join!(ticker_loop, log_loop);
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
