#![allow(dead_code, unused_variables, clippy::let_unit_value)]

mod app;
mod genshin;
mod global_states;
mod winapi_bindings;

use app::App;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    App::run()?;
    Ok(())
}
