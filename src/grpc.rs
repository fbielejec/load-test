mod shared;
pub mod api {
    tonic::include_proto!("pingpong");
}

use api::ping_pong_client::PingPongClient;
use api::{Ping};
use hdrhistogram::Histogram as HdrHistogram;
use log::{debug, info, error};
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use tonic::transport::{Channel, Endpoint, ClientTlsConfig};
use tonic::{Request, Status, Code};
use http::Uri;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

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
        let hist = hist.clone ();
        let uri = config.url.parse::<Uri>().expect ("Could not parse url");
        let endpoint : Endpoint = Channel::builder (uri);//.tls_config(ClientTlsConfig::new()).unwrap ();

        tasks.push(tokio::spawn(async move {
            client(&id, &endpoint, hist).await;
        }));
    }

    for t in tasks {
        t.await?;
    }

    let hist = hist.lock().unwrap();
    shared::print_stats (&hist, tick);

    Ok(())
}

// TODO : store grpc response status
async fn client (client_id: &usize,
                 url: &Endpoint,
                 hist : Arc<Mutex<HdrHistogram::<u64>>>) {

    let mut client = PingPongClient::connect(url.clone ()).await.unwrap ();
    let start_time = Instant::now();
    let status : Status = match client.send_ping(Request::new(Ping {})).await {
        Ok (_) => Status::new(Code::Ok, "Ok"),
        Err (error) => {
            error!("Error: {:?}", error);
            error
        }
    };
    let response_time = start_time.elapsed().as_millis();

    // threads should not fail while holding the lock
    let mut hist = hist.lock().unwrap();
    *hist += response_time as u64;

    debug!("Client {} on {:?} received {:?}, latency: {} ms", client_id, thread::current().id(), status, response_time);
}
