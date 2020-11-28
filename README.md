# [ws-load-test](https://github.com/fbielejec/ws-load-test)

ws-load-test is a high-throughput tool for testing websocket APIs.

It will open a specified number concurrent connections to the websocket endpoint and start flooding it with PING requests, collecting measured time until the response arrives.

## Blog Article:

* [Blog article](https://www.blog.nodrama.io/rust-websocket/)

## Usage

**FLAGS:**
- `-h, --help`      Prints help information
- `-V, --version`   Prints version information

**OPTIONS:**
- `-c, --connections`   the number of concurrent connections to open
- `-g, --gateway`       the URL of the websocket gateway endpoint
- `-v, --verbose`       increase verbosity: true | false

```bash
cargo run -- -v debug -c 3 -g ws://echo.websocket.org
```

## Development

Measurements and the reported statistics (count, min, mean, max) are collected across all client connection tasks using Rust port of
[High Dynamic Range Histograms](https://github.com/HdrHistogram/HdrHistogram_rust), a low-latency, high-range histogram implementation.

Concurrent tasks (WS connections) rely on the [async-std](https://github.com/async-rs/async-std) asynchronous runtime, which chooses how to run them, i.e. how many threads to start and how to distribute tasks on them.
