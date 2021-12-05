use crate::Options;
use anyhow::Result;

pub struct Protoc {}

impl Protoc {
    pub fn with_options(opt: &Options) -> Result<Self> {
        Ok(Self {})
    }

    pub fn run(self) -> Result<()> {
        // run the thing
        todo!()
    }
}

// fn protoc(args: &[String]) {
//     let protoc_path = prost_build::protoc();
//     info!("located protoc: {:?}", protoc_path);
//     info!("running:\nprotoc {:?}", args.join(" "));
//     let mut child = Command::new(protoc_path)
//         .args(args)
//         .spawn()
//         .expect("Failed to execute protoc");
//     match child.wait() {
//         Ok(status) => {
//             if !status.success() {
//                 println!("Exited with status {}", status);
//             }
//         }
//         Err(err) => println!("Exited with error {}", err),
//     }
// }
