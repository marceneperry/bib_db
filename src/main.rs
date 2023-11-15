
mod ui;
mod app;
mod db;

use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use tokio::io;
use crate::{
    app::{App},
};


/// If database is not already created, initialize it by running `init_db` binary crate.
/// Update const DB_URL to match what you have named it in `init_db`
const DB_URL: &str = "sqlite://bibliographic_db/bib_data.db";
const DB_PATH: &str = "bibliographic_db/db.json";



#[tokio::main]
async fn main() {

    // setup terminal
    enable_raw_mode().expect("can enable raw mode");
    let mut stderr = std::io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend).expect("can run terminal");

    // create app and run it... moved rest to App::new()
    let mut app = App::new();

    app.expect("can run app").run(&mut terminal).await
}




// async fn run_app<B: Backend>(terminal: &mut Terminal<B>, _app: App<'_>) {
//
// }
