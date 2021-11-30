use std::{fs, io, result};
use std::error::Error;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::process::Command;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

pub fn exe_name() -> Result<String> {
    let result = std::env::current_exe()?
        .file_stem()
        .ok_or("invalid file_stem")?
        .to_str()
        .ok_or("file_stem not valid str")?
        .to_string();
    Ok(result)
}

pub fn examples_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path
}

pub fn current_example_path() -> Result<PathBuf> {
    let mut path = examples_path();
    path.push(exe_name()?);
    Ok(path)
}

pub fn input_path() -> Result<PathBuf> {
    let mut path = current_example_path()?;
    path.push("input");
    Ok(path)
}

pub fn input_dir() -> Result<String> {
    let result = input_path()?
        .to_str()
        .ok_or("input_path is invalid str")?
        .to_string();
    Ok(result)
}

pub fn output_path() -> Result<PathBuf> {
    let mut path = current_example_path()?;
    path.push("output");
    Ok(path)
}

pub fn output_dir() -> Result<String> {
    let result = output_path()?
        .to_str()
        .ok_or("output_path is invalid str")?
        .to_string();
    Ok(result)
}

pub fn find_all_with_extension<P: AsRef<Path>>(base_path: P, ext: &str) -> Result<Vec<PathBuf>> {
    let results = fs::read_dir(base_path.as_ref())?
        .filter_map(io::Result::ok)
        .filter_map(|entry: DirEntry| {
            let is_file = entry.file_type().unwrap().is_file();
            let is_proto = entry.path().file_name().unwrap().to_str().unwrap().ends_with(ext);
            if is_file && is_proto {
                match entry.path().strip_prefix(base_path.as_ref()) {
                    Err(_) => None,
                    Ok(path) => Some(path.to_path_buf()),
                }
            } else {
                None
            }
        })
        .collect::<Vec<PathBuf>>();
    Ok(results)
}

pub fn struct_ffi_gen(args: &[&str]) -> Result<()> {
    let mut full_args = Vec::<String>::new();
    let base_args = ["run", "--bin", "struct-ffi-gen", "--quiet", "--"];
    for arg in base_args {
        full_args.push(arg.to_string());
    }
    for arg in args {
        full_args.push(arg.to_string());
    }

    println!();
    print_example_info()?;

    let exit_status = Command::new("cargo")
        .args(full_args)
        .env("RUST_LOG", "INFO")
        .spawn()
        .expect("Failed to spawn struct-ffi-gen process")
        .wait()
        .expect("Failed to wait for struct-ffi-gen process");
    assert!(exit_status.success());
    Ok(())
}

fn print_example_info() -> Result<()> {
    println!("running example {}...", exe_name()?);
    println!();
    Ok(())
}
