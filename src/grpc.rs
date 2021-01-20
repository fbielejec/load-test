use clap::{Arg, App};
use hdrhistogram::Histogram;
use log::{debug, info};
use std::env;
use std::error::Error;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use tokio::runtime::Runtime;
use tokio::task;
use tonic::transport::channel::Endpoint;
use tonic::{Request, Response, Status};
use hdrhistogram::iterators::IterationValue;
use url::Url;

mod utils;

pub mod api {
    tonic::include_proto!("pingpong");
}

use api::{Ping};
use api::ping_pong_client::PingPongClient;

type MainResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
struct Config {
    url: String,
    n_connections: usize,
    verbosity: String
}

#[tokio::main]
async fn main()
        -> Result<(), anyhow::Error>
{

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

    let config : Config = Config {
        url: match matches.value_of("url") {
            None => panic!("Unknown url argument"),
            Some(url) => //Endpoint::from_static (url)
            url.to_string ()
            // {
            //     match Url::parse(url) {
            //         Ok(url) => url,
            //         Err(_) => panic!("Could not parse url"),
            //     }
            // }
        },
        n_connections: match matches.value_of("connections") {
            None => 1000,
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

    let h = Histogram::<u64>::new_with_bounds(1, 60000, 3).unwrap();
    let hist = Arc::new (Mutex::new (h));

    let mut tasks = Vec::with_capacity(config.n_connections);

    let tick = Instant::now();

    for id in 0..config.n_connections {
        let config = config.clone ();
        let hist = hist.clone ();
        tasks.push(tokio::spawn(async move {
            client(&id, &config, hist).await;
        }));
    }

    for t in tasks {
        t.await?;
    }

    let tock = tick.elapsed().as_millis();

    // print statistics
    // TODO : collect responses

    let mut i = 0;
    let hist = hist.lock().unwrap();

    println!("Summary:\n Count: {}\n Total: {} ms\n Slowest: {} ms\n Fastest: {} ms\n Average: {:.1} ms\n Throughput: {:.1} request/s",
             hist.count (),
             // sum,
             tock,
             hist.max (),
             hist.min (),
             hist.mean (),
             1000.0 * hist.count () as f64 / tock as f64
    );

    println!("Latency cumulative distribution:\n  5 % ≤ {:.2} ms\n 10 % ≤ {:.2} ms\n 50 % ≤ {:.2} ms\n 95 % ≤ {:.2} ms\n 99 % ≤ {:.2} ms \n 100% ≤ {:.2} ms",
             hist.value_at_quantile (0.05f64),
             hist.value_at_quantile (0.1f64),
             hist.value_at_quantile (0.5f64),
             hist.value_at_quantile (0.95f64),
             hist.value_at_quantile (0.95f64),
             hist.value_at_quantile (1.0f64)
    );

    // TODO : print latency distribution (histogram)



    Ok(())
}

async fn client (client_id: &usize,
                 config : &Config,
                 hist : Arc<Mutex<Histogram::<u64>>>) {

    let Config { url, .. } = config;

    let address : Endpoint = Endpoint::from_static ("http://127.0.0.1:3001") ;
    // let address : Endpoint = Endpoint::from_static (url) ;
    let mut client = PingPongClient::connect(address).await.unwrap ();

    let mut start_time = Instant::now();
    let response = client.send_ping(Request::new(Ping {})).await.unwrap ();
    let response_time = start_time.elapsed().as_millis();

    // TODO : store response status

    // threads should not fail while holding the lock.
    let mut hist = hist.lock().unwrap();
    *hist += response_time as u64;

    if ((client_id % 100) == 0) {
        info!("client id: {} response: {:#?} time: {}", client_id, response, response_time);
    }
}
