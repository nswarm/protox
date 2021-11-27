use std::path::PathBuf;
use std::process::Command;

pub fn exe_name() -> String {
    std::env::current_exe()
        .unwrap()
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn examples_dir() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path
}

pub fn current_example_dir() -> PathBuf {
    let mut path = examples_dir();
    path.push(exe_name());
    path
}

pub fn input_dir() -> PathBuf {
    let mut path = current_example_dir();
    path.push("input");
    path
}

pub fn output_dir() -> PathBuf {
    let mut path = current_example_dir();
    path.push("output");
    path
}

pub fn struct_ffi_gen(args: &[&str]) {
    let mut full_args = Vec::<String>::new();
    let base_args = ["run", "--bin", "struct-ffi-gen", "--"];
    for arg in base_args {
        full_args.push(arg.to_string());
    }
    for arg in args {
        full_args.push(arg.to_string());
    }

    print_example_info(&full_args);

    let exit_status = Command::new("cargo")
        .args(full_args)
        .output()
        .expect("Failed to execute struct-ffi-gen process");
    assert!(exit_status.status.success());
}

fn print_example_info(args: &Vec<String>) {
    println!("running example {}...", exe_name());
    println!("  args: {:?}", args);
    println!("  input: {}", input_dir().display());
    println!("  output: {}", output_dir().display());
    println!();
}
