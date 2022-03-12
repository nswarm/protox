use crate::Config;
use anyhow::{anyhow, Result};

pub fn generate(config: &Config) -> Result<()> {
    // if config.templates.is_empty() {
    //     return Ok(());
    // }
    // let descriptor_set = load_descriptor_set(&config)?;
    generate_from_descriptor_set(config)?;
    Ok(())
}

fn generate_from_descriptor_set(config: &Config) -> Result<()> {
    // let mut engine = rhai::Engine::new();
    // let script = config.input.join("../scripts/test.rhai");
    // engine.register_fn("do_things", do_things);
    //
    // // let r: rhai::Scope;
    // // let c = engine.run_file_with_scope();
    // // let a = engine.eval_file("".into());
    // // let b = engine.compile_file("".into());
    //
    // if let Err(err) = engine.run_file(script) {
    //     return Err(anyhow!("Error running script: {}", err));
    // }
    Ok(())
}

fn do_things(x: i64, s: &str) {
    println!("rust fn 'do_things': {}, {}", x, s);
}
