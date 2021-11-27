use examples::examples_dir;
use std::fs;
use std::process::Command;

fn main() {
    let examples = find_all_examples();
    println!();
    println!("building all examples...");
    build_examples();
    println!();
    for example in examples {
        run_example(&example);
    }
}

fn build_examples() {
    let _ = Command::new("cargo")
        .args(["build", "--examples", "--quiet"])
        .spawn()
        .unwrap()
        .wait()
        .expect("Failed to run example");
}

fn run_example(name: &str) {
    if name.is_empty() || name == "all" {
        return;
    }
    let _ = Command::new("cargo")
        .args(["run", "--quiet", "--example", name])
        .spawn()
        .unwrap()
        .wait()
        .expect("Failed to run example");
}

fn find_all_examples() -> Vec<String> {
    let examples_dir = examples_dir();
    let dirs = fs::read_dir(examples_dir).unwrap();
    dirs.filter_map(|entry| match entry {
        Ok(entry) if entry.file_type().unwrap().is_dir() => {
            Some(entry.file_name().to_str().unwrap().to_string())
        }
        _ => None,
    })
    .collect()
}
