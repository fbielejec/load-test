use async_std::task;
use async_tungstenite::async_std::connect_async;
use async_tungstenite::tungstenite::protocol::Message;
use futures::StreamExt;
use futures::prelude::*;
use hdrhistogram::Histogram;
use log::{debug, info};
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use url::Url;
use clap::{Arg, App};

type MainResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
#[derive(Clone)]
struct Config {
    gateway_url: Url,
    n_connections: usize,
    verbosity: String
}

// cargo run -- -v debug -c 3 -g ws://echo.websocket.org
fn main() -> MainResult<()> {

    let matches = App::new("ws-load-test")
        .version("0.1.0")
        .author("filip bielejec <fbielejec@gmail.com>")
        .about("high-throughput tool for testing websocket APIs")
        .arg(Arg::with_name("verbose?")
             .short("v")
             .long("verbose")
             .takes_value(true)
             .help("increase verbosity: true | false"))
        .arg(Arg::with_name("gateway-url")
             .short("g")
             .long("gateway")
             .takes_value(true)
             .help("the URL of the websocket gateway endpoint"))
        .arg(Arg::with_name("connections")
             .short("c")
             .long("connections")
             .takes_value(true)
             .help("the number of concurrent connections to open"))
        .get_matches();

    let config = Config {
        gateway_url: match matches.value_of("gateway-url") {
            None => panic!("Unknown gateway-url argument"),
            Some(url) => {
                match Url::parse(url) {
                    Ok(url) => url,
                    Err(_) => panic!("Could not parse gateway url"),
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

    let h = Histogram::<u64>::new_with_bounds(1, 60000, 3).unwrap();
    let hist = Arc::new (Mutex::new (h));
    let mut tasks = Vec::with_capacity(config.n_connections);

    for id in 0..config.n_connections {
        let config = config.clone ();
        let hist = Arc::clone(&hist); // hist.clone ();
        tasks.push(task::spawn(async move {
            client(&id, &config, hist).await;
        }));
    }

    task::block_on(async {
        for t in tasks {
            t.await;
        }
    });

    Ok (())
}

async fn client(client_id: &usize,
                config : &Config,
                hist : Arc<Mutex<Histogram::<u64>>>) {

    let Config { gateway_url, .. } = config;
    let (mut ws_stream, _) = connect_async(gateway_url).await.unwrap ();

    // send first message to get the loop going
    ws_stream.send(Message::text("PING")).await.unwrap ();
    let mut start_time = Instant::now();

    // request-response loop
    while let Some(msg) = ws_stream.next().await {

        let response_time = start_time.elapsed().as_millis();

        let msg = msg.unwrap ();
        if msg.is_text() || msg.is_binary() {
            ws_stream.send(Message::text("PING")).await.unwrap ();
            start_time = Instant::now();

            // threads should not fail while holding the lock.
            let mut hist = hist.lock().unwrap();
            *hist += response_time as u64;
            debug!("Client {} on {:?} received {:?}", client_id, thread::current().id(), msg);
            info!("N: {} Min: {} Mean: {:.2}ms Max: {}", hist.len (), hist.min (), hist.mean (), hist.max ());

        }
    }

}
