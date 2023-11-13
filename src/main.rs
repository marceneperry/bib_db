
mod ui;
mod app;
mod db;


use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::{error::Error, fs, io, thread};
use std::process::exit;
use std::time::Duration;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Layout, Modifier, Style};
// use ratatui::prelude::Marker::Block;
use ratatui::Terminal;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Widget, Block, Borders, BorderType, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs};
// use ratatui_textarea::TextArea;
// use ratatui_textarea;

use tui_textarea::{Input, Key, TextArea};
use tokio::sync::mpsc;
use tokio::time::Instant;
use crate::{
    app::{App, AppEvent, MenuItem},
};
use crate::app::{ArticleMenuItem, BookMenuItem};
use crate::db::Book;

/// If database is not already created, initialize it by running `init_db` binary crate.
/// Update const DB_URL to match what you have named it in `init_db`
const DB_URL: &str = "sqlite://bibliographic_db/bib_data.db";
const DB_PATH: &str = "bibliographic_db/db.json";
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // open database

    // setup terminal
    enable_raw_mode()?;


    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();


    // setup mpsc to handle the channels in the rendering loop
    let (tx, mut rx) = mpsc::channel(10);
    let tick_rate = Duration::from_millis(200);
    tokio::spawn(async move  {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let Event::Key(key) = event::read().expect("can read events") {
                    tx.send(AppEvent::Input(key)).await.expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(AppEvent::Tick).await {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let menu_titles = vec!["Home", "Books", "Articles", "Quit"];
    let book_menu_titles = vec!["Home", "Show Books", "New Book", "Quit"];
    let article_menu_titles = vec!["Home", "Show Articles", "New Article", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut book_menu_item = BookMenuItem::ShowBooks;
    let mut article_menu_item = ArticleMenuItem::ShowArticles;

    let mut book_list_state = ListState::default();
    book_list_state.select(Some(0));

    let mut textarea = TextArea::default();

    // terminal loop
    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3), // main menu chunks[0]
                        Constraint::Length(3), // submenu chunks[1]
                        Constraint::Min(2), // content chunks[2]
                        Constraint::Length(3), // copyright chunks[3]
                    ]
                        .as_ref(),
                )
                .split(size);

            // Copyright section
            let copyright = Paragraph::new("Library DB 2023 - all rights reserved")
                .style(Style::default().fg(Color::Rgb(35, 70, 184)))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );

            rect.render_widget(copyright, chunks[3]);

            // Main menu section
            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Line::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);

            // Change to a different menu item
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[2]),

                MenuItem::Book => {
                    match book_menu_item {
                        BookMenuItem::ShowBooks => {
                            // Book menu
                            let book_menu = book_menu_titles
                                .iter()
                                .map(|t| {
                                    let (first, rest) = t.split_at(1);
                                    Line::from(vec![
                                        Span::styled(
                                            first,
                                            Style::default()
                                                .fg(Color::Yellow)
                                                .add_modifier(Modifier::UNDERLINED),
                                        ),
                                        Span::styled(rest, Style::default().fg(Color::White)),
                                    ])
                                })
                                .collect();

                            let book_tabs = Tabs::new(book_menu)
                                .select(book_menu_item.into())
                                .block(Block::default().title("Book Menu").borders(Borders::ALL))
                                .style(Style::default().fg(Color::White))
                                .highlight_style(Style::default().fg(Color::Yellow))
                                .divider(Span::raw("|"));


                            let book_chunks = Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints(
                                    [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                                )
                                .split(chunks[2]);

                            rect.render_widget(book_tabs, chunks[1]);

                            let (left, right) = render_books(&book_list_state);
                            rect.render_stateful_widget(left, book_chunks[0], &mut book_list_state);
                            rect.render_widget(right, book_chunks[1]);
                        }
                        BookMenuItem::AddBook => {
                            // Book menu
                            let book_menu = book_menu_titles
                                .iter()
                                .map(|t| {
                                    let (first, rest) = t.split_at(1);
                                    Line::from(vec![
                                        Span::styled(
                                            first,
                                            Style::default()
                                                .fg(Color::Yellow)
                                                .add_modifier(Modifier::UNDERLINED),
                                        ),
                                        Span::styled(rest, Style::default().fg(Color::White)),
                                    ])
                                })
                                .collect();

                            let book_tabs = Tabs::new(book_menu)
                                .select(book_menu_item.into())
                                .block(Block::default().title("Book Menu").borders(Borders::ALL))
                                .style(Style::default().fg(Color::White))
                                .highlight_style(Style::default().fg(Color::Yellow))
                                .divider(Span::raw("|"));


                            let book_chunks = Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints(
                                    [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                                )
                                .split(chunks[2]);

                            rect.render_widget(book_tabs, chunks[1]);

                            let widget = textarea.widget();
                        }
                    }
                }
                MenuItem::Article => {}
            }
        }).expect("can't finish terminal");

        match rx.recv().await {
            Some(AppEvent::Input(event)) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode().expect("can disable raw mode");
                    terminal.show_cursor().expect("can show cursor");
                    exit(0);
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('b') => active_menu_item = MenuItem::Book,
                KeyCode::Char('a') => active_menu_item = MenuItem::Article,
                KeyCode::Char('n') => book_menu_item = BookMenuItem::AddBook,
                KeyCode::Char('s') => book_menu_item = BookMenuItem::ShowBooks,

                _ => {}
            },
            Some(AppEvent::Tick) => {}
            None => {}
        }
    }
}

// async fn run_app<B: Backend>(terminal: &mut Terminal<B>, _app: App<'_>) {
//
// }

fn read_db() -> Result<Vec<Book>, io::Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Book> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

// fn render_add_book() -> Box<dyn Widget> {
//     let new_book = Block::default()
//         .borders(Borders::ALL)
//         .style(Style::default().fg(Color::White))
//         .title("New Book")
//         .border_type(BorderType::Plain);
//
//     let lines = vec![
//         "{".to_string(),
//         "    \"title\": \"\",".to_string(),
//         "    \"author\": \"\",".to_string(),
//         "    \"pages\": \"\",".to_string(),
//         "    \"volume\": \"\",".to_string(),
//         "    \"edition\": \"\",".to_string(),
//         "    \"series\": \"\",".to_string(),
//         "    \"note\": \"\"".to_string(),
//         "},".to_string()
//         ];
//
//     let textarea = TextArea::new(lines);
//
//     textarea.widget()
// }

fn render_books<'a>(book_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let books = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Books")
        .border_type(BorderType::Plain);

    let book_list = read_db().expect("can fetch book list");
    let items: Vec<_> = book_list
        .iter()
        .map(|book| {
            ListItem::new(Line::from(vec![Span::styled(
                book.title.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_book = book_list
        .get(
            book_list_state
                .selected()
                .expect("there is always a selected book"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(books).highlight_style(
        Style::default()
            .bg(Color::LightBlue)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let book_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_book.book_id)),
        Cell::from(Span::raw(selected_book.title)),
        Cell::from(Span::raw(selected_book.author)),
        Cell::from(Span::raw(selected_book.pages)),
        Cell::from(Span::raw(selected_book.volume)),
        Cell::from(Span::raw(selected_book.edition)),
        Cell::from(Span::raw(selected_book.series)),
        Cell::from(Span::raw(selected_book.note)),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Title",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Author",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Pages",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Volume",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Edition",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Series",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Note",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Book Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(5),
        Constraint::Percentage(5),
        Constraint::Percentage(5),
        Constraint::Percentage(10),
        Constraint::Percentage(20),
    ]);

    (list, book_detail)
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Welcome")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("to")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "Library DB",
            Style::default().fg(Color::LightBlue),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press 'b' to access Books, 'a' to access Articles.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}