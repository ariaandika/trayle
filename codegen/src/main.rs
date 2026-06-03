use std::env::args;
use std::fs::read_to_string;

use crate::str::Str;
use crate::error::Error;
use crate::io::Write;

mod str;
mod io;
mod error;
mod parser;
mod schema;

fn main() -> Result<(), Error> {
    let Some(path) = args().nth(1) else {
        return Err(Error::new("path argument is required"));
    };

    let content = Str::new(read_to_string(path)?);
    let _result = parser::parse_wayland(content)?;
    writeln!(std::io::stdout().lock(), "test");

    Ok(())
}

