mod app;
mod game;
mod normalize;
mod widgets;

use crate::app::App;

rust_i18n::i18n!();

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    rust_i18n::set_locale(env!("SOLETRA_RS_LANGUAGE"));
    let mut terminal = ratatui::init();
    let result = App::init().await.run(&mut terminal).await;
    ratatui::restore();
    result
}
