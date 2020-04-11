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
        "rest-api" => rest_api::run().await,
        "all" => {
            let t_core = thread::spawn(core::run);
            rest_api::run().await;
            t_core.join().expect("error in the core thread");
        }
        _ => {
            eprintln!("Unsupported module: {}", module);
            exit(1);
        }
    };
}
