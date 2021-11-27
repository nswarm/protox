use std::process::Command;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut child = Command::new("protoc")
        .args(args)
        .spawn()
        .expect("Failed to execute protoc");
    
    match child.wait() {
        Ok(status) => println!("Exited with status {}", status),
        Err(err) => println!("Exited with error {}", err),
    }
}
