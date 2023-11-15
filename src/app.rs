use std::process::exit;
use std::string::String as String;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;
use crate::db::{MasterEntries, Book, Publisher, Article, MonthYear};
use async_trait::async_trait;
use crossterm::event::{KeyCode, Event::Key};
use crossterm::{event, execute};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::{Terminal};
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Layout, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs};
use sqlx::SqlitePool;
use tokio::{fs};
use tokio::io::Stderr;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tui_textarea::{TextArea};
use crate::{DB_PATH, DB_URL};


// currently only adding new items. later add ability to search and edit items.

pub(crate) enum AppEvent<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum MenuItem {
    Home,
    Book,
    ShowBooks,
    NewBook,
    Article,
    ListArticles,
    InsertArticle,
}

pub struct App<'a> {
    pub menu_titles: Vec<&'a str>,
    pub index: usize,
    pub active_menu_item: MenuItem,
    pub book_list_state: Mutex<ListState>,
    // pub terminal: Arc<Terminal<CrosstermBackend<io::Stderr>>>,
}

impl<'a> App<'a> {
    pub fn new() -> tokio::io::Result<Self> {
        let mut stderr = tokio::io::stderr();
        if !is_raw_mode_enabled()? {
            enable_raw_mode()?;
            execute!(stderr, EnterAlternateScreen)?;
        };
        // let backend = CrosstermBackend::new(stderr);
        Ok(
            App {
                menu_titles: vec!["Home", "Books", "Show Books", "New Book", "Articles", "List Articles", "Insert Articles", "Quit"],
                index: 0,
                active_menu_item: MenuItem::Home,
                book_list_state: Mutex::new(ListState::default()),
                // terminal: Arc::new(Terminal::new(backend)?),
            })
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.menu_titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.menu_titles.len() - 1;
        }
    }

    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stderr>>) {
        // setup mpsc to handle the channels in the rendering loop
        let (tx, mut rx) = mpsc::channel(1);
        let tick_rate = Duration::from_millis(100);
        tokio::spawn(async move {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll works") {
                    if let Key(key) = event::read().expect("can read events") {
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

        loop {
            // let term = self.terminal.clone();
            terminal.draw( move |frame|{
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Length(3), // main menu chunks[0]
                            Constraint::Min(2), // content chunks[1]
                            Constraint::Length(3), // copyright chunks[2]
                        ]
                            .as_ref(),
                    )
                    .split(terminal.size().expect("can size terminal"));

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

                frame.render_widget(copyright, chunks[2]);

                // Main menu section
                let menu = self.menu_titles
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
                    .select(self.active_menu_item as usize)
                    .block(Block::default().title("Menu").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .divider(Span::raw("|"));

                frame.render_widget(tabs, chunks[0]);

                // Change to a different menu item
                match self.active_menu_item {
                    MenuItem::Home => frame.render_widget(App::render_home(), chunks[1]),

                    MenuItem::Book => frame.render_widget(App::render_book_page(), chunks[1]),

                    MenuItem::ShowBooks => {
                        self.book_list_state.lock().expect("can lock state").select(Some(0));
                        let book_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                            )
                            .split(chunks[1]);


                        let (left, right) = self.render_books();
                        let mut lock = self.book_list_state.lock().expect("can lock state");
                        frame.render_stateful_widget(left, book_chunks[0],  &mut *lock);
                        drop(lock);
                        frame.render_widget(right, book_chunks[1]);

                    }

                    MenuItem::NewBook => {
                        let book_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                            )
                            .split(chunks[1]);

                        let text_area = self.render_add_book();


                        let widget = text_area.widget();

                        frame.render_widget(widget, book_chunks[1]);

                        // todo! take input and save to db
                        // match crossterm::event::read().expect("can read tui text_area").into() {
                        //     Input { KeyCode::Esc } => {},
                        //     input => {
                        //         text_area.input(input);
                        //     }
                        // }
                    }

                    MenuItem::Article => {}
                    MenuItem::ListArticles => {}
                    MenuItem::InsertArticle => {}
                }
            }).expect("can finish terminal");


            match rx.recv().await {
                Some(AppEvent::Input(event)) => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode().expect("can disable raw mode");
                        terminal.show_cursor().expect("can show cursor");
                        exit(0);
                    }
                    KeyCode::Char('h') => self.active_menu_item = MenuItem::Home,
                    KeyCode::Char('b') => self.active_menu_item = MenuItem::Book,
                    KeyCode::Char('s') => self.active_menu_item = MenuItem::ShowBooks,
                    KeyCode::Char('n') => self.active_menu_item = MenuItem::NewBook,
                    KeyCode::Char('a') => self.active_menu_item = MenuItem::Article,
                    KeyCode::Char('l') => self.active_menu_item = MenuItem::ListArticles,
                    KeyCode::Char('i') => self.active_menu_item = MenuItem::InsertArticle,
                    _ => {}
                },
                Some(AppEvent::Tick) => {}
                None => {}
            }
        }
    }

    async fn read_db() -> Result<Vec<Book>, tokio::io::Error> {
        let db_content = fs::read_to_string(&DB_PATH).await?;
        let parsed: Vec<Book> = serde_json::from_str(&db_content).unwrap();
        Ok(parsed)
    }

    fn render_add_book(&self) -> TextArea<'static> {
        let new_book = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("New Book")
            .border_type(BorderType::Plain);

        let lines = vec![
            "{".to_string(),
            "    \"title\": \"\",".to_string(),
            "    \"author\": \"\",".to_string(),
            "    \"pages\": \"\",".to_string(),
            "    \"volume\": \"\",".to_string(),
            "    \"edition\": \"\",".to_string(),
            "    \"series\": \"\",".to_string(),
            "    \"note\": \"\"".to_string(),
            "},".to_string()
            ];

        let mut textarea = TextArea::new(lines);
        textarea.set_block(new_book);
        textarea
    }

    fn render_books(&self) -> (List<'a>, Table<'a>) {
        let books = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Books")
            .border_type(BorderType::Plain);

        let book_list = App::read_db().expect("can fetch book list");
        let items: Vec<_> = App::read_db().expect("can fetch book list")
            .iter()
            .map(|book| {
                ListItem::new(Line::from(vec![Span::styled(
                    book.title.clone(),
                    Style::default(),
                )]))
            })
            .collect();

        let mut lock = self.book_list_state.lock().expect("can lock state");
        let selected_book = book_list
            .get(
                lock
                    .selected()
                    .expect("there is always a selected book"),
            )
            .expect("exists")
            .clone();
        drop(lock);

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

    fn render_home() -> Paragraph<'a> {
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

    fn render_book_page() -> Paragraph<'a> {
        let home = Paragraph::new(vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Books Home",
                Style::default().fg(Color::LightBlue),
            )]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Press 's' to show a list of Books")]),
            Line::from(vec![Span::raw("Press 'n' to add a new book.")]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Book options")
                .border_type(BorderType::Plain),
        );
        home
    }
}

// impl<'a> Drop for App<'a> {
//     fn drop(&mut self) {
//         self.terminal.show_cursor().unwrap();
//         if !is_raw_mode_enabled().unwrap() {
//             return;
//         }
//         disable_raw_mode().unwrap();
//         execute!(terminal.backend_mut(), LeaveAlternateScreen)
//             .unwrap();
//     }
// }





/// Database structures
#[async_trait]
pub trait TableInsert {
    async fn insert(&self) {} // maybe use return type Result<> here?
}

impl MasterEntries {
    pub fn new_book() -> MasterEntries {
        let key = Uuid::new_v4().to_string();
        MasterEntries {
            cite_key: key,
            entry_type: "BOOK".parse().unwrap(),
        }
    }

    pub fn new_article() -> MasterEntries {
        let key = Uuid::new_v4().to_string();
        MasterEntries {
            cite_key: key,
            entry_type: "ARTICLE".parse().unwrap()
        }
    }
}

#[async_trait]
impl TableInsert for MasterEntries {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO master_entries (cite_key, entry_type) VALUES (?,?,)")
                .bind(&self.cite_key)
                .bind(&self.entry_type)
                .execute(&*db)
                .await;

            match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
            };
    }
}

impl Book {
    /// Create and Add book to SQLite database
    async fn book_transaction() {
        let master = MasterEntries::new_book();
        let publisher = Publisher::new();
        let year = String::new();
        let m_y = MonthYear::new(year);
        let book_id = Uuid::new_v4().to_string();
        let book = Book {
            book_id,
            cite_key: master.cite_key.clone(),
            publisher_id: publisher.publisher_id.clone(),
            month_year_id: m_y.month_year_id.clone(),
            author: String::new(),
            title: String::new(),
            pages: String::new(),
            volume: String::new(),
            edition: String::new(),
            series: String::new(),
            note: String::new(),
        };

        master.insert().await;
        book.insert().await;
        publisher.insert().await;
        m_y.insert().await;
    }
}

#[async_trait]
impl TableInsert for Book {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO book (book_id, cite_key, publisher_id, month_year_id, editor, title, pages, volume, edition, series, notes) VALUES (?,?,?,?,?,?,?,?,?,?,?,)")
            .bind(&self.book_id)
            .bind(&self.cite_key)
            .bind(&self.publisher_id)
            .bind(&self.month_year_id)
            .bind(&self.author)
            .bind(&self.title)
            .bind(&self.pages)
            .bind(&self.volume)
            .bind(&self.edition)
            .bind(&self.series)
            .bind(&self.note)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}


impl MonthYear {
    pub fn new(year: String) -> MonthYear {
        let month_year_id = Uuid::new_v4().to_string();
        MonthYear {
            month_year_id,
            month: String::new(),
            year,
        }
    }
}

#[async_trait]
impl TableInsert for MonthYear {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO month_year (month_year_id, month, year) VALUES (?,?,?,)")
            .bind(&self.month_year_id)
            .bind(&self.month)
            .bind(&self.year)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}

impl Article {
    /// Create and Add book to SQLite database
    async fn article_transaction() {
        let master = MasterEntries::new_article();
        let publisher = Publisher::new();
        let year = String::new();
        let m_y = MonthYear::new(year);
        let article_id = Uuid::new_v4().to_string();
        let article = Article {
            cite_key: master.cite_key.clone(),
            article_id,
            publisher_id: publisher.publisher_id.clone(),
            month_year_id: m_y.month_year_id.clone(),
            title: String::new(),
            journal: String::new(),
            volume: String::new(),
            pages: String::new(),
            note: String::new(),
            edition: String::new(),
        };

        master.insert().await;
        article.insert().await;
        publisher.insert().await;
        m_y.insert().await;
    }
}

#[async_trait]
impl TableInsert for Article {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO article (cite_key, article_id, publisher_id, month_year_id, title, journal, volume, pages, note, edition) VALUES (?,?,?,?,?,?,?,?,?,?)")
            .bind(&self.cite_key)
            .bind(&self.article_id)
            .bind(&self.publisher_id)
            .bind(&self.month_year_id)
            .bind(&self.title)
            .bind(&self.journal)
            .bind(&self.volume)
            .bind(&self.pages)
            .bind(&self.note)
            .bind(&self.edition)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}


impl Publisher {
    pub fn new() -> Publisher {
        let publisher_id = Uuid::new_v4().to_string();
        Publisher {
            publisher_id,
            publisher: String::new(),
            address: String::new(),
        }
    }
}

#[async_trait]
impl TableInsert for Publisher {
    async fn insert(&self) {
        let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
        let result = sqlx::query("INSERT INTO publisher (publisher_id, publisher, address) VALUES (?,?,?,)")
            .bind(&self.publisher_id)
            .bind(&self.publisher)
            .bind(&self.address)
            .execute(&*db)
            .await;

        match result {
            Ok(rs) => eprintln!("Row inserted: {:?}", rs),
            Err(e) => eprintln!("Error inserting row: {}", e),
        };
    }
}


// // Implement later
// impl Relationship {
//     pub fn new(master_key: String) -> Relationship {
//         let parent_id = Uuid::new_v4().to_string();
//         let child_id = Uuid::new_v4().to_string();
//         Relationship {
//             parent_id,
//             child_id,
//             cite_key: master_key,
//         }
//     }
// }
//
// #[async_trait]
// impl TableInsert for Relationship {
//     async fn insert(&self) {
//         let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
//         let result = sqlx::query("INSERT INTO relationship (parent_id, child_id, cite_key) VALUES (?,?,?,)")
//             .bind(&self.parent_id)
//             .bind(&self.child_id)
//             .bind(&self.cite_key)
//             .execute(&*db)
//             .await;
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }

// // Implement later
// impl Author {
//     pub fn new(master_key: String) -> Author {
//         let author_id = Uuid::new_v4().to_string();
//         Author {
//             cite_key: master_key,
//             author_id,
//             authors: String::new(),
//         }
//     }
// }
//
// #[async_trait]
// impl TableInsert for Author {
//     async fn insert(&self) {
//         let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
//         let result = sqlx::query("INSERT INTO author (cite_key, author_id, authors) VALUES (?,?,?,)")
//             .bind(&self.cite_key)
//             .bind(&self.author_id)
//             .bind(&self.authors)
//             .execute(&*db)
//             .await;
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }
// // Implement later
// impl Organizations {
//     pub fn new() -> Organizations {
//         let organization_id = Uuid::new_v4().to_string();
//         Organizations {
//             organization_id,
//             organization: String::new(),
//             address: String::new(),
//         }
//     }
// }
//
// #[async_trait]
// impl TableInsert for Organizations {
//     async fn insert(&self) {
//         let db = Arc::new(SqlitePool::connect(DB_URL).await.unwrap());
//         let result = sqlx::query("INSERT INTO organizations (organization_id, organization, address) VALUES (?,?,?,)")
//             .bind(&self.organization_id)
//             .bind(&self.organization)
//             .bind(&self.address)
//             .execute(&*db)
//             .await;
//
//         match result {
//             Ok(rs) => eprintln!("Row inserted: {:?}", rs),
//             Err(e) => eprintln!("Error inserting row: {}", e),
//         };
//     }
// }
