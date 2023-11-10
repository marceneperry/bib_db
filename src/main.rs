
mod ui;
mod app;
mod db;


use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::{error::Error, io};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use crate::{
    app::App,
    ui::ui,

};

/// If database is not already created, initialize it by running `init_db` binary crate.
/// Update const DB_URL to match what you have named it in `init_db`
const DB_URL: &str = "sqlite://bibliographic_db/bib_data.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // open database

    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);


    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // if let Ok(do_print) = res {
    //     if do_print {
    //         app.print_json()?;
    //     }
    // } else if let Err(err) = res {
    //     println!("{err:?}");
    // }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right => app.next(),
                    KeyCode::Left => app.previous(),
                    _ => {}
                }
            }
        }
    }
}

// fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
//     loop {
//         terminal.draw(|f| ui(f, app))?;
//         if let Event::Key(key) = event::read()? {
//             if key.kind == event::KeyEventKind::Release {
//                 continue;
//             }
//             match app.current_screen {
//                 CurrentScreen::Main => match key.code {
//                     KeyCode::Char('b') => {
//                         app.current_screen = CurrentScreen::AddBook;
//                         app.currently_editing = Some(CurrentlyEditing::Book);
//                     }
//                     KeyCode::Char('a') => {
//                         app.current_screen = CurrentScreen::AddArticle;
//                         app.currently_editing = Some(CurrentlyEditing::Article);
//                     }
//                     KeyCode::Char('q') => {
//                         app.current_screen = CurrentScreen::Exiting;
//                     }
//                     _ => {}
//                 },
//                 CurrentScreen::Exiting => match key.code {
//                     KeyCode::Char('y') => {
//                         return Ok(true);
//                     }
//                     KeyCode::Char('n') | KeyCode::Char('q') => {
//                         return Ok(false);
//                     }
//                     _ => {}
//                 },
//                 CurrentScreen::AddBook if key.kind == KeyEventKind::Press => {
//                     match key.code {
//                         KeyCode::Enter => {
//                             if let Some(editing) = &app.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Book => {
//                                         app.currently_editing = Some(CurrentlyEditing::Value);
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         app.save_key_value();
//                                         app.current_screen = CurrentScreen::Main;
//                                     }
//                                 }
//                             }
//                         }
//                         KeyCode::Backspace => {
//                             if let Some(editing) = &app.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Key => {
//                                         app.key_input.pop();
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         app.value_input.pop();
//                                     }
//                                 }
//                             }
//                         }
//                         KeyCode::Esc => {
//                             app.current_screen = CurrentScreen::Main;
//                             app.currently_editing = None;
//                         }
//                         KeyCode::Tab => {
//                             app.toggle_editing();
//                         }
//                         KeyCode::Char(value) => {
//                             if let Some(editing) = &app.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Key => {
//                                         app.key_input.push(value);
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         app.value_input.push(value);
//                                     }
//                                 }
//                             }
//                         }
//                         _ => {}
//                     }
//                 },
//
//                 CurrentScreen::AddArticle if key.kind == KeyEventKind::Press => {
//                     match key.code {
//                         KeyCode::Enter => {
//                             if let Some(editing) = &app.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Key => {
//                                         app.currently_editing = Some(CurrentlyEditing::Value);
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         app.save_key_value();
//                                         app.current_screen = CurrentScreen::Main;
//                                     }
//                                 }
//                             }
//                         }
//                         KeyCode::Backspace => {
//                             if let Some(editing) = &app.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Key => {
//                                         app.key_input.pop();
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         app.value_input.pop();
//                                     }
//                                 }
//                             }
//                         }
//                         KeyCode::Esc => {
//                             app.current_screen = CurrentScreen::Main;
//                             app.currently_editing = None;
//                         }
//                         KeyCode::Tab => {
//                             app.toggle_editing();
//                         }
//                         KeyCode::Char(value) => {
//                             if let Some(editing) = &app.currently_editing {
//                                 match editing {
//                                     CurrentlyEditing::Key => {
//                                         app.key_input.push(value);
//                                     }
//                                     CurrentlyEditing::Value => {
//                                         app.value_input.push(value);
//                                     }
//                                 }
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }
// }