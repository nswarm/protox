use examples::{
    find_all_with_extension, input_dir, input_path, output_dir, output_path, struct_ffi_gen, Result,
};
use std::fs::create_dir_all;

fn main() -> Result<()> {
    let _ = create_dir_all(output_path()?);
    let input = input_dir()?;
    let output = output_dir()?;
    let proto_paths = find_all_with_extension(input_path()?.as_path(), ".proto")?;
    let mut args = vec!["--proto_path", &input, "--csharp_out", &output];
    for proto_path in &proto_paths {
        args.push(proto_path.to_str().unwrap());
    }
    struct_ffi_gen(&args)?;
    Ok(())
}
