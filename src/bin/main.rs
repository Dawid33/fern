#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
#![allow(ambiguous_glob_reexports)]

use std::error::Error;

use flexi_logger::Logger;
extern crate core;

fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("trace, core::grammar = info")?
        .format(flexi_logger::colored_default_format)
        .start_with_specfile("log.toml")?;
    libfern::fern::compile()?;
    // json::compile()?;
    Ok(())
}
