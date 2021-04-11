pub mod core;
pub mod order_book;
pub mod protocol;
pub mod rest_api;
pub mod ws_md_api;
pub mod outbox;
pub mod transport;
use std::env;
use std::process::exit;
use std::thread;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let module = match args.len() {
        1 => "all",
        2 => args[1].as_str(),
        _ => {
            eprintln!("Usage: {} <rest-api|core|ws-md-api|all>", args[0]);
            exit(1);
        }
    };

    match module {
        "core" => core::run().unwrap(),
        "rest-api" => rest_api::run().unwrap(),
        "ws-md-api" => ws_md_api::run().unwrap(),
        "all" => {
            let mut threads = vec![];
            threads.push(thread::spawn(core::run));
            threads.push(thread::spawn(rest_api::run));
            threads.push(thread::spawn(ws_md_api::run));
            for t in threads {
                if let Err(e) = t.join().unwrap() {
                    panic!(e)
                }
            }
        }
        _ => {
            eprintln!("Unsupported module: {}", module);
            exit(1);
        }
    };
}
