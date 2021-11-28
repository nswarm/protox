use log::info;
use std::{env};
use std::process::Command;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let protoc_args = collect_protoc_args(&args);
    protoc(&protoc_args);
}

fn collect_protoc_args(args: &Vec<String>) -> Vec<String> {
    // Slice off the first arg (typically exe name) and pass all actual args to protoc.
    args[1..].to_vec()
}

fn protoc(args: &Vec<String>) {
    info!("running:\nprotoc {:?}", args.join(" "));
    let mut child = Command::new("protoc")
        .args(args)
        .spawn()
        .expect("Failed to execute protoc");
    match child.wait() {
        Ok(status) => {
            if !status.success() {
                println!("Exited with status {}", status);
            }
        }
        Err(err) => println!("Exited with error {}", err),
    }
}
