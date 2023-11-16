use std::sync::{Arc, mpsc, Mutex};
use std::{io, thread};
use std::time::{Duration, Instant};
use crate::db::{Book};
use crossterm::event::{Event, KeyCode};
use crossterm::{event};
use crossterm::terminal::{disable_raw_mode};
use ratatui::backend::{Backend};
use ratatui::{Terminal};
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Layout, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs};
use tui_textarea::{TextArea};
use crate::{DB_PATH};

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
    active_menu_item: MenuItem,
    pub book_list_state: Arc<Mutex<ListState>>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
            App {
                menu_titles: vec!["Home", "Books", "Show Books", "New Book", "Articles", "List Articles", "Insert Articles", "Quit"],
                index: 0,
                active_menu_item: MenuItem::Home,
                book_list_state: Arc::new(Mutex::new(ListState::default())),
            }
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

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        // setup mpsc to handle the channels in the rendering loop
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(100);
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll works") {
                    if let Event::Key(key) = event::read().expect("can read events") {
                        tx.send(AppEvent::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(AppEvent::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        loop {
            // let term = self.terminal.clone();
            let terminal_size = terminal.size().expect("can size terminal");
            let menu_titles = self.menu_titles.iter().cloned();
            let active_menu_item = self.active_menu_item;
            let book_list_state = self.book_list_state.clone();
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
                    .split(terminal_size);

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
                let menu = menu_titles
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
                    .select(active_menu_item as usize)
                    .block(Block::default().title("Menu").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .divider(Span::raw("|"));

                frame.render_widget(tabs, chunks[0]);

                // Change to a different menu item
                match active_menu_item {
                    MenuItem::Home => frame.render_widget(App::render_home(), chunks[1]),

                    MenuItem::Book => frame.render_widget(App::render_book_page(), chunks[1]),

                    MenuItem::ShowBooks => {
                        book_list_state.lock().expect("can lock state").select(Some(0));
                        let book_chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(
                                [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                            )
                            .split(chunks[1]);


                        let (left, right) = App::render_books(book_list_state.clone());
                        let mut lock = book_list_state.lock().expect("can lock state");
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

                        let text_area = App::render_add_book();


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
            })?; //.expect("can finish terminal");


            match rx.recv().expect("can receive") {
               AppEvent::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode().expect("can disable raw mode");
                        terminal.show_cursor().expect("can show cursor");
                        break;
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
                AppEvent::Tick => {}
            }
        }
        Ok(())
        // Ok::<(), Error>(()).unwrap()
        // Err(Error)

    }

    fn read_db() -> Result<Vec<Book>, std::io::Error> {
        let db_content = std::fs::read_to_string(&DB_PATH).unwrap();
        let parsed: Vec<Book> = serde_json::from_str(&db_content).unwrap();
        Ok(parsed)
    }

    fn render_add_book() -> TextArea<'static> {
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

    fn render_books(book_list_state: Arc<Mutex<ListState>>) -> (List<'a>, Table<'a>) {
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

        let lock = book_list_state.lock().expect("can lock state");
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
