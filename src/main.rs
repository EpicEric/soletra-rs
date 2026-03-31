mod app;
mod game;
mod normalize;

use crate::app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::init().run(terminal))?;
    Ok(())
}
