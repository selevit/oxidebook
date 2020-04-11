pub mod core;
pub mod order_book;
pub mod rest_api;
use std::env;
use std::process::exit;
use std::thread;
use tokio;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rest-api|core|all>", args[0]);
        exit(1);
    }
    let module = args[1].as_str();

    match module {
        "core" => core::run(),
        "rest-api" => rest_api::run(),
        "all" => {
            let mut threads = vec![];
            threads.push(thread::spawn(core::run));
            threads.push(thread::spawn(rest_api::run));
            for t in threads {
                t.join().unwrap();
            }
        }
        _ => {
            eprintln!("Unsupported module: {}", module);
            exit(1);
        }
    };
}
