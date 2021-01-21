use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins};
use plotlib::view::ContinuousView;
use hdrhistogram::Histogram as HdrHistogram;
use std::time::Instant;
use clap::{Arg, App, ArgMatches};

#[derive(Clone, Debug)]
pub struct Config {
    pub url: String,
    pub n_connections: usize,
    pub verbosity: String
}

pub fn build_app () -> App<'static, 'static> {
    App::new("ws-load-test")
        .version("0.1.0")
        .author("filip bielejec <fbielejec@gmail.com>")
        .about("high-throughput tool for testing websocket APIs")
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
}

pub fn build_config (matches: &ArgMatches) -> Config {
    Config {
        url: match matches.value_of("url") {
            None => panic!("Unknown url argument"),
            Some(url) => url.to_string ()
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
        verbosity: match matches.value_of("verbosity") {
            None => "info".to_string (),
            Some (v) =>
                v.parse::<String> ().expect ("Could not parse verbosity argument")
        }
    }
}

pub fn print_stats (hist : &HdrHistogram::<u64>, tick : Instant) {

    let tock = tick.elapsed().as_millis();

    println!("Summary:\n Requests:   {}\n Total:      {} ms\n Slowest:    {} ms\n Fastest:    {} ms\n Average:    {:.1} ms\n Throughput: {:.1} request/s",
             hist.len (),
             tock,
             hist.max (),
             hist.min (),
             hist.mean (),
             1000.0 * hist.len () as f64 / tock as f64
    );

    println!("Cumulative distribution of the response times:\n  5% ≤ {:.2} ms\n 10% ≤ {:.2} ms\n 50% ≤ {:.2} ms\n 95% ≤ {:.2} ms\n 99% ≤ {:.2} ms",
             hist.value_at_quantile (0.05f64),
             hist.value_at_quantile (0.1f64),
             hist.value_at_quantile (0.5f64),
             hist.value_at_quantile (0.95f64),
             hist.value_at_quantile (0.95f64)
    );

    let mut data : Vec<f64> = Vec::new();
    hist.iter_recorded()
        .for_each(|value| {
            (0..value.count_at_value ())
                .for_each (|_| {
                    data.push(value.value_iterated_to () as f64);
                });
        });

    let h = Histogram::from_slice(&data, HistogramBins::Count(10));
    let v = ContinuousView::new().add(h);

    println!("Distribution of the response times:");
    println!("{}", Page::single(&v).dimensions(60, 15).to_text().unwrap());
}
