mod shared;

use async_std::task;
use async_tungstenite::async_std::connect_async;
use async_tungstenite::tungstenite::protocol::Message;
use futures::StreamExt;
use futures::prelude::*;
use hdrhistogram::Histogram as HdrHistogram;
use log::{debug, info, error};
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use url::Url;

type MainResult<T> = Result<T, Box<dyn Error>>;

fn main() -> MainResult<()> {

    let matches = shared::build_app ().get_matches();
    let config = shared::build_config (&matches);

    env::set_var("RUST_LOG", &config.verbosity);
    env_logger::init();
    info!("Running with config {:#?}", &config);

    let h = HdrHistogram::<u64>::new_with_bounds(1, 60000, 3).unwrap();
    let hist = Arc::new (Mutex::new (h));
    let mut tasks = Vec::with_capacity(config.n_connections);
    let tick = Instant::now();

    for id in 0..config.n_connections {
        let config = config.clone ();
        let url = Url::parse(&config.url)?;
        let hist = Arc::clone(&hist); // hist.clone ();
        tasks.push(task::spawn(async move {
            client(&id, &url, hist).await;
        }));
    }

    task::block_on(async {
        for t in tasks {
            t.await;
        }
    });

    let hist = hist.lock().unwrap();
    shared::print_stats (&hist, tick);

    Ok (())
}

// TODO : store response status
async fn client (client_id: &usize,
                 url: &Url,
                 hist : Arc<Mutex<HdrHistogram::<u64>>>) {

    // TODO : handle conn error
    // let (mut ws_stream, _) =
    match connect_async(url).await {
        Ok ((mut ws_stream, _)) => {

            let start_time = Instant::now();
            ws_stream.send(Message::text("PING")).await.unwrap ();
            let response = ws_stream.next().await.unwrap ();
            let response_time = start_time.elapsed().as_millis();

            // threads should not fail while holding the lock
            let mut hist = hist.lock().unwrap();
            *hist += response_time as u64;

            debug!("Client {} on {:?} received {:?}, latency: {} ms", client_id, thread::current().id(), response, response_time);
        },
        Err (error) => {
            error!("Error: {:?}", error);
        }

    };
}
