pub fn unquote_arg(arg: &str) -> String {
    arg[1..arg.len() - 1].to_string()
}
