use futures::prelude::*;
use async_std::task;
// use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::env;
use log::{debug, info};
// use tokio::sync::mpsc;
use clap::{Arg, App};
use std::error::Error;
use url::Url;
// use tonic::{transport::Server, Request, Response, Status};

pub mod api {
    tonic::include_proto!("pingpong");
}

use api::{Ping};
use api::ping_pong_client::PingPongClient;

type MainResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
struct Config {
    url: Url,
    n_connections: usize,
    verbosity: String
}

// #[tokio::main]
fn main() -> MainResult<()> {

    let matches = App::new("ws-load-test")
        .version("0.1.0")
        .author("filip bielejec <fbielejec@gmail.com>")
        .about("high-throughput tool for testing gRPC APIs")
        .arg(Arg::with_name("verbosity")
             .short("v")
             .long("verbosity")
             .takes_value(true)
             .help("verbosity level : debug | info | warn | error"))
        .arg(Arg::with_name("url")
             .short("u")
             .long("url")
             .takes_value(true)
             .help("the URL of the gRPC endpoint"))
        .arg(Arg::with_name("connections")
             .short("c")
             .long("connections")
             .takes_value(true)
             .help("the number of concurrent requests to make"))
        .get_matches();

    let config = Config {
        url: match matches.value_of("url") {
            None => panic!("Unknown url argument"),
            Some(url) => {
                match Url::parse(url) {
                    Ok(url) => url,
                    Err(_) => panic!("Could not parse url"),
                }
            }
        },
        n_connections: match matches.value_of("connections") {
            None => 1,
            Some(c) => {
                match c.parse::<usize> () {
                    Ok (n) => n,
                    Err (_) => panic!("Wrong number of connections specified {}", c)
                }
            }
        },
        verbosity: match matches.value_of("verbose?") {
            None => "info".to_string (),
            Some (v) => {
                match v.parse::<bool> ().expect ("Could not parse verbosity argument") {
                    true => "debug".to_string (),
                    false => "info".to_string ()
                }
            }
        }
    };


    env::set_var("RUST_LOG", &config.verbosity);
    env_logger::init();
    info!("Running with config {:#?}", &config);

    // let mut tasks = Vec::with_capacity(config.n_connections);

        task::spawn(async move {
            client(&config).await;
        });

// TODO : make N concurrent requests and collect response times



    Ok(())
}

async fn client(
                config : &Config,
) -> Result<(), anyhow::Error> {

        let address = "http://127.0.0.1:3001";//.parse ().unwrap ();
    //format!("{}:{}", &config.url.host ().unwrap (), &config.url.port ().unwrap ()).parse().unwrap();
    let mut client = PingPongClient::connect(address).await?;

Ok (())
}
