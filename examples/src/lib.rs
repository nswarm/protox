use std::process::Command;

pub fn struct_ffi_gen() {
    let root_dir = env!("CARGO_MANIFEST_DIR");
    println!("Running from: {:?}", std::env::current_dir().unwrap());
    println!("manifest dir: {}", env!("CARGO_MANIFEST_DIR"));

    // Command::new("cargo run --bin struct-ffi-gen -- asdfasdf").spawn().unwrap().wait();
    // Command::new

    // let exe_path = env!("CARGO_BIN_EXE_struct-ffi-gen");
    // println!("{}", exe_path);/**/
}
