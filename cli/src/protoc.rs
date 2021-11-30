// use std::path::PathBuf;
//
// #[derive(Default)]
// struct ProtocCommand {
//     proto_path: PathBuf,
// }
//
// impl ProtocCommand {
//     pub fn new() -> Self {
//         Self::default()
//     }
//
//     pub fn go(&mut self) {}
//
//     /// --proto_path
//     pub fn proto_path(&mut self, path: PathBuf) -> &mut Self {
//         self.proto_path = path;
//         self
//     }
//
//     /// Use for any other protoc --key=value argument.
//     pub fn arg(&mut self, key: &str, value: &str {
//
//     }
//
//     /// Use for any other protoc --flag argument.
//     pub fn flag(&mut self, key: &str, value: &str) {
//
//     }
// }
//
// fn protoc_path() -> PathBuf {
//     prost_build::protoc()
// }
