mod app;
mod game;
mod normalize;
mod widgets;

use crate::app::App;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::init().await.run(&mut terminal).await;
    ratatui::restore();
    result
}
